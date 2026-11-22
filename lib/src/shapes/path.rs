use geo::{LineString, Polygon, SimplifyIdx};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    borrow::{Borrow, Cow},
    ops::Deref,
    slice::{Iter, IterMut},
};

use super::containment::contains_shape_by_variant;
use super::segments::{any_segments_touch, ray_cast_inside, segment_touches_circle, segments};
use crate::{
    geometry::TransformMatrix,
    traits::{bounds_may_touch, ClosestPoint, Scale, Scale2D, Transform, Translate},
    Angle, BoundingBox, Circle, Containment, Line, Mirror, Plottable, Rect, Rotate, Rotate90,
    SampleSettings, Shape, V2,
};

/// A polyline. Derefs to its points, so slice methods like `len`, `first`, `last` and `windows` are available.
/// Note that `len()` is the number of points while [`Plottable::length`] is the geometric length.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Path {
    points: Vec<V2>,
}

impl Path {
    fn map_points(&self, f: impl Fn(V2) -> V2) -> Path {
        self.iter().copied().map(f).collect()
    }
    fn map_points_mut(&mut self, f: impl Fn(V2) -> V2) {
        for point in self.points.iter_mut() {
            *point = f(*point);
        }
    }

    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
    /// Path from anything that yields points, owned or borrowed.
    pub fn new_from<I>(points: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<V2>,
    {
        Self {
            points: points.into_iter().map(|p| *p.borrow()).collect(),
        }
    }
    /// Path with the two points `from` and `to`.
    pub fn line(from: V2, to: V2) -> Self {
        Self {
            points: vec![from, to],
        }
    }

    pub fn new_shape() -> Shape {
        Shape::Path(Self { points: vec![] })
    }
    pub fn new_shape_from<I>(points: I) -> Shape
    where
        I: IntoIterator,
        I::Item: Borrow<V2>,
    {
        Shape::Path(Self::new_from(points))
    }
    pub fn new_shape_from_geo_polygon(geo_polygon: Polygon<f32>) -> Shape {
        Shape::Path(Self::from_iter(
            geo_polygon.exterior().into_iter().map(V2::new_from_geo),
        ))
    }
    pub fn new_shape_from_geo_line_string(geo_line_string: &LineString<f32>) -> Shape {
        Shape::Path(Self::from_iter(
            geo_line_string.into_iter().map(V2::new_from_geo),
        ))
    }

    pub fn push(&mut self, point: V2) {
        self.points.push(point);
    }
    /// Appends points, owned or borrowed.
    pub fn push_many<I>(&mut self, points: I)
    where
        I: IntoIterator,
        I::Item: Borrow<V2>,
    {
        self.points.extend(points.into_iter().map(|p| *p.borrow()));
    }

    pub fn close_mut(&mut self) {
        if let Some(first) = self.points.first() {
            self.points.push(*first);
        }
    }
    #[must_use]
    pub fn close(&self) -> Self {
        let mut path = self.clone();
        path.close_mut();
        path
    }
    pub fn reverse_mut(&mut self) {
        self.points.reverse();
    }
    #[must_use]
    pub fn reverse(&self) -> Self {
        let mut points = self.points.clone();
        points.reverse();
        Self { points }
    }

    pub fn iter(&self) -> Iter<'_, V2> {
        self.points.iter()
    }
    /// The segments between consecutive points.
    pub fn segments(&self) -> impl Iterator<Item = Line> + Clone + '_ {
        segments(&self.points)
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, V2> {
        self.points.iter_mut()
    }

    pub fn as_slice(&self) -> &[V2] {
        &self.points
    }
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    /// True if the path draws nothing: fewer than two distinct points.
    pub fn is_degenerate(&self) -> bool {
        self.points.windows(2).all(|pair| pair[0] == pair[1])
    }
    pub fn start(&self) -> Option<&V2> {
        self.points.first()
    }
    pub fn end(&self) -> Option<&V2> {
        self.points.last()
    }

    #[must_use]
    pub fn round_chaikins(&self, iterations: i32) -> Self {
        let mut new_points = self.points.clone();
        for _ in 0..iterations {
            new_points = Self::round_chaikins_iteration(new_points);
        }
        Self::new_from(new_points)
    }

