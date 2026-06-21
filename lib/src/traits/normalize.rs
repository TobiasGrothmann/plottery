use anyhow::Result;

use super::Scale;
use crate::{BoundingBox, Rect, Translate, LARGE_EPSILON};

pub enum Alignment {
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

pub enum NormalizeMode {
    /// Scale to fit inside target while preserving aspect ratio.
    Inside,
    /// Scale so target is fully covered, around target while preserving aspect ratio.
    Around,
}

pub trait Normalize: Scale + Translate + BoundingBox + Clone
where
    Self: Sized,
{
    /// Backwards-compatible alias for `normalize_inside`.
    fn normalize(&self, target: &Rect, alignment: Alignment) -> Result<Self> {
        self.normalize_inside(target, alignment)
    }

    /// Backwards-compatible alias for `normalize_inside_mut`.
    fn normalize_mut(&mut self, target: &Rect, alignment: Alignment) -> Result<()> {
        self.normalize_inside_mut(target, alignment)
    }

    fn normalize_inside(&self, target: &Rect, alignment: Alignment) -> Result<Self> {
        self.normalize_with_mode(target, alignment, NormalizeMode::Inside)
    }

    fn normalize_inside_mut(&mut self, target: &Rect, alignment: Alignment) -> Result<()> {
        self.normalize_mut_with_mode(target, alignment, NormalizeMode::Inside)
    }

    fn normalize_around(&self, target: &Rect, alignment: Alignment) -> Result<Self> {
        self.normalize_with_mode(target, alignment, NormalizeMode::Around)
    }

    fn normalize_around_mut(&mut self, target: &Rect, alignment: Alignment) -> Result<()> {
        self.normalize_mut_with_mode(target, alignment, NormalizeMode::Around)
    }

    fn normalize_with_mode(
        &self,
        target: &Rect,
        alignment: Alignment,
        mode: NormalizeMode,
    ) -> Result<Self> {
        let mut copy = self.clone();
        copy.normalize_mut_with_mode(target, alignment, mode)?;
        Ok(copy)
    }

    fn normalize_mut_with_mode(
        &mut self,
        target: &Rect,
        alignment: Alignment,
        mode: NormalizeMode,
    ) -> Result<()> {
        let bounds = self
            .bounding_box()
            .ok_or(anyhow::Error::msg("No bounding box"))?;

        if bounds.width() <= LARGE_EPSILON || bounds.height() <= LARGE_EPSILON {
            return Err(anyhow::Error::msg(
                "Cannot normalize shape with zero width or height",
            ));
        }

        let ar_target = target.aspect_ratio();
        let ar_shape = bounds.aspect_ratio();

        match mode {
            NormalizeMode::Inside => {
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
                    self.translate_mut(offset)
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
                    self.translate_mut(offset)
                }
            }
            NormalizeMode::Around => {
                if ar_shape < ar_target {
                    // shape is taller than target, scale to target width
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
                    self.translate_mut(offset)
                } else {
                    // shape is wider than target, scale to target height
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
                    self.translate_mut(offset)
                }
            }
        }

        Ok(())
    }
}
