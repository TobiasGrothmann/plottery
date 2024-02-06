pub mod plottable;
mod plottable_test;
pub mod rotate;
pub mod rotate90;
pub mod scale;
pub mod scale2d;
pub mod translate;

pub use plottable::{Masked, Plottable, SampleSettings};
pub use rotate::Rotate;
pub use rotate90::Rotate90;
pub use scale::Scale;
pub use scale2d::Scale2D;
pub use translate::Translate;
