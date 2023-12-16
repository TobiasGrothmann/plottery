pub mod composition;
pub mod geometry;
pub mod project_definition;
pub mod shapes;
pub mod traits;

pub use project_definition::Project;
pub use project_definition::ProjectConfig;

pub use geometry::angle::Angle;
pub use geometry::angle_operators::*;
pub use geometry::line::Line;
pub use geometry::line::LineIntersection;
pub use geometry::line::PointLineRelation;
pub use geometry::vec2::V2;
pub use geometry::vec2_operators::*;

pub use shapes::circle::Circle;
pub use shapes::path::Path;
pub use shapes::rect::Rect;

pub use traits::Masked;
pub use traits::Rotate;
pub use traits::Rotate90;
pub use traits::SampleSettings;
pub use traits::Shape;

pub use composition::Layer;
