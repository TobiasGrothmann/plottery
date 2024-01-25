pub use crate::shapes::circle::Circle;
pub use crate::shapes::path::Path;
pub use crate::shapes::rect::Rect;

use crate::{Plottable, Rotate, SampleSettings, V2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Shape {
    Circle(Circle),
    Rect(Rect),
    Path(Path),
}

impl Plottable for Shape {
    fn get_points(&self, sample_settings: &SampleSettings) -> Vec<V2> {
        match self {
            Shape::Circle(c) => c.get_points(sample_settings),
            Shape::Rect(r) => r.get_points(sample_settings),
            Shape::Path(p) => p.get_points(sample_settings),
        }
    }

    fn length(&self) -> f32 {
        match self {
            Shape::Circle(c) => c.length(),
            Shape::Rect(r) => r.length(),
            Shape::Path(p) => p.length(),
        }
    }

    fn is_closed(&self) -> bool {
        match self {
            Shape::Circle(c) => c.is_closed(),
            Shape::Rect(r) => r.is_closed(),
            Shape::Path(p) => p.is_closed(),
        }
    }
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.clone()),
            Shape::Rect(r) => Shape::Rect(r.clone()),
            Shape::Path(p) => Shape::Path(p.clone()),
        }
    }
}

impl Rotate for Shape {
    fn rotate(&self, angle: &crate::Angle) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate(angle)),
            Shape::Rect(r) => {
                Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()]).rotate(angle)
            }
            Shape::Path(p) => Shape::Path(p.rotate(angle)),
        }
    }

    fn rotate_around(&self, pivot: &V2, angle: &crate::Angle) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_around(pivot, angle)),
            Shape::Rect(r) => Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()])
                .rotate_around(pivot, angle),
            Shape::Path(p) => Shape::Path(p.rotate_around(pivot, angle)),
        }
    }
}
