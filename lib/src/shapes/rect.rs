use crate::{
    traits::{ClosestPoint, Normalize, Scale, Scale2D, Translate},
    Angle, BoundingBox, Circle, Containment, Line, LineIntersection, Mirror, Path, Plottable,
    PointLineRelation, Rotate90, SampleSettings, Shape, LARGE_EPSILON, V2,
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

    fn fix_corners_min_max(&mut self) {
        let bl = self.bot_left.min(self.top_right);
        let tr = self.bot_left.max(self.top_right);
        self.bot_left = bl;
        self.top_right = tr;
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
        (self.width() - self.height()).abs() < LARGE_EPSILON
    }

    pub fn left_mid(&self) -> V2 {
        V2::new(self.bot_left.x, self.bot_left.y + self.height() * 0.5)
    }
    pub fn right_mid(&self) -> V2 {
        V2::new(self.top_right.x, self.top_right.y - self.height() * 0.5)
    }
    pub fn top_mid(&self) -> V2 {
        V2::new(self.top_right.x - self.width() * 0.5, self.top_right.y)
    }
    pub fn bot_mid(&self) -> V2 {
        V2::new(self.bot_left.x + self.width() * 0.5, self.bot_left.y)
    }

    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    pub fn intersects_rect(&self, other: &Rect) -> bool {
        let self_edges = [
            (self.bl(), self.tl()),
            (self.tl(), self.tr()),
            (self.tr(), self.br()),
            (self.br(), self.bl()),
        ];
        let other_edges = [
            (other.bl(), other.tl()),
            (other.tl(), other.tr()),
            (other.tr(), other.br()),
            (other.br(), other.bl()),
        ];

        self_edges.iter().any(|(self_from, self_to)| {
            let self_segment = Line::new(*self_from, *self_to);
            other_edges.iter().any(|(other_from, other_to)| {
                segments_intersect_or_touch(self_segment, Line::new(*other_from, *other_to))
            })
        })
    }

    pub fn intersects_circle(&self, other: &Circle) -> bool {
        let radius_squared = other.radius.powi(2);
        let edges = [
            (self.bl(), self.tl()),
            (self.tl(), self.tr()),
            (self.tr(), self.br()),
            (self.br(), self.bl()),
        ];

        edges.iter().any(|(from, to)| {
            let segment = Line::new(*from, *to);
            let from_dist_squared = from.dist_squared(other.center);
            let to_dist_squared = to.dist_squared(other.center);
            let closest_dist_squared = segment
                .closest_point(other.center)
                .dist_squared(other.center);

            closest_dist_squared <= radius_squared
                && (from_dist_squared >= radius_squared || to_dist_squared >= radius_squared)
        })
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
            || [self.bl(), self.tl(), self.tr(), self.br()]
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
        let other_points_closed = other.points_closed();
        if other_points_closed.is_empty() {
            return Containment::None;
        }

        if other_points_closed
            .iter()
            .all(|point| self.contains_point(*point))
        {
            return Containment::Full;
        }

        let other_closed = Path::new_from(other_points_closed.clone());
        if self.intersects_path(&other_closed)
            || other_points_closed
                .iter()
                .any(|point| self.contains_point(*point))
            || [self.bl(), self.tl(), self.tr(), self.br()]
                .iter()
                .any(|corner| other.contains_point_or_on_boundary_as_closed(*corner))
        {
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

    pub fn contains_shape(&self, other: &Shape) -> Containment {
        match other {
            Shape::Circle(c) => self.contains_circle(c),
            Shape::Rect(r) => self.contains_rect(r),
            Shape::Path(p) => self.contains_path(p),
        }
    }

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

fn segments_intersect_or_touch(a: Line, b: Line) -> bool {
    if matches!(a.intersection(b), LineIntersection::Intersection(_)) {
        return true;
    }

    if a.point_relation(b.from) == PointLineRelation::OnLine
        && a.point_relation(b.to) == PointLineRelation::OnLine
    {
        let a_min_x = a.from.x.min(a.to.x);
        let a_max_x = a.from.x.max(a.to.x);
        let a_min_y = a.from.y.min(a.to.y);
        let a_max_y = a.from.y.max(a.to.y);

        let b_min_x = b.from.x.min(b.to.x);
        let b_max_x = b.from.x.max(b.to.x);
        let b_min_y = b.from.y.min(b.to.y);
        let b_max_y = b.from.y.max(b.to.y);

        let overlaps_x = a_min_x <= b_max_x && a_max_x >= b_min_x;
        let overlaps_y = a_min_y <= b_max_y && a_max_y >= b_min_y;

        return overlaps_x && overlaps_y;
    }

    false
}

impl Plottable for Rect {
    fn get_points(&self, _: SampleSettings) -> Vec<V2> {
        vec![self.bl(), self.tl(), self.tr(), self.br(), self.bl()]
    }
    fn get_points_from(
        &self,
        _current_drawing_head_pos: V2,
        sample_settings: SampleSettings,
    ) -> Vec<V2> {
        self.get_points(sample_settings)
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
        Rect::new(self.bot_left.rotate_90(), self.top_right.rotate_90())
    }
    fn rotate_90_mut(&mut self) {
        self.top_right = self.top_right.rotate_90();
        self.bot_left = self.bot_left.rotate_90();
        self.fix_corners_min_max();
    }
    fn rotate_180(&self) -> Self {
        Rect::new(self.bot_left.rotate_180(), self.top_right.rotate_180())
    }
    fn rotate_180_mut(&mut self) {
        self.top_right = self.top_right.rotate_180();
        self.bot_left = self.bot_left.rotate_180();
        self.fix_corners_min_max();
    }
    fn rotate_270(&self) -> Self {
        Rect::new(self.bot_left.rotate_270(), self.top_right.rotate_270())
    }
    fn rotate_270_mut(&mut self) {
        self.top_right = self.top_right.rotate_270();
        self.bot_left = self.bot_left.rotate_270();
        self.fix_corners_min_max();
    }

    fn rotate_90_around(&self, pivot: V2) -> Self {
        Rect::new(
            self.bot_left.rotate_90_around(pivot),
            self.top_right.rotate_90_around(pivot),
        )
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        self.top_right = self.top_right.rotate_90_around(pivot);
        self.bot_left = self.bot_left.rotate_90_around(pivot);
        self.fix_corners_min_max();
    }
    fn rotate_180_around(&self, pivot: V2) -> Self {
        Rect::new(
            self.bot_left.rotate_180_around(pivot),
            self.top_right.rotate_180_around(pivot),
        )
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        self.top_right = self.top_right.rotate_180_around(pivot);
        self.bot_left = self.bot_left.rotate_180_around(pivot);
        self.fix_corners_min_max();
    }
    fn rotate_270_around(&self, pivot: V2) -> Self {
        Rect::new(
            self.bot_left.rotate_270_around(pivot),
            self.top_right.rotate_270_around(pivot),
        )
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        self.top_right = self.top_right.rotate_270_around(pivot);
        self.bot_left = self.bot_left.rotate_270_around(pivot);
        self.fix_corners_min_max();
    }
}

impl Translate for Rect {
    fn translate(&self, dist: V2) -> Self {
        Rect::new(self.bot_left + dist, self.top_right + dist)
    }

    fn translate_mut(&mut self, dist: V2) {
        self.bot_left += dist;
        self.top_right += dist;
    }
}

impl Scale for Rect {
    fn scale(&self, scale: f32) -> Self {
        Rect::new(self.bot_left * scale, self.top_right * scale)
    }

    fn scale_mut(&mut self, scale: f32) {
        self.bot_left *= scale;
        self.top_right *= scale;
    }
}

impl Scale2D for Rect {
    fn scale_2d(&self, scale: V2) -> Self {
        Rect::new(self.bot_left * scale, self.top_right * scale)
    }

    fn scale_2d_mut(&mut self, scale: V2) {
        self.bot_left *= scale;
        self.top_right *= scale;
    }
}

impl Normalize for Rect {}

impl Mirror for Rect {
    fn mirror_x(&self) -> Self {
        Rect::new(self.bot_left.mirror_x(), self.top_right.mirror_x())
    }

    fn mirror_x_mut(&mut self) {
        self.bot_left.mirror_x_mut();
        self.top_right.mirror_x_mut();
        self.fix_corners_min_max();
    }

    fn mirror_y(&self) -> Self {
        Rect::new(self.bot_left.mirror_y(), self.top_right.mirror_y())
    }

    fn mirror_y_mut(&mut self) {
        self.bot_left.mirror_y_mut();
        self.top_right.mirror_y_mut();
        self.fix_corners_min_max();
    }
}

impl BoundingBox for Rect {
    fn bounding_box(&self) -> Option<Rect> {
        Some(*self)
    }
}

impl ClosestPoint for Rect {
    fn closest_point(&self, _: SampleSettings, point: V2) -> Option<V2> {
        Path::new_from(vec![self.bl(), self.tl(), self.tr(), self.br(), self.bl()])
            .closest_point(SampleSettings::default(), point)
    }
}
