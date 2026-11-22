use super::containment::contains_shape_by_variant;
use super::segments::{any_segments_touch, segment_touches_circle, segments};
use crate::{
    traits::{bounds_may_touch, ClosestPoint, Scale, Scale2D, Translate},
    Angle, BoundingBox, Circle, Containment, Line, Mirror, Path, Plottable, Rotate, Rotate90,
    SampleSettings, Shape, LENGTH_TOLERANCE, V2,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    /// guaranteed: `bot_left.x <= top_right.x && bot_left.y <= top_right.y`
    bot_left: V2,
    /// guaranteed: `top_right.x >= bot_left.x && top_right.y >= bot_left.y`
    top_right: V2,
}

impl Rect {
    pub fn new(bl: V2, tr: V2) -> Self {
        Self {
            bot_left: bl.min(tr),
            top_right: tr.max(bl),
        }
    }
    /// Smallest rect around all `points`, `None` if there are none.
    pub fn from_points<P: std::borrow::Borrow<V2>>(
        points: impl IntoIterator<Item = P>,
    ) -> Option<Rect> {
        let mut points = points.into_iter().map(|p| *p.borrow());
        let first = points.next()?;
        let (min, max) = points.fold((first, first), |(min, max), p| (min.min(p), max.max(p)));
        Some(Rect::new(min, max))
    }
    /// Smallest rect around both rects.
    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        Rect::new(
            self.bot_left.min(other.bot_left),
            self.top_right.max(other.top_right),
        )
    }
    pub fn new_shape(bl: V2, tr: V2) -> Shape {
        Shape::Rect(Rect::new(bl, tr))
    }
    pub fn new_from_center(center: V2, size: V2) -> Self {
        Self {
            bot_left: center - size * 0.5,
            top_right: center + size * 0.5,
        }
    }
    pub fn new_shape_from_center(center: V2, size: V2) -> Shape {
        Shape::Rect(Rect::new_from_center(center, size))
    }

    /// Both corners mapped by `f`, sorted again into bottom left and top right.
    fn map_corners(&self, f: impl Fn(V2) -> V2) -> Rect {
        Rect::new(f(self.bot_left), f(self.top_right))
    }

    pub fn tr(&self) -> V2 {
        self.top_right
    }
    pub fn tl(&self) -> V2 {
        V2::new(self.bot_left.x, self.top_right.y)
    }
    pub fn bl(&self) -> V2 {
        self.bot_left
    }
    pub fn br(&self) -> V2 {
        V2::new(self.top_right.x, self.bot_left.y)
    }

    /// Corners in drawing order: bottom left, top left, top right, bottom right.
    pub fn corners(&self) -> [V2; 4] {
        [self.bl(), self.tl(), self.tr(), self.br()]
    }
    /// Edges in drawing order, see [`Rect::corners`].
    pub fn edges(&self) -> [Line; 4] {
        let [a, b, c, d] = self.corners();
        [
            Line::new(a, b),
            Line::new(b, c),
            Line::new(c, d),
            Line::new(d, a),
        ]
    }

    /// Square with the given center and half side length.
    pub fn square_around(center: V2, half_size: f32) -> Self {
        Self::new(center - V2::xy(half_size), center + V2::xy(half_size))
    }

    /// Grows the rect by `by` on every side. Negative values shrink it.
    #[must_use]
    pub fn expand(&self, by: f32) -> Self {
        self.expand_2d(V2::xy(by))
    }
    /// Grows the rect by `by.x` on the left and right and `by.y` on the top and bottom.
    #[must_use]
    pub fn expand_2d(&self, by: V2) -> Self {
        Self::new(self.bot_left - by, self.top_right + by)
    }
    #[must_use]
    pub fn shrink(&self, by: f32) -> Self {
        self.expand(-by)
    }
    #[must_use]
    pub fn shrink_2d(&self, by: V2) -> Self {
        self.expand_2d(-by)
    }

    pub fn random_point(&self) -> V2 {
        V2::random_in_rect(*self)
    }

    /// The outline rotated around the origin; a rotated rect is a [`Path`].
    #[must_use]
    pub fn rotate(&self, angle: Angle) -> Path {
        self.rotate_around(V2::zero(), angle)
    }
    /// The outline rotated around `pivot`; a rotated rect is a [`Path`].
    #[must_use]
    pub fn rotate_around(&self, pivot: V2, angle: Angle) -> Path {
        self.to_path(SampleSettings::default())
            .rotate_around(pivot, angle)
    }

    pub fn top_center(&self) -> V2 {
        V2::new(self.bot_left.x + self.width() * 0.5, self.top_right.y)
    }
    pub fn left_center(&self) -> V2 {
        V2::new(self.bot_left.x, self.bot_left.y + self.height() * 0.5)
    }
    pub fn right_center(&self) -> V2 {
        V2::new(self.top_right.x, self.bot_left.y + self.height() * 0.5)
    }
    pub fn bottom_center(&self) -> V2 {
        V2::new(self.bot_left.x + self.width() * 0.5, self.bot_left.y)
    }

    pub fn size(&self) -> V2 {
        self.top_right - self.bot_left
    }
    pub fn height(&self) -> f32 {
        self.top_right.y - self.bot_left.y
    }
    pub fn width(&self) -> f32 {
        self.top_right.x - self.bot_left.x
    }

    pub fn max_dist_to_any_corner(&self, point: V2) -> f32 {
        *[
            self.bl().dist(point),
            self.br().dist(point),
            self.tr().dist(point),
            self.tl().dist(point),
        ]
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
    }

    pub fn center(&self) -> V2 {
        V2::new(
            (self.bot_left.x + self.top_right.x) * 0.5,
            (self.bot_left.y + self.top_right.y) * 0.5,
        )
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.width() / self.height()
    }
    pub fn is_square(&self) -> bool {
        (self.width() - self.height()).abs() < LENGTH_TOLERANCE
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    /// Path to draw a rect with a given thickness.
    /// It creates a spiraling path from the rectangle border towards the center with `thickness`.
    pub fn filled_partially_towards_center(
        &self,
        mut thickness: f32,
        pen_width: f32,
    ) -> super::Path {
        if pen_width <= LENGTH_TOLERANCE {
            return Path::new_from(self.points(SampleSettings::default()));
        }

        let max_thickness_x = (self.width() * 0.5 - pen_width * 0.5).max(0.0);
        let max_thickness_y = (self.height() * 0.5 - pen_width * 0.5).max(0.0);
        thickness = thickness.min(max_thickness_x.min(max_thickness_y));

        if thickness <= LENGTH_TOLERANCE {
            return Path::new_from(self.points(SampleSettings::default()));
        }

        let thickness_x = thickness.min(max_thickness_x);
        let thickness_y = thickness.min(max_thickness_y);

        let num_steps_x = (thickness_x / pen_width).ceil().max(1.0) as usize;
        let num_steps_y = (thickness_y / pen_width).ceil().max(1.0) as usize;
        let spiral_steps = num_steps_x.min(num_steps_y);
        let step_x = thickness_x / num_steps_x as f32;
        let step_y = thickness_y / num_steps_y as f32;

        self.spiral_fill_with_steps(spiral_steps, step_x, step_y)
    }

    /// Path to draw a rect's outer border and spiral filling.
    pub fn filled_spiral(&self, pen_width: f32) -> super::Path {
        let max_thickness = (self.width().min(self.height()) * 0.5 - pen_width * 0.4).max(0.0);
        self.filled_partially_towards_center(max_thickness, pen_width)
    }

    fn spiral_fill_with_steps(&self, max_steps: usize, step_x: f32, step_y: f32) -> super::Path {
        let border = self.points(SampleSettings::default());

        if max_steps == 0 || step_x <= LENGTH_TOLERANCE || step_y <= LENGTH_TOLERANCE {
            return Path::new_from(border);
        }

        let mut left = self.bot_left.x + step_x;
        let mut right = self.top_right.x - step_x;
        let mut bottom = self.bot_left.y + step_y;
        let mut top = self.top_right.y - step_y;

        if left > right || bottom > top {
            return Path::new_from(border);
        }

        let mut points = Vec::with_capacity(4 + 1 + max_steps * 4);
        for point in border {
            points.push(point);
        }
        points.push(V2::new(left, bottom));

        for _ in 0..max_steps {
            points.push(V2::new(left, top));
            left += step_x;
            if left > right + LENGTH_TOLERANCE {
                break;
            }

            points.push(V2::new(right, top));
            top -= step_y;
            if bottom > top + LENGTH_TOLERANCE {
                break;
            }

            points.push(V2::new(right, bottom));
            right -= step_x;
            if left > right + LENGTH_TOLERANCE {
                break;
            }

            points.push(V2::new(left, bottom));
            bottom += step_y;
            if bottom > top + LENGTH_TOLERANCE {
                break;
            }
        }

        Path::new_from(points)
    }

    /// Path to draw a rect's outer border and criss cross filling
    pub fn filled_criss_cross(&self, pen_width: f32) -> super::Path {
        let outer_border = self.points(SampleSettings::default());

        if self.height() <= pen_width * 0.5 || self.width() <= pen_width * 0.5 {
            return Path::new_from(outer_border);
        }

        // If we cannot inset by `pen_width` on all sides, skip filling.
        // Otherwise `Rect::new` would normalize inverted corners into a bigger rect.
        if self.width() <= pen_width * 2.0 || self.height() <= pen_width * 2.0 {
            return Path::new_from(outer_border);
        }

        let smaller_inner = Rect::new(
            self.bot_left + V2::xy(pen_width),
            self.top_right - V2::xy(pen_width),
        );
        let filling = smaller_inner.criss_cross_filling(pen_width);

        Path::new_from(outer_border.into_iter().chain(filling))
    }

    fn criss_cross_filling(&self, pen_width: f32) -> Vec<V2> {
        let height = self.height();
        if height <= pen_width * 0.05 {
            return Vec::new();
        }
        if self.height() <= pen_width * 0.5 {
            return self.points(SampleSettings::default());
        }

        let num_lines = (self.height() / pen_width).ceil() as usize;
        let height_per_line = self.height() / num_lines as f32;

        let mut points = Vec::with_capacity(num_lines * 2);
        for i in 0..(num_lines + 1) {
            let y = self.bot_left.y + i as f32 * height_per_line;
            if i % 2 == 0 {
                points.push(V2::new(self.bot_left.x, y));
                points.push(V2::new(self.top_right.x, y));
            } else {
                points.push(V2::new(self.top_right.x, y));
                points.push(V2::new(self.bot_left.x, y));
            }
        }

        points
    }

    /// True if the rects share at least one point, edges included.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.bot_left.x <= other.top_right.x
            && other.bot_left.x <= self.top_right.x
            && self.bot_left.y <= other.top_right.y
            && other.bot_left.y <= self.top_right.y
    }

    pub fn intersects_rect(&self, other: &Rect) -> bool {
        any_segments_touch(self.edges(), other.edges().into_iter())
    }

    pub fn intersects_circle(&self, other: &Circle) -> bool {
        self.edges()
            .into_iter()
            .any(|edge| segment_touches_circle(edge, other))
    }

    pub fn intersects_path(&self, other: &Path) -> bool {
        other.intersects_rect(self)
    }

    pub fn contains_circle(&self, other: &Circle) -> Containment {
        let fully_inside = other.center.x - other.radius >= self.bot_left.x
            && other.center.x + other.radius <= self.top_right.x
            && other.center.y - other.radius >= self.bot_left.y
            && other.center.y + other.radius <= self.top_right.y;

        if fully_inside {
            return Containment::Full;
        }

        if self.intersects_circle(other)
            || self.contains_point(other.center)
            || self
                .corners()
                .iter()
                .any(|corner| corner.dist_squared(other.center) <= other.radius.powi(2))
        {
            return Containment::Partial;
        }

        Containment::None
    }

    pub fn contains_rect(&self, other: &Rect) -> Containment {
        let fully_inside = self.contains_point(other.bl())
            && self.contains_point(other.tl())
            && self.contains_point(other.tr())
            && self.contains_point(other.br());

        if fully_inside {
            return Containment::Full;
        }

        if self.intersects_rect(other) || self.overlaps_area_rect(other) {
            return Containment::Partial;
        }

        Containment::None
    }

    pub fn contains_path(&self, other: &Path) -> Containment {
        if !bounds_may_touch(self, other) {
            return Containment::None;
        }
        let other_points_closed = other.points_closed();

        if other_points_closed
            .iter()
            .all(|point| self.contains_point(*point))
        {
            return Containment::Full;
        }

        if other_points_closed
            .iter()
            .any(|point| self.contains_point(*point))
            || self.corners().iter().any(|corner| {
                Path::contains_point_or_on_boundary_in_closed_points(
                    other_points_closed.as_ref(),
                    *corner,
                )
            })
        {
            return Containment::Partial;
        }

        if any_segments_touch(segments(&other_points_closed), self.edges().into_iter()) {
            return Containment::Partial;
        }

        Containment::None
    }

    fn overlaps_area_rect(&self, other: &Rect) -> bool {
        self.bot_left.x < other.top_right.x
            && self.top_right.x > other.bot_left.x
            && self.bot_left.y < other.top_right.y
            && self.top_right.y > other.bot_left.y
    }

    contains_shape_by_variant!();

    pub fn rounded_corners(
        &self,
        radius: f32,
        sample_settings: SampleSettings,
    ) -> anyhow::Result<Path> {
        if self.width() < 2.0 * radius || self.height() < 2.0 * radius {
            return Err(anyhow::anyhow!("Radius too large for rectangle"));
        }
        let half_radius = radius * 0.5;

        let bl_inner = self.bl() + V2::xy(half_radius);
        let tr_inner = self.tr() - V2::xy(half_radius);
        let br_inner = self.br() + V2::new(-half_radius, half_radius);
        let tl_inner = self.tl() + V2::new(half_radius, -half_radius);

        let path = std::iter::once(bl_inner + V2::polar(Angle::left_cc(), half_radius))
            .chain(Path::arc(
                Angle::left_cc(),
                Angle::up_cc(),
                tl_inner,
                half_radius,
                sample_settings,
            ))
            .chain(Path::arc(
                Angle::up_cc(),
                Angle::right_cc(),
                tr_inner,
                half_radius,
                sample_settings,
            ))
            .chain(Path::arc(
                Angle::full_rotation(),
                Angle::down_cc(),
                br_inner,
                half_radius,
                sample_settings,
            ))
            .chain(Path::arc(
                Angle::down_cc(),
                Angle::left_cc(),
                bl_inner,
                half_radius,
                sample_settings,
            ))
            .collect();
        Ok(path)
    }
}

