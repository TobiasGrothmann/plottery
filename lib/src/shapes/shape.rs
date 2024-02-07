pub use crate::shapes::circle::Circle;
pub use crate::shapes::path::Path;
pub use crate::shapes::rect::Rect;

use crate::{
    traits::{Normalize, Scale, Scale2D, Translate},
    BoundingBox, Plottable, Rotate, Rotate90, SampleSettings, V2,
};
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
    fn rotate_inplace(&mut self, angle: &crate::Angle) {
        match self {
            Shape::Circle(c) => c.rotate_inplace(angle),
            Shape::Rect(r) => {
                *self =
                    Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()]).rotate(angle)
            }
            Shape::Path(p) => p.rotate_inplace(angle),
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
    fn rotate_around_inplace(&mut self, pivot: &V2, angle: &crate::Angle) {
        match self {
            Shape::Circle(c) => c.rotate_around_inplace(pivot, angle),
            Shape::Rect(r) => {
                *self = Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()])
                    .rotate_around(pivot, angle)
            }
            Shape::Path(p) => p.rotate_around_inplace(pivot, angle),
        }
    }
}

impl Rotate90 for Shape {
    fn rotate_90(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_90()),
            Shape::Rect(r) => Shape::Rect(r.rotate_90()),
            Shape::Path(p) => Shape::Path(p.rotate_90()),
        }
    }
    fn rotate_90_inplace(&mut self) {
        match self {
            Shape::Circle(c) => c.rotate_90_inplace(),
            Shape::Rect(r) => r.rotate_90_inplace(),
            Shape::Path(p) => p.rotate_90_inplace(),
        }
    }

    fn rotate_180(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_180()),
            Shape::Rect(r) => Shape::Rect(r.rotate_180()),
            Shape::Path(p) => Shape::Path(p.rotate_180()),
        }
    }
    fn rotate_180_inplace(&mut self) {
        match self {
            Shape::Circle(c) => c.rotate_180_inplace(),
            Shape::Rect(r) => r.rotate_180_inplace(),
            Shape::Path(p) => p.rotate_180_inplace(),
        }
    }

    fn rotate_270(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_270()),
            Shape::Rect(r) => Shape::Rect(r.rotate_270()),
            Shape::Path(p) => Shape::Path(p.rotate_270()),
        }
    }
    fn rotate_270_inplace(&mut self) {
        match self {
            Shape::Circle(c) => c.rotate_270_inplace(),
            Shape::Rect(r) => r.rotate_270_inplace(),
            Shape::Path(p) => p.rotate_270_inplace(),
        }
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_90_around(pivot)),
            Shape::Rect(r) => Shape::Rect(r.rotate_90_around(pivot)),
            Shape::Path(p) => Shape::Path(p.rotate_90_around(pivot)),
        }
    }
    fn rotate_90_around_inplace(&mut self, pivot: &V2) {
        match self {
            Shape::Circle(c) => c.rotate_90_around_inplace(pivot),
            Shape::Rect(r) => r.rotate_90_around_inplace(pivot),
            Shape::Path(p) => p.rotate_90_around_inplace(pivot),
        }
    }

    fn rotate_180_around(&self, pivot: &V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_180_around(pivot)),
            Shape::Rect(r) => Shape::Rect(r.rotate_180_around(pivot)),
            Shape::Path(p) => Shape::Path(p.rotate_180_around(pivot)),
        }
    }
    fn rotate_180_around_inplace(&mut self, pivot: &V2) {
        match self {
            Shape::Circle(c) => c.rotate_180_around_inplace(pivot),
            Shape::Rect(r) => r.rotate_180_around_inplace(pivot),
            Shape::Path(p) => p.rotate_180_around_inplace(pivot),
        }
    }

    fn rotate_270_around(&self, pivot: &V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_270_around(pivot)),
            Shape::Rect(r) => Shape::Rect(r.rotate_270_around(pivot)),
            Shape::Path(p) => Shape::Path(p.rotate_270_around(pivot)),
        }
    }

    fn rotate_270_around_inplace(&mut self, pivot: &V2) {
        match self {
            Shape::Circle(c) => c.rotate_270_around_inplace(pivot),
            Shape::Rect(r) => r.rotate_270_around_inplace(pivot),
            Shape::Path(p) => p.rotate_270_around_inplace(pivot),
        }
    }
}

impl Translate for Shape {
    fn translate(&self, dist: &V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.translate(dist)),
            Shape::Rect(r) => Shape::Rect(r.translate(dist)),
            Shape::Path(p) => Shape::Path(p.translate(dist)),
        }
    }

    fn translate_inplace(&mut self, dist: &V2) {
        match self {
            Shape::Circle(c) => c.translate_inplace(dist),
            Shape::Rect(r) => r.translate_inplace(dist),
            Shape::Path(p) => p.translate_inplace(dist),
        }
    }
}

impl Scale for Shape {
    fn scale(&self, scale: f32) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.scale(scale)),
            Shape::Rect(r) => Shape::Rect(r.scale(scale)),
            Shape::Path(p) => Shape::Path(p.scale(scale)),
        }
    }

    fn scale_inplace(&mut self, scale: f32) {
        match self {
            Shape::Circle(c) => c.scale_inplace(scale),
            Shape::Rect(r) => r.scale_inplace(scale),
            Shape::Path(p) => p.scale_inplace(scale),
        }
    }
}

impl Scale2D for Shape {
    fn scale_2d(&self, factor: &V2) -> Self {
        match self {
            Shape::Circle(c) => {
                let mut points = c.get_points(&SampleSettings::default());
                for p in &mut points {
                    *p *= factor;
                }
                Path::new_shape_from(points)
            }
            Shape::Rect(r) => Shape::Rect(r.scale_2d(factor)),
            Shape::Path(p) => Shape::Path(p.scale_2d(factor)),
        }
    }

    fn scale_2d_inplace(&mut self, factor: &V2) {
        match self {
            Shape::Circle(c) => {
                let mut points = c.get_points(&SampleSettings::default());
                for p in &mut points {
                    *p *= factor;
                }
                *self = Path::new_shape_from(points);
            }
            Shape::Rect(r) => r.scale_2d_inplace(factor),
            Shape::Path(p) => p.scale_2d_inplace(factor),
        }
    }
}

impl Normalize for Shape {}

impl BoundingBox for Shape {
    fn bounding_box(&self) -> Option<Rect> {
        match self {
            Shape::Circle(c) => c.bounding_box(),
            Shape::Rect(r) => r.bounding_box(),
            Shape::Path(p) => p.bounding_box(),
        }
    }
}
