use crate::shapes::circle::Circle;
use crate::shapes::path::Path;
use crate::shapes::rect::Rect;

use super::containment::contains_shape_by_variant;
use crate::{
    geometry::TransformMatrix,
    traits::{ClosestPoint, Scale, Scale2D, Transform, Translate},
    Angle, BoundingBox, Containment, Line, Mirror, Plottable, Rotate, Rotate90, SampleSettings,
    LENGTH_TOLERANCE, V2,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Shape {
    Circle(Circle),
    Rect(Rect),
    Path(Path),
}

/// The same expression for every variant, with the inner shape bound to `$s`.
macro_rules! dispatch {
    ($shape:expr, $s:ident => $e:expr) => {
        match $shape {
            Shape::Circle($s) => $e,
            Shape::Rect($s) => $e,
            Shape::Path($s) => $e,
        }
    };
}

/// Like [`dispatch!`], wrapping each result in the variant it came from.
macro_rules! map_variant {
    ($shape:expr, $s:ident => $e:expr) => {
        match $shape {
            Shape::Circle($s) => Shape::Circle($e),
            Shape::Rect($s) => Shape::Rect($e),
            Shape::Path($s) => Shape::Path($e),
        }
    };
}

impl Shape {
    /// True if the shape draws nothing, e.g. a [`Path`] with fewer than two distinct points.
    pub fn is_degenerate(&self) -> bool {
        match self {
            Shape::Circle(c) => c.radius <= LENGTH_TOLERANCE,
            Shape::Rect(r) => r.width() <= LENGTH_TOLERANCE && r.height() <= LENGTH_TOLERANCE,
            Shape::Path(p) => p.is_degenerate(),
        }
    }

    pub fn intersects_circle(&self, other: &Circle) -> bool {
        dispatch!(self, s => s.intersects_circle(other))
    }
    pub fn intersects_rect(&self, other: &Rect) -> bool {
        dispatch!(self, s => s.intersects_rect(other))
    }
    pub fn intersects_path(&self, other: &Path) -> bool {
        dispatch!(self, s => s.intersects_path(other))
    }
    pub fn intersects(&self, other: &Shape) -> bool {
        match other {
            Shape::Circle(c) => self.intersects_circle(c),
            Shape::Rect(r) => self.intersects_rect(r),
            Shape::Path(p) => self.intersects_path(p),
        }
    }

    pub fn contains_circle(&self, other: &Circle) -> Containment {
        dispatch!(self, s => s.contains_circle(other))
    }
    pub fn contains_rect(&self, other: &Rect) -> Containment {
        dispatch!(self, s => s.contains_rect(other))
    }
    pub fn contains_path(&self, other: &Path) -> Containment {
        dispatch!(self, s => s.contains_path(other))
    }

    contains_shape_by_variant!();
}

impl Plottable for Shape {
    fn points(&self, sample_settings: SampleSettings) -> Vec<V2> {
        dispatch!(self, s => s.points(sample_settings))
    }
    fn points_from(
        &self,
        current_drawing_head_pos: V2,
        sample_settings: SampleSettings,
    ) -> Vec<V2> {
        dispatch!(self, s => s.points_from(current_drawing_head_pos, sample_settings))
    }
    fn length(&self) -> f32 {
        dispatch!(self, s => s.length())
    }
    fn is_closed(&self) -> bool {
        dispatch!(self, s => s.is_closed())
    }
    fn contains_point(&self, point: V2) -> bool {
        dispatch!(self, s => s.contains_point(point))
    }
    fn reduce_points(&self, aggression_factor: f32) -> Self {
        map_variant!(self, s => s.reduce_points(aggression_factor))
    }
}

/// A [`Rect`] is always upright, so rotating one turns it into a [`Path`].
impl Rotate for Shape {
    fn rotate(&self, angle: Angle) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate(angle)),
            Shape::Rect(r) => Shape::Path(r.to_path(SampleSettings::default()).rotate(angle)),
            Shape::Path(p) => Shape::Path(p.rotate(angle)),
        }
    }
    fn rotate_mut(&mut self, angle: Angle) {
        match self {
            Shape::Circle(c) => c.rotate_mut(angle),
            Shape::Rect(_) => *self = self.rotate(angle),
            Shape::Path(p) => p.rotate_mut(angle),
        }
    }

    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        match self {
            Shape::Circle(c) => Shape::Circle(c.rotate_around(pivot, angle)),
            Shape::Rect(r) => Shape::Path(
                r.to_path(SampleSettings::default())
                    .rotate_around(pivot, angle),
            ),
            Shape::Path(p) => Shape::Path(p.rotate_around(pivot, angle)),
        }
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        match self {
            Shape::Circle(c) => c.rotate_around_mut(pivot, angle),
            Shape::Rect(_) => *self = self.rotate_around(pivot, angle),
            Shape::Path(p) => p.rotate_around_mut(pivot, angle),
        }
    }
}

