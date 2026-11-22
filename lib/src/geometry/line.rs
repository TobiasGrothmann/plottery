use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    geometry::TransformMatrix,
    traits::{ClosestPoint, Transform},
    Angle, BoundingBox, Mirror, Rect, Rotate, Rotate90, SampleSettings, Scale, Translate,
    AREA_TOLERANCE, LENGTH_TOLERANCE, V2,
};

/// A line defined by two points. Can represent both finite line segments and infinite lines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub from: V2,
    pub to: V2,
}

/// Describes the position of a point relative to a line.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PointLineRelation {
    /// The point lies on the infinite line.
    ///
    /// Note: this does **not** imply the point lies on the finite segment between `from` and `to`.
    /// Use [`Line::contains_point_on_segment`] when segment bounds matter.
    OnLine,
    /// The point is on the left side of the line (relative to direction from `from` to `to`).
    Left,
    /// The point is on the right side of the line (relative to direction from `from` to `to`).
    Right,
}

/// Represents the result of a line intersection check.
#[derive(Debug, Clone, PartialEq)]
pub enum LineIntersection {
    NoIntersection,
    Intersection(V2),
}

impl Line {
    pub fn new(from: V2, to: V2) -> Self {
        Self { from, to }
    }

    /// Returns the vector from `from` to `to`.
    pub fn vector(&self) -> V2 {
        self.to - self.from
    }

    pub fn mid(&self) -> V2 {
        (self.from + self.to) * 0.5
    }

    pub fn length(&self) -> f32 {
        self.vector().length()
    }

    /// Point at fraction `t` along the segment, `0.0` is `from` and `1.0` is `to`.
    pub fn point_at(&self, t: f32) -> V2 {
        self.from.lerp(self.to, t)
    }

    /// Unit vector orthogonal to the right of the line's direction (clockwise from it).
    pub fn normal_right(&self) -> V2 {
        let dir = self.vector().normalize();
        V2::new(dir.y, -dir.x)
    }

    /// Unit vector orthogonal to the left of the line's direction.
    pub fn normal_left(&self) -> V2 {
        -self.normal_right()
    }

    fn map_points<F: Fn(V2) -> V2>(&self, f: F) -> Self {
        Self::new(f(self.from), f(self.to))
    }

    /// Returns the angle of the line (from `from` to `to`).
    pub fn angle(&self) -> Angle {
        (self.to - self.from).angle()
    }

    /// Returns a new line offset to the right of its direction by the given distance.
    #[must_use]
    pub fn offset_right(&self, distance: f32) -> Self {
        self.translate(self.normal_right() * distance)
    }

    /// Returns a new line offset to the left by the given distance.
    #[must_use]
    pub fn offset_left(&self, distance: f32) -> Self {
        self.offset_right(-distance)
    }

    /// Projects a point onto the infinite line defined by this line segment.
    pub fn project(&self, point: V2) -> V2 {
        self.from + (point - self.from).project_onto(self.vector())
    }

    /// Determines the position of a point relative to this infinite line.
    ///
    /// If you need to check membership on the finite segment, use [`Self::contains_point_on_segment`].
    pub fn point_relation(&self, point: V2) -> PointLineRelation {
        // twice the signed area of the triangle, positive if `point` is to the left
        let f64s = |p: V2| (p.x as f64, p.y as f64);
        let (from, to, point) = (f64s(self.from), f64s(self.to), f64s(point));
        let orientation =
            (from.0 - point.0) * (to.1 - point.1) - (from.1 - point.1) * (to.0 - point.0);
        if orientation >= AREA_TOLERANCE as f64 {
            return PointLineRelation::Left;
        } else if orientation <= -AREA_TOLERANCE as f64 {
            return PointLineRelation::Right;
        }
        PointLineRelation::OnLine
    }

    /// Returns whether a point lies on this finite line segment (including endpoints).
    pub fn contains_point_on_segment(&self, point: V2) -> bool {
        self.closest_point(point).dist_squared(point) <= AREA_TOLERANCE
    }

    /// Checks for an intersection between this line segment and another.
    ///
    /// For infinite lines, see [`Self::intersection_as_inf_lines`].
    pub fn intersection(&self, other: Line) -> LineIntersection {
        if self.from == other.from || self.from == other.to {
            return LineIntersection::Intersection(self.from);
        } else if self.to == other.from || self.to == other.to {
            return LineIntersection::Intersection(self.to);
        }

        let LineIntersection::Intersection(point) = self.intersection_as_inf_lines(&other) else {
            return LineIntersection::NoIntersection;
        };
        // the point has to lie on both segments
        let within = |line: &Line| {
            point.x >= line.from.x.min(line.to.x) - LENGTH_TOLERANCE
                && point.x <= line.from.x.max(line.to.x) + LENGTH_TOLERANCE
                && point.y >= line.from.y.min(line.to.y) - LENGTH_TOLERANCE
                && point.y <= line.from.y.max(line.to.y) + LENGTH_TOLERANCE
        };
        if within(self) && within(&other) {
            LineIntersection::Intersection(point)
        } else {
            LineIntersection::NoIntersection
        }
    }

