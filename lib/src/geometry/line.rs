use geometry_predicates::orient2d;
use itertools::Itertools;

use crate::{Angle, LARGE_EPSILON, V2};

/// handles both line segments between `from` and `to`, and infinite lines defined by these points
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub from: V2,
    pub to: V2,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PointLineRelation {
    /// point is on the line
    OnLine,
    /// point is on the left hand side of the line. Left and Right are given in relation to the direction from `from` to `to`.
    Left,
    /// point is on the right hand side of the line. Left and Right are given in relation to the direction from `from` to `to`.
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineIntersection {
    NoIntersection,
    Intersection(V2),
}

impl Line {
    pub fn new(from: V2, to: V2) -> Self {
        Self { from, to }
    }

    /// vector from `from` to `to`
    pub fn vector(&self) -> V2 {
        self.to - self.from
    }
    /// midpoint between `from` and `to`
    pub fn mid(&self) -> V2 {
        (self.from + self.to) * 0.5
    }
    /// angle of the [`V2`] from `from` to `to`
    pub fn angle(&self) -> Angle {
        (self.to - self.from).angle()
    }
    /// new Line moved to its right by `distance`. Right is given by the orthogonal vector to the right of `self.angle()`, which
    /// is the vector from `from` to `to` rotated by 90 degrees.
    pub fn offset_right(&self, distance: f32) -> Self {
        let normal_scaled = V2::polar(self.angle().normal_right(), distance);
        Line::new(self.from + normal_scaled, self.to + normal_scaled)
    }

    /// project a point onto this the infinite line defined by `self.from` and `self.to`.
    pub fn project(&self, point: &V2) -> V2 {
        self.from + (point - self.from).project_onto(&self.vector())
    }

    /// get the relation of `point` to this line.
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

    /// check for an intersection of `self` with `other`. The lines are interpreted as segments between `from` and `to`.
    /// If you are looking for the intersection of the infinite lines defined by the segments, see [`Self::intersection_as_inf_lines`].
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

    /// check for an intersection of `self` with `other`. Both lines are interpreted as infinite lines defined by the points `from` and `to`.
    /// see [`Self::intersection`] for intersection of line segments.
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

    /// get closest location to `point` on the infinite line defined by `self` to `point`
    /// see [`Self::closest_point`] for closest point on the line segment.
    pub fn closest_point_on_infinite_line(&self, point: &V2) -> V2 {
        if self.from == self.to {
            return self.from;
        }
        let l = self.to - self.from;
        let t = (point - self.from).dot(&l) / l.len_squared();
        self.from + l * t
    }

    /// get closest location to `point` on the line segment defined by `self` to `point`
    /// see [`Self::closest_point_on_infinite_line`] for closest point on the infinite line.
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

    /// get all intersections of `self` with `line_segments` ordered by distance to `self.from`
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