    fn round_chaikins_iteration(points: Vec<V2>) -> Vec<V2> {
        if points.len() <= 2 {
            return points;
        }
        let (first, last) = (points[0], points[points.len() - 1]);
        // a closed path also cuts the corner at its start, an open one keeps both end points
        let closed = first == last;
        let segment_count = points.len() - 1;

        let mut new_points = Vec::with_capacity(points.len() * 2 + 1);
        if !closed {
            new_points.push(first);
        }
        for (i, (from, to)) in points.iter().tuple_windows().enumerate() {
            let vec = *to - *from;
            if closed || i > 0 {
                new_points.push(*from + vec * 0.25);
            }
            if closed || i < segment_count - 1 {
                new_points.push(*from + vec * 0.75);
            }
        }
        new_points.push(if closed { new_points[0] } else { last });
        new_points
    }

    pub fn arc(
        from: Angle,
        to: Angle,
        center: V2,
        radius: f32,
        sample_settings: SampleSettings,
    ) -> Self {
        let arc_length = (to.to_rad() - from.to_rad()).abs() * radius;
        let num_points = sample_settings.num_points_for_length(arc_length);

        (0..=num_points)
            .map(|i| {
                let rot = from + (to - from) * (i as f32 / num_points as f32);
                center + V2::polar(rot, radius)
            })
            .collect()
    }

    /// Arc around `center` from `a` to `b`, taking the shorter way round.
    pub fn arc_between(
        center: V2,
        radius: f32,
        a: V2,
        b: V2,
        sample_settings: SampleSettings,
    ) -> Self {
        let from = (a - center).angle();
        let mut sweep = ((b - center).angle() - from).positive();
        if sweep > Angle::half_rotation() {
            sweep -= Angle::full_rotation();
        }
        Path::arc(from, from + sweep, center, radius, sample_settings)
    }

    pub fn intersects_circle(&self, other: &Circle) -> bool {
        if !bounds_may_touch(self, other) {
            return false;
        }

        self.segments()
            .any(|segment| segment_touches_circle(segment, other))
    }

    pub fn intersects_rect(&self, other: &Rect) -> bool {
        if !bounds_may_touch(self, other) {
            return false;
        }

        any_segments_touch(self.segments(), other.edges().into_iter())
    }

    pub fn intersects_path(&self, other: &Path) -> bool {
        if !bounds_may_touch(self, other) {
            return false;
        }

        any_segments_touch(self.segments(), other.segments())
    }

    pub(crate) fn points_closed(&self) -> Cow<'_, [V2]> {
        if self.is_closed() || self.points.is_empty() {
            return Cow::Borrowed(&self.points);
        }

