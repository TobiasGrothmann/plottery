pub mod composition;
pub mod generate;
pub mod geometry;
pub mod maths;
pub mod shapes;
pub mod traits;

pub use geometry::angle::Angle;
pub use geometry::line::Line;
pub use geometry::line::LineIntersection;
pub use geometry::line::PointLineRelation;
pub use geometry::v2::V2;

pub use shapes::circle::Circle;
pub use shapes::path::Path;
pub use shapes::rect::Rect;
pub use shapes::shape::Shape;

pub use traits::BoundingBox;
pub use traits::Masked;
pub use traits::Plottable;
pub use traits::Rotate;
pub use traits::Rotate90;
pub use traits::SampleSettings;
pub use traits::Scale;
pub use traits::Scale2D;
pub use traits::Translate;
pub use traits::{normalize::Alignment, Normalize};

pub use composition::Frame;
pub use composition::Layer;

pub use generate::func_2d::marching_squares::MarchingSquares;

pub use maths::*;
