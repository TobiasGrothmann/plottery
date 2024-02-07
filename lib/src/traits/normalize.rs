use anyhow::Result;

use super::Scale;
use crate::{BoundingBox, Rect, Translate};

pub enum Alignment {
    Min,
    Center,
    Max,
}

pub trait Normalize: Scale + Translate + BoundingBox
where
    Self: Sized,
{
    fn normalize(&self, target: &Rect, alignment: Alignment) -> Result<Self> {
        let bounds = self
            .bounding_box()
            .ok_or(anyhow::Error::msg("No bounding box"))?;

        let ar_target = target.aspect_ratio();
        let ar_shape = bounds.aspect_ratio();

        let normalized = if ar_shape < ar_target {
            // shape is taller than target
            let scale: f32 = target.height() / bounds.height();
            let scaled = self.scale(scale);
            let scaled_bounds = bounds.scale(scale);

            let new_size = bounds.size() * scale;
            let bl_offset = target.bl() - scaled_bounds.bl();
            let offset = match alignment {
                Alignment::Min => bl_offset,
                Alignment::Center => bl_offset + (target.size() - new_size) * 0.5,
                Alignment::Max => bl_offset + (target.size() - new_size),
            };
            scaled.translate(&offset)
        } else {
            // shape is wider than target
            let scale: f32 = target.width() / bounds.width();
            let scaled = self.scale(scale);
            let scaled_bounds = bounds.scale(scale);

            let new_size = bounds.size() * scale;
            let bl_offset = target.bl() - scaled_bounds.bl();
            let offset = match alignment {
                Alignment::Min => bl_offset,
                Alignment::Center => bl_offset + (target.size() - new_size) * 0.5,
                Alignment::Max => bl_offset + (target.size() - new_size),
            };
            scaled.translate(&offset)
        };

        Ok(normalized)
    }
}
