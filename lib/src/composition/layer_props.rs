use serde::{Deserialize, Serialize};

use crate::LARGE_EPSILON;

use super::ColorRgb;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LayerProps {
    Inherit,
    Custom(LayerPropsSettings),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LayerPropsSettings {
    pub color: ColorRgb,
    pub pen_width_cm: f32,
}

impl Default for LayerPropsSettings {
    fn default() -> Self {
        Self {
            color: ColorRgb::black(),
            pen_width_cm: 0.05,
        }
    }
}

impl PartialEq for LayerPropsSettings {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color && (self.pen_width_cm - other.pen_width_cm).abs() < LARGE_EPSILON
    }
}
