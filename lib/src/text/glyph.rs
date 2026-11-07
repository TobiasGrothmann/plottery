use crate::{Angle, BoundingBox, Path, Rect, Rotate, SampleSettings, Scale, Spline, Translate, V2};

/// Index of a glyph within its font; 0 is the missing glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphId(pub u32);

/// One glyph in cm with the baseline at the origin.
#[derive(Clone, Debug)]
pub struct Glyph {
    pub id: GlyphId,
    /// Horizontal distance to the next glyph's origin.
    pub advance: f32,
    pub contours: Vec<Spline>,
}

impl Glyph {
    pub fn to_paths(&self, sample_settings: SampleSettings) -> Vec<Path> {
        self.contours
            .iter()
            .map(|contour| contour.to_path(sample_settings))
            .collect()
    }
}

impl Translate for Glyph {
    fn translate(&self, dist: V2) -> Self {
        Self {
            id: self.id,
            advance: self.advance,
            contours: self
                .contours
                .iter()
                .map(|contour| contour.translate(dist))
                .collect(),
        }
    }
}

/// Rotates the contours only, the advance stays horizontal.
impl Rotate for Glyph {
    fn rotate(&self, angle: Angle) -> Self {
        Self {
            id: self.id,
            advance: self.advance,
            contours: self
                .contours
                .iter()
                .map(|contour| contour.rotate(angle))
                .collect(),
        }
    }
}

/// Scales the advance too.
impl Scale for Glyph {
    fn scale(&self, factor: f32) -> Self {
        Self {
            id: self.id,
            advance: self.advance * factor,
            contours: self
                .contours
                .iter()
                .map(|contour| contour.scale(factor))
                .collect(),
        }
    }
}

/// Hull of all control points, see [`Spline`]'s bounding box.
impl BoundingBox for Glyph {
    fn bounding_box(&self) -> Option<Rect> {
        self.contours
            .iter()
            .filter_map(Spline::bounding_box)
            .reduce(|a, b| Rect::new(a.bl().min(b.bl()), a.tr().max(b.tr())))
    }
}

/// A shaped glyph: `offset` is relative to the pen, `advance` moves the pen, both in cm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PositionedGlyph {
    pub id: GlyphId,
    pub offset: V2,
    pub advance: f32,
}
