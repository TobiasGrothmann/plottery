use crate::{
    traits::{ClosestPoint, Normalize, Scale, Scale2D, Translate},
    Angle, BoundingBox, Mirror, Path, Plottable, Rotate90, SampleSettings, Shape, LARGE_EPSILON,
    V2,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    bot_left: V2,
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
