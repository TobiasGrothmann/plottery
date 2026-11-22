/// Units per em when a font doesn't say.
pub(crate) const DEFAULT_UNITS_PER_EM: f32 = 1000.0;
/// Ascender and descender heights in em when a font doesn't say.
pub(crate) const FALLBACK_ASCENDER_EM: f32 = 0.8;
pub(crate) const FALLBACK_DESCENDER_EM: f32 = 0.2;

/// Font-wide vertical metrics in cm at an em size; `descender` is negative (y-up), baseline is 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontMetrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_gap: f32,
    pub cap_height: Option<f32>,
    pub x_height: Option<f32>,
}

impl FontMetrics {
    pub(crate) fn scale(&self, factor: f32) -> Self {
        Self {
            ascender: self.ascender * factor,
            descender: self.descender * factor,
            line_gap: self.line_gap * factor,
            cap_height: self.cap_height.map(|v| v * factor),
            x_height: self.x_height.map(|v| v * factor),
        }
    }
}