        let mut points = self.points.clone();
        points.push(*points.first().unwrap());
        Cow::Owned(points)
    }

    pub(crate) fn contains_point_or_on_boundary_in_closed_points(
        points_closed: &[V2],
        point: V2,
    ) -> bool {
        if points_closed.len() < 2 {
            return false;
        }

        segments(points_closed).any(|segment| segment.contains_point_on_segment(point))
            || ray_cast_inside(points_closed.iter().tuple_windows(), point)
    }

    pub fn contains_circle(&self, other: &Circle) -> Containment {
        if !bounds_may_touch(self, other) {
            return Containment::None;
        }
        let self_points = self.points_closed();
        if self_points.len() < 2 {
            return Containment::None;
        }

        let radius_squared = other.radius.powi(2);
        if segments(&self_points).any(|segment| segment_touches_circle(segment, other)) {
            return Containment::Partial;
        }

        let center_inside = Self::contains_point_or_on_boundary_in_closed_points(
            self_points.as_ref(),
            other.center,
        );
        let min_dist_to_boundary_squared = segments(&self_points)
            .map(|segment| {
                segment
                    .closest_point(other.center)
                    .dist_squared(other.center)
            })
            .fold(f32::INFINITY, f32::min);

        if center_inside && min_dist_to_boundary_squared >= radius_squared {
            return Containment::Full;
        }

        if center_inside
            || self_points
                .iter()
                .any(|point| point.dist_squared(other.center) <= radius_squared)
        {
            return Containment::Partial;
        }

        Containment::None
    }

    pub fn contains_rect(&self, other: &Rect) -> Containment {
        if !bounds_may_touch(self, other) {
            return Containment::None;
        }
        let self_points = self.points_closed();
        if self_points.len() < 2 {
            return Containment::None;
        }

        let rect_points = other.corners();
        if any_segments_touch(segments(&self_points), other.edges().into_iter()) {
            return Containment::Partial;
        }

        let all_rect_points_inside = rect_points.iter().all(|point| {
            Self::contains_point_or_on_boundary_in_closed_points(self_points.as_ref(), *point)
        });
        if all_rect_points_inside {
            return Containment::Full;
        }

        if rect_points.iter().any(|point| {
            Self::contains_point_or_on_boundary_in_closed_points(self_points.as_ref(), *point)
        }) || self_points.iter().any(|point| other.contains_point(*point))
        {
            return Containment::Partial;
        }

        Containment::None
    }

    pub fn contains_path(&self, other: &Path) -> Containment {
        if !bounds_may_touch(self, other) {
            return Containment::None;
        }
        let self_points = self.points_closed();
        let other_points = other.points_closed();
        if self_points.len() < 2 || other_points.len() < 2 {
            return Containment::None;
        }

        if any_segments_touch(segments(&self_points), segments(&other_points)) {
            return Containment::Partial;
        }

        let all_other_points_inside = other_points.iter().all(|point| {
            Self::contains_point_or_on_boundary_in_closed_points(self_points.as_ref(), *point)
        });
        if all_other_points_inside {
            return Containment::Full;
        }

        if other_points.iter().any(|point| {
            Self::contains_point_or_on_boundary_in_closed_points(self_points.as_ref(), *point)
        }) || self_points.iter().any(|point| {
            Self::contains_point_or_on_boundary_in_closed_points(other_points.as_ref(), *point)
        }) {
            return Containment::Partial;
        }

        Containment::None
    }

    contains_shape_by_variant!();
}

impl Plottable for Path {
    fn points(&self, _: SampleSettings) -> Vec<V2> {
        self.points.clone()
    }

    fn points_from(
        &self,
        current_drawing_head_pos: V2,
        sample_settings: SampleSettings,
    ) -> Vec<V2> {
        let mut points = self.points(sample_settings);
        if points.is_empty() {
            return points;
        }
        if current_drawing_head_pos.dist(*points.last().unwrap())
            < current_drawing_head_pos.dist(*points.first().unwrap())
        {
            points.reverse();
        }
        points
    }

    fn length(&self) -> f32 {
        self.points
            .iter()
            .tuple_windows()
            .fold(0.0, |acc, (from, to)| acc + from.dist(*to))
    }

    fn is_closed(&self) -> bool {
        !self.points.is_empty() && self.points.first() == self.points.last()
    }

    fn contains_point(&self, point: V2) -> bool {
        ray_cast_inside(self.points.iter().circular_tuple_windows(), point)
    }

    fn reduce_points(&self, aggression_factor: f32) -> Self {
        let epsilon = 0.0001 + aggression_factor.clamp(0.0, 1.0).powf(3.0);

        // in f64 so nearly collinear points are judged as precisely as before
        let line: LineString<f64> = self
            .points
            .iter()
            .map(|point| (point.x as f64, point.y as f64))
            .collect();
        line.simplify_idx(epsilon as f64)
            .into_iter()
            .map(|index| self.points[index])
            .collect()
    }
}

impl FromIterator<V2> for Path {
    fn from_iter<I: IntoIterator<Item = V2>>(iter: I) -> Self {
        Path {
            points: iter.into_iter().collect(),
        }
    }
}

impl<'a> FromIterator<&'a V2> for Path {
    fn from_iter<I: IntoIterator<Item = &'a V2>>(iter: I) -> Self {
        Path {
            points: iter.into_iter().copied().collect(),
        }
    }
}

impl Rotate for Path {
    fn rotate(&self, angle: Angle) -> Self {
        self.map_points(|p| p.rotate(angle))
    }
    fn rotate_mut(&mut self, angle: Angle) {
        self.map_points_mut(|p| p.rotate(angle));
    }
    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        self.map_points(|p| p.rotate_around(pivot, angle))
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        self.map_points_mut(|p| p.rotate_around(pivot, angle));
    }
}

