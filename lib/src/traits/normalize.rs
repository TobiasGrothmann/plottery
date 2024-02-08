use anyhow::Result;

use super::Scale;
use crate::{BoundingBox, Rect, Translate};

pub enum Alignment {
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

pub trait Normalize: Scale + Translate + BoundingBox + Clone
where
    Self: Sized,
{
    fn normalize(&self, target: &Rect, alignment: Alignment) -> Result<Self> {
        let mut copy = self.clone();
        copy.normalize_mut(target, alignment)?;
        Ok(copy)
    }

    fn normalize_mut(&mut self, target: &Rect, alignment: Alignment) -> Result<()> {
        let bounds = self
            .bounding_box()
            .ok_or(anyhow::Error::msg("No bounding box"))?;

        let ar_target = target.aspect_ratio();
        let ar_shape = bounds.aspect_ratio();

        if ar_shape < ar_target {
            // shape is taller than target
            let scale: f32 = target.height() / bounds.height();
            self.scale_mut(scale);
            let scaled_bounds = bounds.scale(scale);

            let new_size = bounds.size() * scale;
            let bl_offset = target.bl() - scaled_bounds.bl();
            let size_difference = target.size() - new_size;
            let offset = match alignment {
                Alignment::Left => bl_offset,
                Alignment::Center | Alignment::Bottom | Alignment::Top => {
                    bl_offset + size_difference * 0.5
                }
                Alignment::Right => bl_offset + size_difference,
            };
            self.translate_mut(&offset)
        } else {
            // shape is wider than target
            let scale: f32 = target.width() / bounds.width();
            self.scale_mut(scale);
            let scaled_bounds = bounds.scale(scale);

            let new_size = bounds.size() * scale;
            let bl_offset = target.bl() - scaled_bounds.bl();
            let size_difference = target.size() - new_size;
            let offset = match alignment {
                Alignment::Bottom => bl_offset,
                Alignment::Center | Alignment::Left | Alignment::Right => {
                    bl_offset + size_difference * 0.5
                }
                Alignment::Top => bl_offset + size_difference,
            };
            self.translate_mut(&offset)
        };

        Ok(())
    }
}
