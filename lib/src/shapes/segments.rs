use itertools::Itertools;

use crate::{Circle, Line, LineIntersection, PointLineRelation, Rect, V2};

/// Consecutive points as segments.
pub(super) fn segments(points: &[V2]) -> impl Iterator<Item = Line> + Clone + '_ {
    points
        .iter()
        .tuple_windows()
        .map(|(from, to)| Line::new(*from, *to))
}

/// True if two segments cross or touch, collinear overlaps included.
pub(super) fn segments_intersect_or_touch(a: Line, b: Line) -> bool {
    if matches!(a.intersection(b), LineIntersection::Intersection(_)) {
        return true;
    }
    a.point_relation(b.from) == PointLineRelation::OnLine
        && a.point_relation(b.to) == PointLineRelation::OnLine
        && Rect::new(a.from, a.to).overlaps(&Rect::new(b.from, b.to))
}

/// True if any segment of `a` crosses or touches any segment of `b`.
pub(super) fn any_segments_touch(
    a: impl IntoIterator<Item = Line>,
    b: impl Iterator<Item = Line> + Clone,
) -> bool {
    a.into_iter()
        .any(|a| b.clone().any(|b| segments_intersect_or_touch(a, b)))
}

/// True if `segment` reaches the boundary of `circle`: it comes within the radius and is not fully inside.
pub(super) fn segment_touches_circle(segment: Line, circle: &Circle) -> bool {
    let radius_squared = circle.radius.powi(2);
    segment
        .closest_point(circle.center)
        .dist_squared(circle.center)
        <= radius_squared
        && (segment.from.dist_squared(circle.center) >= radius_squared
            || segment.to.dist_squared(circle.center) >= radius_squared)
}

/// Even-odd rule: true if a ray from `point` to the right crosses `segments` an odd number of times.
pub(super) fn ray_cast_inside<'a>(
    segments: impl Iterator<Item = (&'a V2, &'a V2)>,
    point: V2,
) -> bool {
    segments
        .filter(|(from, to)| {
            (from.y > point.y) != (to.y > point.y)
                && point.x < (to.x - from.x) * (point.y - from.y) / (to.y - from.y) + from.x
        })
        .count()
        % 2
        == 1
}