impl Rotate90 for Path {
    fn rotate_90(&self) -> Self {
        self.map_points(|p| p.rotate_90())
    }
    fn rotate_90_mut(&mut self) {
        self.map_points_mut(|p| p.rotate_90());
    }
    fn rotate_180(&self) -> Self {
        self.map_points(|p| p.rotate_180())
    }
    fn rotate_180_mut(&mut self) {
        self.map_points_mut(|p| p.rotate_180());
    }
    fn rotate_270(&self) -> Self {
        self.map_points(|p| p.rotate_270())
    }
    fn rotate_270_mut(&mut self) {
        self.map_points_mut(|p| p.rotate_270());
    }
    fn rotate_90_around(&self, pivot: V2) -> Self {
        self.map_points(|p| p.rotate_90_around(pivot))
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        self.map_points_mut(|p| p.rotate_90_around(pivot));
    }
    fn rotate_180_around(&self, pivot: V2) -> Self {
        self.map_points(|p| p.rotate_180_around(pivot))
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        self.map_points_mut(|p| p.rotate_180_around(pivot));
    }
    fn rotate_270_around(&self, pivot: V2) -> Self {
        self.map_points(|p| p.rotate_270_around(pivot))
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        self.map_points_mut(|p| p.rotate_270_around(pivot));
    }
}

impl IntoIterator for Path {
    type Item = V2;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a V2;
    type IntoIter = Iter<'a, V2>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl Deref for Path {
    type Target = [V2];

    fn deref(&self) -> &[V2] {
        &self.points
    }
}

impl Extend<V2> for Path {
    fn extend<I: IntoIterator<Item = V2>>(&mut self, iter: I) {
        self.points.extend(iter);
    }
}

impl<'a> Extend<&'a V2> for Path {
    fn extend<I: IntoIterator<Item = &'a V2>>(&mut self, iter: I) {
        self.points.extend(iter);
    }
}

impl Translate for Path {
    fn translate(&self, dist: V2) -> Self {
        self.map_points(|p| p + dist)
    }
    fn translate_mut(&mut self, dist: V2) {
        self.map_points_mut(|p| p + dist);
    }
}

impl Scale for Path {
    fn scale(&self, scale: f32) -> Self {
        self.map_points(|p| p * scale)
    }
    fn scale_mut(&mut self, scale: f32) {
        self.map_points_mut(|p| p * scale);
    }
}

impl Scale2D for Path {
    fn scale_2d(&self, factor: V2) -> Self {
        self.map_points(|p| p * factor)
    }
    fn scale_2d_mut(&mut self, factor: V2) {
        self.map_points_mut(|p| p * factor);
    }
}

impl BoundingBox for Path {
    fn bounding_box(&self) -> Option<Rect> {
        Rect::from_points(self.iter())
    }
}

impl Transform for Path {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        self.map_points(|p| matrix.mul_vector(p))
    }
    fn transform_mut(&mut self, matrix: &TransformMatrix) {
        self.map_points_mut(|p| matrix.mul_vector(p));
    }
}

impl Mirror for Path {
    fn mirror_x(&self) -> Self {
        self.map_points(|p| p.mirror_x())
    }
    fn mirror_x_mut(&mut self) {
        self.map_points_mut(|p| p.mirror_x());
    }
    fn mirror_y(&self) -> Self {
        self.map_points(|p| p.mirror_y())
    }
    fn mirror_y_mut(&mut self) {
        self.map_points_mut(|p| p.mirror_y());
    }
}

impl ClosestPoint for Path {
    fn closest_point(&self, _: SampleSettings, point: V2) -> Option<V2> {
        self.segments()
            .map(|line| {
                let closest = line.closest_point(point);
                (closest, closest.dist_squared(point))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(closest, _)| closest)
    }
}

impl From<Vec<V2>> for Path {
    fn from(points: Vec<V2>) -> Self {
        Self { points }
    }
}

impl From<&[V2]> for Path {
    fn from(points: &[V2]) -> Self {
        Self {
            points: points.to_vec(),
        }
    }
}

impl From<Vec<&V2>> for Path {
    fn from(points: Vec<&V2>) -> Self {
        Self {
            points: points.into_iter().cloned().collect(),
        }
    }
}

impl From<Line> for Path {
    fn from(line: Line) -> Self {
        Path::line(line.from, line.to)
    }
}
