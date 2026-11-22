use std::{fmt, path::Path, sync::Arc};

use super::{
    outline_font::OutlineFont,
    stroke_font::StrokeFont,
    system::{find_system_font, FontStyle},
    FontMetrics, Glyph, GlyphId, PositionedGlyph,
};
use anyhow::{bail, Context, Result};

/// A loaded font: outlines (TTF, OTF, TTC) or a single-stroke SVG 1.1 font (Hershey, EMS). Cheap to clone.
#[derive(Clone)]
pub struct Font {
    data: Arc<FontData>,
}

enum FontData {
    Outline(OutlineFont),
    Stroke(StrokeFont),
}

/// The same expression for either backend, with the backend font bound to `$font`.
macro_rules! backend {
    ($data:expr, $font:ident => $e:expr) => {
        match $data {
            FontData::Outline($font) => $e,
            FontData::Stroke($font) => $e,
        }
    };
}

impl Font {
    /// Loads by extension (`.svg` is a stroke font), otherwise by magic bytes.
    pub fn load(path: impl AsRef<Path>) -> Result<Font> {
        Self::load_with(path, false)
    }

    /// Loads a single-line TTF/OTF whose strokes are stored as open contours, see [`Font::from_bytes_single_line`].
    pub fn load_single_line(path: impl AsRef<Path>) -> Result<Font> {
        Self::load_with(path, true)
    }

    fn load_with(path: impl AsRef<Path>, open_contours: bool) -> Result<Font> {
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
            Self::from_bytes_with(bytes, open_contours)
        }
        .with_context(|| format!("loading font {}", path.display()))
    }

    /// Detects an SVG font by a leading `<`, otherwise expects TTF/OTF/TTC data (first face of a collection).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Font> {
        Self::from_bytes_with(bytes, false)
    }

    /// TrueType has no open contours, so single-line fonts (Relief SingleLine, Hershey TTFs) store
    /// each stroke as a closed one; this leaves them open instead of drawing back to the start.
    /// Fonts with an OpenType `SVG ` table (Mistral SingleLine) need no flag.
    pub fn from_bytes_single_line(bytes: Vec<u8>) -> Result<Font> {
        Self::from_bytes_with(bytes, true)
    }

    fn from_bytes_with(bytes: Vec<u8>, open_contours: bool) -> Result<Font> {
        let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        if text.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'<') {
            return Self::from_svg_str(std::str::from_utf8(text)?);
        }
        let is_outline = bytes.get(..4).is_some_and(|magic| {
            magic == [0, 1, 0, 0] || magic == *b"OTTO" || magic == *b"true" || magic == *b"ttcf"
        });
        if !is_outline {
            bail!("unrecognized font data, expected TTF, OTF, TTC or an SVG font");
        }
        Ok(Self::new(FontData::Outline(OutlineFont::from_bytes(
            bytes,
            0,
            open_contours,
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
            false,
        ))))
    }

    fn new(data: FontData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    pub fn family_name(&self) -> Option<&str> {
        backend!(&*self.data, font => font.family())
    }

    pub fn is_stroke(&self) -> bool {
        matches!(&*self.data, FontData::Stroke(_))
    }

    /// True for outline fonts loaded with [`Font::load_single_line`].
    pub fn is_single_line(&self) -> bool {
        matches!(&*self.data, FontData::Outline(font) if font.open_contours())
    }

    /// Factor from font units to cm at em size `size`.
    fn scale(&self, size: f32) -> f32 {
        size / backend!(&*self.data, font => font.units_per_em())
    }

    /// Vertical metrics in cm at em size `size`.
    pub fn metrics(&self, size: f32) -> FontMetrics {
        backend!(&*self.data, font => font.metrics()).scale(self.scale(size))
    }

    pub fn has_glyph(&self, ch: char) -> bool {
        backend!(&*self.data, font => font.has_glyph(ch))
    }

    /// Glyph of `ch` in cm at em size `size`; unmapped chars yield the font's missing glyph.
    pub fn glyph(&self, ch: char, size: f32) -> Glyph {
        self.glyph_by_id(backend!(&*self.data, font => font.glyph_id(ch)), size)
    }

    pub(crate) fn glyph_by_id(&self, id: GlyphId, size: f32) -> Glyph {
        let scale = self.scale(size);
        backend!(&*self.data, font => font.glyph(id, scale))
    }

    pub(crate) fn shape(&self, text: &str, size: f32) -> Result<Vec<PositionedGlyph>> {
        let scale = self.scale(size);
        match &*self.data {
            FontData::Outline(font) => font.shape(text, scale),
            FontData::Stroke(font) => Ok(font.shape(text, scale)),
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