impl Rotate90 for Shape {
    fn rotate_90(&self) -> Self {
        map_variant!(self, s => s.rotate_90())
    }
    fn rotate_90_mut(&mut self) {
        dispatch!(self, s => s.rotate_90_mut())
    }
    fn rotate_180(&self) -> Self {
        map_variant!(self, s => s.rotate_180())
    }
    fn rotate_180_mut(&mut self) {
        dispatch!(self, s => s.rotate_180_mut())
    }
    fn rotate_270(&self) -> Self {
        map_variant!(self, s => s.rotate_270())
    }
    fn rotate_270_mut(&mut self) {
        dispatch!(self, s => s.rotate_270_mut())
    }
    fn rotate_90_around(&self, pivot: V2) -> Self {
        map_variant!(self, s => s.rotate_90_around(pivot))
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        dispatch!(self, s => s.rotate_90_around_mut(pivot))
    }
    fn rotate_180_around(&self, pivot: V2) -> Self {
        map_variant!(self, s => s.rotate_180_around(pivot))
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        dispatch!(self, s => s.rotate_180_around_mut(pivot))
    }
    fn rotate_270_around(&self, pivot: V2) -> Self {
        map_variant!(self, s => s.rotate_270_around(pivot))
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        dispatch!(self, s => s.rotate_270_around_mut(pivot))
    }
}

impl Translate for Shape {
    fn translate(&self, dist: V2) -> Self {
        map_variant!(self, s => s.translate(dist))
    }
    fn translate_mut(&mut self, dist: V2) {
        dispatch!(self, s => s.translate_mut(dist))
    }
}

impl Scale for Shape {
    fn scale(&self, scale: f32) -> Self {
        map_variant!(self, s => s.scale(scale))
    }
    fn scale_mut(&mut self, scale: f32) {
        dispatch!(self, s => s.scale_mut(scale))
    }
}

/// A [`Circle`] has one radius, so scaling one unevenly turns it into a sampled [`Path`].
impl Scale2D for Shape {
    fn scale_2d(&self, factor: V2) -> Self {
        match self {
            Shape::Circle(c) => {
                let mut points = c.points(SampleSettings::default());
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
            Shape::Circle(_) => *self = self.scale_2d(factor),
            Shape::Rect(r) => r.scale_2d_mut(factor),
            Shape::Path(p) => p.scale_2d_mut(factor),
        }
    }
}

impl Mirror for Shape {
    fn mirror_x(&self) -> Self {
        map_variant!(self, s => s.mirror_x())
    }
    fn mirror_x_mut(&mut self) {
        dispatch!(self, s => s.mirror_x_mut())
    }
    fn mirror_y(&self) -> Self {
        map_variant!(self, s => s.mirror_y())
    }
    fn mirror_y_mut(&mut self) {
        dispatch!(self, s => s.mirror_y_mut())
    }
}

impl BoundingBox for Shape {
    fn bounding_box(&self) -> Option<Rect> {
        dispatch!(self, s => s.bounding_box())
    }
}

/// A matrix can rotate and shear, so [`Circle`] and [`Rect`] turn into [`Path`]s.
impl Transform for Shape {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        match self {
            Shape::Circle(c) => Shape::Path(
                c.points(SampleSettings::default())
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
            Shape::Circle(_) | Shape::Rect(_) => *self = self.transform(matrix),
            Shape::Path(p) => p.transform_mut(matrix),
        }
    }
}

impl ClosestPoint for Shape {
    fn closest_point(&self, sample_settings: SampleSettings, point: V2) -> Option<V2> {
        dispatch!(self, s => s.closest_point(sample_settings, point))
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

impl From<&Circle> for Shape {
    fn from(circle: &Circle) -> Self {
        Shape::Circle(*circle)
    }
}

impl From<&Rect> for Shape {
    fn from(rect: &Rect) -> Self {
        Shape::Rect(*rect)
    }
}

impl From<&Path> for Shape {
    fn from(path: &Path) -> Self {
        Shape::Path(path.clone())
    }
}

impl From<&Shape> for Shape {
    fn from(shape: &Shape) -> Self {
        shape.clone()
    }
}

impl From<Line> for Shape {
    fn from(line: Line) -> Self {
        Shape::Path(line.into())
    }
}
