use plottery_lib::V2;

pub const MAX_ZOOM: f32 = 16.0;
/// Zoom level a click on the fitted preview jumps to.
pub const CLICK_ZOOM: f32 = 2.0;

/// Zoom of the editor preview. `pan` is the paper centre's offset from the stage centre in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewZoom {
    pub scale: f32,
    pub pan: V2,
}

impl Default for PreviewZoom {
    fn default() -> Self {
        Self {
            scale: 1.0,
            pan: V2::zero(),
        }
    }
}

/// Unscaled paper size and visible stage size, in px.
#[derive(Debug, Clone, Copy)]
pub struct ZoomViewport {
    pub paper: V2,
    pub stage: V2,
}

impl PreviewZoom {
    pub fn is_fit(&self) -> bool {
        self.scale <= 1.0
    }

    /// Scales by `factor` while keeping the point `at` (relative to the stage centre) in place.
    pub fn zoom_at(self, factor: f32, at: V2, view: ZoomViewport) -> Self {
        let scale = (self.scale * factor).clamp(1.0, MAX_ZOOM);
        let pan = at - (at - self.pan) * (scale / self.scale);
        Self { scale, pan }.clamped(view)
    }

    pub fn pan_by(self, delta: V2, view: ZoomViewport) -> Self {
        Self {
            pan: self.pan + delta,
            ..self
        }
        .clamped(view)
    }

    /// Maps a point relative to the stage centre to px on the unscaled paper.
    pub fn stage_to_paper(&self, at: V2, paper: V2) -> V2 {
        (at - self.pan) / self.scale + paper / 2.0
    }

    /// Keeps the paper edges from moving inside the stage once the paper is larger than it.
    fn clamped(self, view: ZoomViewport) -> Self {
        let limit = ((view.paper * self.scale - view.stage) / 2.0).max(V2::zero());
        Self {
            pan: self.pan.max(-limit).min(limit),
            ..self
        }
    }
}
