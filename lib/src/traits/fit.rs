use anyhow::{bail, Context, Result};

use super::Scale;
use crate::{BoundingBox, Rect, Translate, LARGE_EPSILON, V2};

/// Which edge of the target the fitted shape touches when the aspect ratios differ.
pub enum Alignment {
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

/// Uniform scaling and translation into a target rect, keeping the aspect ratio.
pub trait Fit: Scale + Translate + BoundingBox + Clone
where
    Self: Sized,
{
    /// Scales to fit inside `target`.
    fn fit_inside(&self, target: Rect, alignment: Alignment) -> Result<Self> {
        let mut copy = self.clone();
        copy.fit_inside_mut(target, alignment)?;
        Ok(copy)
    }

    fn fit_inside_mut(&mut self, target: Rect, alignment: Alignment) -> Result<()> {
        fit_mut(self, target, alignment, false)
    }

    /// Scales to cover `target` completely.
    fn fit_around(&self, target: Rect, alignment: Alignment) -> Result<Self> {
        let mut copy = self.clone();
        copy.fit_around_mut(target, alignment)?;
        Ok(copy)
    }

    fn fit_around_mut(&mut self, target: Rect, alignment: Alignment) -> Result<()> {
        fit_mut(self, target, alignment, true)
    }
}

fn fit_mut<T: Fit>(shape: &mut T, target: Rect, alignment: Alignment, around: bool) -> Result<()> {
    let bounds = shape.bounding_box().context("no bounding box")?;
    if bounds.width() <= LARGE_EPSILON || bounds.height() <= LARGE_EPSILON {
        bail!("cannot fit a shape with zero width or height");
    }
    let ratio = target.size() / bounds.size();
    let scale = if around {
        ratio.x.max(ratio.y)
    } else {
        ratio.x.min(ratio.y)
    };
    shape.scale_mut(scale);
    let scaled = bounds.scale(scale);
    // where the slack on the axis that does not touch goes
    let slack = target.size() - scaled.size();
    let share = match alignment {
        Alignment::Left => V2::new(0.0, 0.5),
        Alignment::Right => V2::new(1.0, 0.5),
        Alignment::Bottom => V2::new(0.5, 0.0),
        Alignment::Top => V2::new(0.5, 1.0),
        Alignment::Center => V2::new(0.5, 0.5),
    };
    shape.translate_mut(target.bl() - scaled.bl() + slack * share);
    Ok(())
}