    /// Checks for an intersection between this line and another, treating both as infinite lines.
    ///
    /// For line segments, see [`Self::intersection`].
    pub fn intersection_as_inf_lines(&self, other: &Line) -> LineIntersection {
        let x1 = self.from.x;
        let y1 = self.from.y;
        let x2 = self.to.x;
        let y2 = self.to.y;

        let x3 = other.from.x;
        let y3 = other.from.y;
        let x4 = other.to.x;
        let y4 = other.to.y;

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

        if denom == 0.0 {
            // lines are parallel (or colinear)
            return LineIntersection::NoIntersection;
        }

        let num_x = (x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4);
        let num_y = (x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4);

        let x = num_x / denom;
        let y = num_y / denom;

        LineIntersection::Intersection(V2::new(x, y))
    }

    /// Returns the closest point on the infinite line to the given point.
    ///
    /// For line segments, see [`Self::closest_point`].
    pub fn closest_point_on_infinite_line(&self, point: V2) -> V2 {
        if self.from == self.to {
            return self.from;
        }
        let l = self.to - self.from;
        let t = (point - self.from).dot(l) / l.length_squared();
        self.from + l * t
    }

    /// Returns the closest point on the line segment to the given point.
    ///
    /// For infinite lines, see [`Self::closest_point_on_infinite_line`].
    pub fn closest_point(&self, point: V2) -> V2 {
        if self.from == self.to {
            return self.from;
        }
        let l = self.to - self.from;
        let t = (point - self.from).dot(l) / l.length_squared();
        if t < 0.0 {
            return self.from;
        }
        if t > 1.0 {
            return self.to;
        }
        self.from + l * t
    }

    /// Returns all intersections of this line with the given line segments, sorted by distance from `self.from`.
    pub fn intersect_multiple_sorted_by_dist(&self, line_segments: &[Line]) -> Vec<V2> {
        line_segments
            .par_iter()
            .map(|segment| self.intersection(*segment))
            .filter_map(|intersection| match intersection {
                LineIntersection::Intersection(point) => Some(point),
                _ => None,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .sorted_by_cached_key(|point| point.dist_squared(self.from).to_bits())
            .collect()
    }
}

impl Translate for Line {
    fn translate(&self, dist: V2) -> Self {
        self.map_points(|p| p + dist)
    }
}

impl Rotate for Line {
    fn rotate(&self, angle: Angle) -> Self {
        self.map_points(|p| p.rotate(angle))
    }
    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        self.map_points(|p| p.rotate_around(pivot, angle))
    }
}

impl Rotate90 for Line {
    fn rotate_90(&self) -> Self {
        self.map_points(|p| p.rotate_90())
    }
    fn rotate_180(&self) -> Self {
        self.map_points(|p| p.rotate_180())
    }
    fn rotate_270(&self) -> Self {
        self.map_points(|p| p.rotate_270())
    }
    fn rotate_90_around(&self, pivot: V2) -> Self {
        self.map_points(|p| p.rotate_90_around(pivot))
    }
    fn rotate_180_around(&self, pivot: V2) -> Self {
        self.map_points(|p| p.rotate_180_around(pivot))
    }
    fn rotate_270_around(&self, pivot: V2) -> Self {
        self.map_points(|p| p.rotate_270_around(pivot))
    }
}

impl Scale for Line {
    fn scale(&self, factor: f32) -> Self {
        self.map_points(|p| p * factor)
    }
}

impl Mirror for Line {
    fn mirror_x(&self) -> Self {
        self.map_points(|p| p.mirror_x())
    }
    fn mirror_y(&self) -> Self {
        self.map_points(|p| p.mirror_y())
    }
}

impl Transform for Line {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        self.map_points(|p| p.transform(matrix))
    }
}

impl BoundingBox for Line {
    fn bounding_box(&self) -> Option<Rect> {
        Some(Rect::new(self.from, self.to))
    }
}

/// Note: the inherent [`Line::closest_point`] takes only the point; use `ClosestPoint::closest_point(&line, ..)` for the trait form.
impl ClosestPoint for Line {
    fn closest_point(&self, _: SampleSettings, point: V2) -> Option<V2> {
        Some(Line::closest_point(self, point))
    }
}