impl Plottable for Rect {
    fn points(&self, _: SampleSettings) -> Vec<V2> {
        vec![self.bl(), self.tl(), self.tr(), self.br(), self.bl()]
    }
    fn points_from(
        &self,
        _current_drawing_head_pos: V2,
        sample_settings: SampleSettings,
    ) -> Vec<V2> {
        self.points(sample_settings)
    }

    fn length(&self) -> f32 {
        self.width() * 2.0 + self.height() * 2.0
    }

    fn is_closed(&self) -> bool {
        true
    }

    fn contains_point(&self, point: V2) -> bool {
        point.x >= self.bot_left.x
            && point.x <= self.top_right.x
            && point.y >= self.bot_left.y
            && point.y <= self.top_right.y
    }

    fn reduce_points(&self, _aggression_factor: f32) -> Self {
        *self
    }
}

impl Rotate90 for Rect {
    fn rotate_90(&self) -> Self {
        self.map_corners(|p| p.rotate_90())
    }
    fn rotate_180(&self) -> Self {
        self.map_corners(|p| p.rotate_180())
    }
    fn rotate_270(&self) -> Self {
        self.map_corners(|p| p.rotate_270())
    }
}

impl Translate for Rect {
    fn translate(&self, dist: V2) -> Self {
        self.map_corners(|p| p + dist)
    }
}

impl Scale for Rect {
    fn scale(&self, scale: f32) -> Self {
        self.map_corners(|p| p * scale)
    }
}

impl Scale2D for Rect {
    fn scale_2d(&self, scale: V2) -> Self {
        self.map_corners(|p| p * scale)
    }
}

impl Mirror for Rect {
    fn mirror_x(&self) -> Self {
        self.map_corners(|p| p.mirror_x())
    }
    fn mirror_y(&self) -> Self {
        self.map_corners(|p| p.mirror_y())
    }
}

impl BoundingBox for Rect {
    fn bounding_box(&self) -> Option<Rect> {
        Some(*self)
    }
}

impl ClosestPoint for Rect {
    fn closest_point(&self, _: SampleSettings, point: V2) -> Option<V2> {
        self.to_path(SampleSettings::default())
            .closest_point(SampleSettings::default(), point)
    }
}
