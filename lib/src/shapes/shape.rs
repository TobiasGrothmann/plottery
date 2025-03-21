pub use crate::shapes::circle::Circle;
pub use crate::shapes::path::Path;
pub use crate::shapes::rect::Rect;

use crate::{
    geometry::TransformMatrix,
    traits::{ClosestPoint, Normalize, Scale, Scale2D, Transform, Translate},
    Angle, BoundingBox, Mirror, Plottable, Rotate, Rotate90, SampleSettings, V2,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Shape {
    Circle(Circle),
    Rect(Rect),
    Path(Path),
}

impl Plottable for Shape {
    fn get_points(&self, sample_settings: SampleSettings) -> Vec<V2> {
        match self {
            Shape::Circle(c) => c.get_points(sample_settings),
            Shape::Rect(r) => r.get_points(sample_settings),
            Shape::Path(p) => p.get_points(sample_settings),
        }
    }
    fn get_points_from(
        &self,
        current_drawing_head_pos: V2,
        sample_settings: SampleSettings,
    ) -> Vec<V2> {
        match self {
            Shape::Circle(c) => c.get_points_from(current_drawing_head_pos, sample_settings),
            Shape::Rect(r) => r.get_points_from(current_drawing_head_pos, sample_settings),
            Shape::Path(p) => p.get_points_from(current_drawing_head_pos, sample_settings),
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

    fn contains_point(&self, point: V2) -> bool {
        match self {
            Shape::Circle(c) => c.contains_point(point),
            Shape::Rect(r) => r.contains_point(point),
            Shape::Path(p) => p.contains_point(point),
        }
    }

    fn reduce_points(&self, aggression_factor: f32) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.reduce_points(aggression_factor)),
            Shape::Rect(r) => Shape::Rect(r.reduce_points(aggression_factor)),
            Shape::Path(p) => Shape::Path(p.reduce_points(aggression_factor)),
        }
    }
}

impl Rotate for Shape {
    fn rotate(&self, angle: Angle) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate(angle)),
            Shape::Rect(r) => {
                Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()]).rotate(angle)
            }
            Shape::Path(p) => Shape::Path(p.rotate(angle)),
        }
    }
    fn rotate_mut(&mut self, angle: Angle) {
        match self {
            Shape::Circle(c) => c.rotate_mut(angle),
            Shape::Rect(r) => {
                *self =
                    Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()]).rotate(angle)
            }
            Shape::Path(p) => p.rotate_mut(angle),
        }
    }

    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_around(pivot, angle)),
            Shape::Rect(r) => Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()])
                .rotate_around(pivot, angle),
            Shape::Path(p) => Shape::Path(p.rotate_around(pivot, angle)),
        }
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        match self {
            Shape::Circle(c) => c.rotate_around_mut(pivot, angle),
            Shape::Rect(r) => {
                *self = Path::new_shape_from(vec![r.bl(), r.tl(), r.tr(), r.br(), r.bl()])
                    .rotate_around(pivot, angle)
            }
            Shape::Path(p) => p.rotate_around_mut(pivot, angle),
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
    fn rotate_90_mut(&mut self) {
        match self {
            Shape::Circle(c) => c.rotate_90_mut(),
            Shape::Rect(r) => r.rotate_90_mut(),
            Shape::Path(p) => p.rotate_90_mut(),
        }
    }

    fn rotate_180(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_180()),
            Shape::Rect(r) => Shape::Rect(r.rotate_180()),
            Shape::Path(p) => Shape::Path(p.rotate_180()),
        }
    }
    fn rotate_180_mut(&mut self) {
        match self {
            Shape::Circle(c) => c.rotate_180_mut(),
            Shape::Rect(r) => r.rotate_180_mut(),
            Shape::Path(p) => p.rotate_180_mut(),
        }
    }

    fn rotate_270(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_270()),
            Shape::Rect(r) => Shape::Rect(r.rotate_270()),
            Shape::Path(p) => Shape::Path(p.rotate_270()),
        }
    }
    fn rotate_270_mut(&mut self) {
        match self {
            Shape::Circle(c) => c.rotate_270_mut(),
            Shape::Rect(r) => r.rotate_270_mut(),
            Shape::Path(p) => p.rotate_270_mut(),
        }
    }

    fn rotate_90_around(&self, pivot: V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_90_around(pivot)),
            Shape::Rect(r) => Shape::Rect(r.rotate_90_around(pivot)),
            Shape::Path(p) => Shape::Path(p.rotate_90_around(pivot)),
        }
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        match self {
            Shape::Circle(c) => c.rotate_90_around_mut(pivot),
            Shape::Rect(r) => r.rotate_90_around_mut(pivot),
            Shape::Path(p) => p.rotate_90_around_mut(pivot),
        }
    }

    fn rotate_180_around(&self, pivot: V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_180_around(pivot)),
            Shape::Rect(r) => Shape::Rect(r.rotate_180_around(pivot)),
            Shape::Path(p) => Shape::Path(p.rotate_180_around(pivot)),
        }
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        match self {
            Shape::Circle(c) => c.rotate_180_around_mut(pivot),
            Shape::Rect(r) => r.rotate_180_around_mut(pivot),
            Shape::Path(p) => p.rotate_180_around_mut(pivot),
        }
    }

    fn rotate_270_around(&self, pivot: V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_270_around(pivot)),
            Shape::Rect(r) => Shape::Rect(r.rotate_270_around(pivot)),
            Shape::Path(p) => Shape::Path(p.rotate_270_around(pivot)),
        }
    }

    fn rotate_270_around_mut(&mut self, pivot: V2) {
        match self {
            Shape::Circle(c) => c.rotate_270_around_mut(pivot),
            Shape::Rect(r) => r.rotate_270_around_mut(pivot),
            Shape::Path(p) => p.rotate_270_around_mut(pivot),
        }
    }
}

