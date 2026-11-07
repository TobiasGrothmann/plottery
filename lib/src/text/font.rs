use std::{fmt, path::Path, sync::Arc};

use anyhow::{bail, Context, Result};

use super::{
    outline_font::OutlineFont, stroke_font::StrokeFont, system::find_system_font, FontMetrics,
    Glyph, GlyphId, PositionedGlyph,
};

/// A loaded font: outlines (TTF, OTF, TTC) or a single-stroke SVG 1.1 font (Hershey, EMS). Cheap to clone.
#[derive(Clone)]
pub struct Font {
    data: Arc<FontData>,
}

enum FontData {
    Outline(OutlineFont),
    Stroke(StrokeFont),
}

/// Requested style for system font lookup; `weight` is CSS-like, 400 regular, 700 bold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontStyle {
    pub weight: f32,
    pub italic: bool,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            weight: 400.0,
            italic: false,
        }
    }
}

impl Font {
    /// Loads by extension (`.svg` is a stroke font), otherwise by magic bytes.
    pub fn load(path: impl AsRef<Path>) -> Result<Font> {
        let path = path.as_ref();
        let bytes =
            std::fs::read(path).with_context(|| format!("reading font {}", path.display()))?;
        let is_svg = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));
        if is_svg {
            Self::from_svg_str(std::str::from_utf8(&bytes)?)
        } else {
            Self::from_bytes(bytes)
        }
        .with_context(|| format!("loading font {}", path.display()))
    }

    /// Detects an SVG font by a leading `<`, otherwise expects TTF/OTF/TTC data (first face of a collection).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Font> {
        let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        if text.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'<') {
            return Self::from_svg_str(std::str::from_utf8(text)?);
        }
        let is_outline = bytes.get(..4).is_some_and(|magic| {
            magic == [0, 1, 0, 0] || magic == *b"OTTO" || magic == *b"true" || magic == *b"ttcf"
        });
        if !is_outline {
            bail!("unrecognised font data, expected TTF, OTF, TTC or an SVG font");
        }
        Ok(Self::new(FontData::Outline(OutlineFont::from_bytes(
            bytes, 0,
        )?)))
    }

    pub fn from_svg_str(svg: &str) -> Result<Font> {
        Ok(Self::new(FontData::Stroke(StrokeFont::parse(svg)?)))
    }

    /// Regular upright face of a system font family, e.g. `"Helvetica"`.
    pub fn system(family: &str) -> Result<Font> {
        Self::system_with(family, FontStyle::default())
    }

    pub fn system_with(family: &str, style: FontStyle) -> Result<Font> {
        Ok(Self::new(FontData::Outline(OutlineFont::from_font(
            find_system_font(family, style)?,
        ))))
    }

    fn new(data: FontData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    pub fn family_name(&self) -> Option<&str> {
        match &*self.data {
            FontData::Outline(font) => font.family(),
            FontData::Stroke(font) => font.family(),
        }
    }

    pub fn is_stroke(&self) -> bool {
        matches!(&*self.data, FontData::Stroke(_))
    }

    /// Vertical metrics in cm at em size `size`.
    pub fn metrics(&self, size: f32) -> FontMetrics {
        let (metrics, units_per_em) = match &*self.data {
            FontData::Outline(font) => (font.metrics(), font.units_per_em()),
            FontData::Stroke(font) => (font.metrics(), font.units_per_em()),
        };
        metrics.scale(size / units_per_em)
    }

    pub fn has_glyph(&self, ch: char) -> bool {
        match &*self.data {
            FontData::Outline(font) => font.has_glyph(ch),
            FontData::Stroke(font) => font.has_glyph(ch),
        }
    }

    /// Glyph of `ch` in cm at em size `size`; unmapped chars yield the font's missing glyph.
    pub fn glyph(&self, ch: char, size: f32) -> Glyph {
        let id = match &*self.data {
            FontData::Outline(font) => font.glyph_id(ch),
            FontData::Stroke(font) => font.glyph_id(ch),
        };
        self.glyph_by_id(id, size)
    }

    pub(crate) fn glyph_by_id(&self, id: GlyphId, size: f32) -> Glyph {
        match &*self.data {
            FontData::Outline(font) => font.glyph(id, size / font.units_per_em()),
            FontData::Stroke(font) => font.glyph(id, size / font.units_per_em()),
        }
    }

    pub(crate) fn shape(&self, text: &str, size: f32) -> Result<Vec<PositionedGlyph>> {
        match &*self.data {
            FontData::Outline(font) => font.shape(text, size / font.units_per_em()),
            FontData::Stroke(font) => Ok(font.shape(text, size / font.units_per_em())),
        }
    }
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_stroke() {
            "stroke"
        } else {
            "outline"
        };
        write!(f, "Font({kind}, {:?})", self.family_name())
    }
}
