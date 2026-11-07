use std::collections::HashMap;

use anyhow::{bail, Result};
use svg::{
    node::{element::tag::Type as SvgTagType, Attributes},
    parser::Event,
};

use super::{FontMetrics, Glyph, GlyphId, PositionedGlyph};
use crate::{composition::svg_path_data::parse_svg_path_data, Scale, Spline};

/// Single-stroke SVG 1.1 font (Hershey, EMS); font units, y-up, index 0 is the missing glyph.
pub(crate) struct StrokeFont {
    family: Option<String>,
    units_per_em: f32,
    metrics: FontMetrics,
    glyphs: Vec<StrokeGlyph>,
    by_char: HashMap<char, u32>,
    kerning: HashMap<(u32, u32), f32>,
}

struct StrokeGlyph {
    advance: f32,
    contours: Vec<Spline>,
}

/// An `<hkern>` before glyph names and chars are resolved to ids.
struct RawKern {
    left_chars: Vec<char>,
    left_names: Vec<String>,
    right_chars: Vec<char>,
    right_names: Vec<String>,
    k: f32,
}

impl StrokeFont {
    /// Ligature glyphs (multi-char `unicode`) and `U+XXXX-YYYY` kerning ranges are not supported.
    pub(crate) fn parse(svg: &str) -> Result<StrokeFont> {
        let mut seen_font = false;
        let mut id_name = None;
        let mut family = None;
        let mut units_per_em = 1000.0;
        let mut ascent = None;
        let mut descent = None;
        let mut cap_height = None;
        let mut x_height = None;
        let mut default_advance = 0.0;
        let mut missing_glyph = None;
        let mut glyphs: Vec<StrokeGlyph> = Vec::new();
        let mut by_char: HashMap<char, u32> = HashMap::new();
        let mut by_name: HashMap<String, u32> = HashMap::new();
        let mut raw_kerns: Vec<RawKern> = Vec::new();

        for event in svg::read(svg)? {
            let Event::Tag(tag, tag_type, attributes) = event else {
                if let Event::Error(error) = event {
                    bail!("invalid SVG font: {error}");
                }
                continue;
            };
            if matches!(tag_type, SvgTagType::End) {
                continue;
            }
            match tag {
                "font" => {
                    seen_font = true;
                    id_name = attr(&attributes, "id").map(decode_xml_entities);
                    default_advance = attr_f32(&attributes, "horiz-adv-x").unwrap_or(0.0);
                }
                "font-face" => {
                    family = attr(&attributes, "font-family").map(decode_xml_entities);
                    units_per_em = attr_f32(&attributes, "units-per-em").unwrap_or(1000.0);
                    ascent = attr_f32(&attributes, "ascent");
                    descent = attr_f32(&attributes, "descent");
                    cap_height = attr_f32(&attributes, "cap-height");
                    x_height = attr_f32(&attributes, "x-height");
                }
                "missing-glyph" => {
                    missing_glyph =
                        Some(StrokeGlyph::from_attributes(&attributes, default_advance));
                }
                "glyph" => {
                    // index 0 is reserved for the missing glyph
                    let index = glyphs.len() as u32 + 1;
                    glyphs.push(StrokeGlyph::from_attributes(&attributes, default_advance));
                    let unicode = attr(&attributes, "unicode").map(decode_xml_entities);
                    if let Some(ch) = unicode.as_deref().and_then(single_char) {
                        by_char.entry(ch).or_insert(index);
                    }
                    if let Some(name) = attr(&attributes, "glyph-name") {
                        by_name.entry(name.to_string()).or_insert(index);
                    }
                }
                "hkern" => {
                    if let Some(k) = attr_f32(&attributes, "k") {
                        raw_kerns.push(RawKern {
                            left_chars: kern_chars(attr(&attributes, "u1")),
                            left_names: kern_names(attr(&attributes, "g1")),
                            right_chars: kern_chars(attr(&attributes, "u2")),
                            right_names: kern_names(attr(&attributes, "g2")),
                            k,
                        });
                    }
                }
                _ => {}
            }
        }
        if !seen_font {
            bail!("no <font> element found, not an SVG font");
        }

        let resolve = |chars: &[char], names: &[String]| -> Vec<u32> {
            chars
                .iter()
                .filter_map(|ch| by_char.get(ch))
                .chain(names.iter().filter_map(|name| by_name.get(name)))
                .copied()
                .collect()
        };
        let kerning = raw_kerns
            .iter()
            .flat_map(|kern| {
                let left = resolve(&kern.left_chars, &kern.left_names);
                let right = resolve(&kern.right_chars, &kern.right_names);
                left.into_iter()
                    .flat_map(move |l| right.clone().into_iter().map(move |r| ((l, r), kern.k)))
            })
            .collect();

        let missing_glyph = missing_glyph.unwrap_or(StrokeGlyph {
            advance: default_advance,
            contours: Vec::new(),
        });
        let ascender = ascent.unwrap_or(units_per_em * 0.8);
        let descender = descent.map_or(-units_per_em * 0.2, |descent| -descent.abs());

        Ok(StrokeFont {
            family: family.or(id_name),
            units_per_em,
            metrics: FontMetrics {
                ascender,
                descender,
                line_gap: 0.0,
                cap_height,
                x_height,
            },
            glyphs: std::iter::once(missing_glyph).chain(glyphs).collect(),
            by_char,
            kerning,
        })
    }

