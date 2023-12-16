use geometry_predicates::orient2d;

use crate::V2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line<'a> {
    pub from: &'a V2,
    pub to: &'a V2,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PointLineRelation {
    OnLine,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineIntersection {
    NoIntersection,
    Intersection(V2),
}

impl Line<'_> {
    pub fn point_relation(&self, point: &V2) -> PointLineRelation {
        let orientation = orient2d(
            [self.from.x as f64, self.from.y as f64],
            [self.to.x as f64, self.to.y as f64],
            [point.x as f64, point.y as f64],
        );
        if orientation > 0.0 {
            return PointLineRelation::Left;
        } else if orientation < 0.0 {
            return PointLineRelation::Right;
        }
        PointLineRelation::OnLine
    }

    pub fn intersection(&self, other: &Line) -> LineIntersection {
        if self.from == other.from || self.from == other.to {
            return LineIntersection::Intersection(*self.from);
        } else if self.to == other.from || self.to == other.to {
            return LineIntersection::Intersection(*self.to);
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

        if x < f32::min(x1, x2)
            || x > f32::max(x1, x2)
            || x < f32::min(x3, x4)
            || x > f32::max(x3, x4)
        {
            return LineIntersection::NoIntersection;
        }

        LineIntersection::Intersection(V2::new(x, y))
    }
}