impl Translate for Shape {
    fn translate(&self, dist: V2) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.translate(dist)),
            Shape::Rect(r) => Shape::Rect(r.translate(dist)),
            Shape::Path(p) => Shape::Path(p.translate(dist)),
        }
    }

    fn translate_mut(&mut self, dist: V2) {
        match self {
            Shape::Circle(c) => c.translate_mut(dist),
            Shape::Rect(r) => r.translate_mut(dist),
            Shape::Path(p) => p.translate_mut(dist),
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

    fn scale_mut(&mut self, scale: f32) {
        match self {
            Shape::Circle(c) => c.scale_mut(scale),
            Shape::Rect(r) => r.scale_mut(scale),
            Shape::Path(p) => p.scale_mut(scale),
        }
    }
}

impl Scale2D for Shape {
    fn scale_2d(&self, factor: V2) -> Self {
        match self {
            Shape::Circle(c) => {
                let mut points = c.get_points(SampleSettings::default());
                for p in &mut points {
                    *p *= factor;
                }
                Path::new_shape_from(points)
            }
            Shape::Rect(r) => Shape::Rect(r.scale_2d(factor)),
            Shape::Path(p) => Shape::Path(p.scale_2d(factor)),
        }
    }

    fn scale_2d_mut(&mut self, factor: V2) {
        match self {
            Shape::Circle(c) => {
                let mut points = c.get_points(SampleSettings::default());
                for p in &mut points {
                    *p *= factor;
                }
                *self = Path::new_shape_from(points);
            }
            Shape::Rect(r) => r.scale_2d_mut(factor),
            Shape::Path(p) => p.scale_2d_mut(factor),
        }
    }
}

impl Normalize for Shape {}

impl Mirror for Shape {
    fn mirror_x(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.mirror_x()),
            Shape::Rect(r) => Shape::Rect(r.mirror_x()),
            Shape::Path(p) => Shape::Path(p.mirror_x()),
        }
    }

    fn mirror_x_mut(&mut self) {
        match self {
            Shape::Circle(c) => c.mirror_x_mut(),
            Shape::Rect(r) => r.mirror_x_mut(),
            Shape::Path(p) => p.mirror_x_mut(),
        }
    }

    fn mirror_y(&self) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.mirror_y()),
            Shape::Rect(r) => Shape::Rect(r.mirror_y()),
            Shape::Path(p) => Shape::Path(p.mirror_y()),
        }
    }

    fn mirror_y_mut(&mut self) {
        match self {
            Shape::Circle(c) => c.mirror_y_mut(),
            Shape::Rect(r) => r.mirror_y_mut(),
            Shape::Path(p) => p.mirror_y_mut(),
        }
    }
}

impl BoundingBox for Shape {
    fn bounding_box(&self) -> Option<Rect> {
        match self {
            Shape::Circle(c) => c.bounding_box(),
            Shape::Rect(r) => r.bounding_box(),
            Shape::Path(p) => p.bounding_box(),
        }
    }
}

impl Transform for Shape {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        match self {
            Shape::Circle(c) => Shape::Path(
                c.get_points(SampleSettings::default())
                    .iter()
                    .map(|p| matrix.mul_vector(*p))
                    .collect::<Path>(),
            ),
            Shape::Rect(r) => {
                let points = vec![
                    matrix.mul_vector(r.bl()),
                    matrix.mul_vector(r.tl()),
                    matrix.mul_vector(r.tr()),
                    matrix.mul_vector(r.br()),
                    matrix.mul_vector(r.bl()),
                ];
                Path::new_shape_from(points)
            }
            Shape::Path(p) => Shape::Path(p.transform(matrix)),
        }
    }

    fn transform_mut(&mut self, matrix: &TransformMatrix) {
        match self {
            Shape::Circle(c) => {
                let points = c
                    .get_points(SampleSettings::default())
                    .iter()
                    .map(|p| matrix.mul_vector(*p))
                    .collect::<Path>();
                *self = Shape::Path(points);
            }
            Shape::Rect(r) => {
                let points = vec![
                    matrix.mul_vector(r.bl()),
                    matrix.mul_vector(r.tl()),
                    matrix.mul_vector(r.tr()),
                    matrix.mul_vector(r.br()),
                    matrix.mul_vector(r.bl()),
                ];
                *self = Path::new_shape_from(points);
            }
            Shape::Path(p) => p.transform_mut(matrix),
        }
    }
}

impl ClosestPoint for Shape {
    fn closest_point(&self, sample_settings: SampleSettings, point: V2) -> Option<V2> {
        match self {
            Shape::Circle(c) => c.closest_point(sample_settings, point),
            Shape::Rect(r) => r.closest_point(sample_settings, point),
            Shape::Path(p) => p.closest_point(sample_settings, point),
        }
    }
}

impl From<Vec<V2>> for Shape {
    fn from(points: Vec<V2>) -> Self {
        Path::new_shape_from(points)
    }
}

impl From<Circle> for Shape {
    fn from(circle: Circle) -> Self {
        Shape::Circle(circle)
    }
}

impl From<Rect> for Shape {
    fn from(rect: Rect) -> Self {
        Shape::Rect(rect)
    }
}

impl From<Path> for Shape {
    fn from(path: Path) -> Self {
        Shape::Path(path)
    }
}
