use anyhow::{anyhow, Result};
use skrifa::{font::name::NameId, DrawMemory, PathSink, ScalerFont};

use super::{FontMetrics, Glyph, GlyphId, PositionedGlyph};
use crate::{shapes::spline::sink::SplineSink, Scale, V2};

/// TrueType / OpenType font read by skrifa and shaped by harfrust; font units, y-up.
pub(crate) struct OutlineFont {
    font: skrifa::Font,
    scaler: ScalerFont,
    units_per_em: f32,
    metrics: FontMetrics,
    family: Option<String>,
}

impl OutlineFont {
    pub(crate) fn from_bytes(bytes: Vec<u8>, index: u32) -> Result<Self> {
        let font = skrifa::Font::new(bytes, index)
            .ok_or_else(|| anyhow!("not a readable TrueType/OpenType font (index {index})"))?;
        Ok(Self::from_font(font))
    }

    pub(crate) fn from_font(font: skrifa::Font) -> Self {
        let raw = font.metrics();
        let units_per_em = match raw.units_per_em {
            0 => 1000.0,
            upem => f32::from(upem),
        };
        let line = raw.h_line();
        let metrics = FontMetrics {
            ascender: line.map_or(units_per_em * 0.8, |line| line.ascender.to_f32()),
            descender: line.map_or(-units_per_em * 0.2, |line| line.descender.to_f32()),
            line_gap: line.map_or(0.0, |line| line.line_gap.to_f32()),
            cap_height: raw.cap_height.map(|v| v.to_f32()),
            x_height: raw.x_height.map(|v| v.to_f32()),
        };
        let family = name(&font, NameId::FAMILY_NAME);
        let scaler = ScalerFont::unhinted(font.clone(), None);
        Self {
            font,
            scaler,
            units_per_em,
            metrics,
            family,
        }
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
        self.font.charmap().map_unicode(ch).is_some()
    }

    pub(crate) fn glyph_id(&self, ch: char) -> GlyphId {
        GlyphId(
            self.font
                .charmap()
                .map_unicode(ch)
                .map_or(0, |id| id.to_u32()),
        )
    }

    /// Unicode shaping with the font's default features (kerning, ligatures).
    pub(crate) fn shape(&self, text: &str, scale: f32) -> Result<Vec<PositionedGlyph>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let shaper = harfrust::ShaperFont::new(&self.font);
        let mut buffer = harfrust::Buffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        harfrust::shape(&shaper, &mut buffer, harfrust::ShapeOptions::default())
            .map_err(|error| anyhow!("shaping {text:?} failed: {error}"))?;
        Ok(buffer
            .glyph_infos()
            .iter()
            .zip(buffer.glyph_positions())
            .map(|(info, position)| PositionedGlyph {
                id: GlyphId(info.glyph_id),
                offset: V2::new(position.x_offset as f32, position.y_offset as f32) * scale,
                advance: position.x_advance as f32 * scale,
            })
            .collect())
    }

    pub(crate) fn glyph(&self, id: GlyphId, scale: f32) -> Glyph {
        let raw_id = skrifa::GlyphId::new(id.0);
        let mut pen = SplinePen(SplineSink::new());
        let (contours, drawn_advance) = match self
            .scaler
            .outline(raw_id)
            .map(|outline| outline.draw(DrawMemory::Scratch, &mut pen))
        {
            Some(Ok(drawn)) => (pen.0.finish(), drawn.h_advance),
            _ => (Vec::new(), None),
        };
        let advance = drawn_advance.unwrap_or_else(|| self.font.glyph_metrics().h_advance(raw_id));
        Glyph {
            id,
            advance: advance * scale,
            contours: contours
                .iter()
                .map(|contour| contour.scale(scale))
                .collect(),
        }
    }
}

/// English name if there is one, otherwise the first.
pub(crate) fn name(font: &skrifa::Font, id: NameId) -> Option<String> {
    font.names()
        .get(id)?
        .english_or_first()
        .map(|localized| localized.name().to_string())
}

struct SplinePen(SplineSink);

impl PathSink for SplinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(V2::new(x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(V2::new(x, y));
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.0.quad_to(V2::new(cx0, cy0), V2::new(x, y));
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0
            .cubic_to(V2::new(cx0, cy0), V2::new(cx1, cy1), V2::new(x, y));
    }
    fn close(&mut self) {
        self.0.close();
    }
}