    pub(crate) fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    pub(crate) fn units_per_em(&self) -> f32 {
        self.units_per_em
    }

    /// In font units.
    pub(crate) fn metrics(&self) -> FontMetrics {
        self.metrics
    }

    pub(crate) fn has_glyph(&self, ch: char) -> bool {
        self.by_char.contains_key(&ch)
    }

    pub(crate) fn glyph_id(&self, ch: char) -> GlyphId {
        GlyphId(self.by_char.get(&ch).copied().unwrap_or(0))
    }

    /// Kerning of a pair is taken off the left glyph's advance.
    pub(crate) fn shape(&self, text: &str, scale: f32) -> Vec<PositionedGlyph> {
        let ids: Vec<u32> = text.chars().map(|ch| self.glyph_id(ch).0).collect();
        ids.iter()
            .enumerate()
            .map(|(i, &id)| {
                let kern = ids
                    .get(i + 1)
                    .and_then(|&next| self.kerning.get(&(id, next)))
                    .copied()
                    .unwrap_or(0.0);
                PositionedGlyph {
                    id: GlyphId(id),
                    offset: crate::V2::zero(),
                    advance: (self.raw_glyph(id).advance - kern) * scale,
                }
            })
            .collect()
    }

    pub(crate) fn glyph(&self, id: GlyphId, scale: f32) -> Glyph {
        let raw = self.raw_glyph(id.0);
        Glyph {
            id,
            advance: raw.advance * scale,
            contours: raw
                .contours
                .iter()
                .map(|contour| contour.scale(scale))
                .collect(),
        }
    }

    fn raw_glyph(&self, id: u32) -> &StrokeGlyph {
        self.glyphs.get(id as usize).unwrap_or(&self.glyphs[0])
    }
}

impl StrokeGlyph {
    fn from_attributes(attributes: &Attributes, default_advance: f32) -> Self {
        Self {
            advance: attr_f32(attributes, "horiz-adv-x").unwrap_or(default_advance),
            contours: attr(attributes, "d").map_or_else(Vec::new, parse_svg_path_data),
        }
    }
}

fn attr<'a>(attributes: &'a Attributes, key: &str) -> Option<&'a str> {
    attributes.get(key).map(|value| &**value)
}

fn attr_f32(attributes: &Attributes, key: &str) -> Option<f32> {
    attr(attributes, key)?.trim().parse().ok()
}

fn single_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    chars.next().filter(|_| chars.next().is_none())
}

/// Comma separated chars; `U+XXXX-YYYY` ranges are skipped.
fn kern_chars(value: Option<&str>) -> Vec<char> {
    value.map_or_else(Vec::new, |value| {
        decode_xml_entities(value)
            .split(',')
            .filter_map(single_char)
            .collect()
    })
}

fn kern_names(value: Option<&str>) -> Vec<String> {
    value.map_or_else(Vec::new, |value| {
        value
            .split(',')
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect()
    })
}

/// Decodes the five XML entities and numeric references; unknown ones are kept verbatim.
pub(crate) fn decode_xml_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let decoded = after
            .find(';')
            .and_then(|end| decode_entity(&after[1..end]).map(|ch| (ch, end + 1)));
        match decoded {
            Some((ch, consumed)) => {
                out.push(ch);
                rest = &after[consumed..];
            }
            None => {
                out.push('&');
                rest = &after[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let digits = entity.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}
