use geometry_predicates::orient2d;
use itertools::Itertools;

use crate::{Angle, LARGE_EPSILON, V2};

/// A line defined by two points. Can represent both finite line segments and infinite lines.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub from: V2,
    pub to: V2,
}

/// Describes the position of a point relative to a line.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PointLineRelation {
    /// The point lies on the line.
    OnLine,
    /// The point is on the left side of the line (relative to direction from `from` to `to`).
    Left,
    /// The point is on the right side of the line (relative to direction from `from` to `to`).
    Right,
}

/// Represents the result of a line intersection check.
#[derive(Debug, Clone, PartialEq)]
pub enum LineIntersection {
    /// The lines do not intersect.
    NoIntersection,
    /// The lines intersect at the given point.
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

    /// Returns the midpoint of the line.
    pub fn mid(&self) -> V2 {
        (self.from + self.to) * 0.5
    }

    /// Returns the angle of the line (from `from` to `to`).
    pub fn angle(&self) -> Angle {
        (self.to - self.from).angle()
    }

    /// Returns a new line offset to the right by the given distance.
    ///
    /// "Right" is determined by the orthogonal vector to the right of the line's direction.
    pub fn offset_right(&self, distance: f32) -> Self {
        let normal_scaled = V2::polar(self.angle().normal_right(), distance);
        Line::new(self.from + normal_scaled, self.to + normal_scaled)
    }

    /// Projects a point onto the infinite line defined by this line segment.
    pub fn project(&self, point: &V2) -> V2 {
        self.from + (point - self.from).project_onto(&self.vector())
    }

    /// Determines the position of a point relative to this line.
    pub fn point_relation(&self, point: &V2) -> PointLineRelation {
        let orientation = orient2d(
            [self.from.x as f64, self.from.y as f64],
            [self.to.x as f64, self.to.y as f64],
            [point.x as f64, point.y as f64],
        );
        if orientation >= LARGE_EPSILON as f64 {
            return PointLineRelation::Left;
        } else if orientation <= -LARGE_EPSILON as f64 {
            return PointLineRelation::Right;
        }
        PointLineRelation::OnLine
    }

    /// Checks for an intersection between this line segment and another.
    ///
    /// For infinite lines, see [`Self::intersection_as_inf_lines`].
    pub fn intersection(&self, other: &Line) -> LineIntersection {
        if self.from == other.from || self.from == other.to {
            return LineIntersection::Intersection(self.from);
        } else if self.to == other.from || self.to == other.to {
            return LineIntersection::Intersection(self.to);
        }

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
            return LineIntersection::NoIntersection; // Lines are parallel
        }

        let num_x = (x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4);
        let num_y = (x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4);

        let x = num_x / denom;
        let y = num_y / denom;

        // Check if the intersection point lies on both line segments
        // by verifying both x and y coordinates are within bounds
        if x < f32::min(x1, x2) - LARGE_EPSILON
            || x > f32::max(x1, x2) + LARGE_EPSILON
            || x < f32::min(x3, x4) - LARGE_EPSILON
            || x > f32::max(x3, x4) + LARGE_EPSILON
            || y < f32::min(y1, y2) - LARGE_EPSILON
            || y > f32::max(y1, y2) + LARGE_EPSILON
            || y < f32::min(y3, y4) - LARGE_EPSILON
            || y > f32::max(y3, y4) + LARGE_EPSILON
        {
            return LineIntersection::NoIntersection;
        }

        LineIntersection::Intersection(V2::new(x, y))
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
    pub fn closest_point_on_infinite_line(&self, point: &V2) -> V2 {
        if self.from == self.to {
            return self.from;
        }
        let l = self.to - self.from;
        let t = (point - self.from).dot(&l) / l.len_squared();
        self.from + l * t
    }

    /// Returns the closest point on the line segment to the given point.
    ///
    /// For infinite lines, see [`Self::closest_point_on_infinite_line`].
    pub fn closest_point(&self, point: &V2) -> V2 {
        if self.from == self.to {
            return self.from;
        }
        let l = self.to - self.from;
        let t = (point - self.from).dot(&l) / l.len_squared();
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
            .iter()
            .map(|segment| self.intersection(segment))
            .filter_map(|intersection| match intersection {
                LineIntersection::Intersection(point) => Some(point),
                _ => None,
            })
            .sorted_by_cached_key(|point| point.dist_squared(&self.from).to_bits())
            .collect()
    }
}
