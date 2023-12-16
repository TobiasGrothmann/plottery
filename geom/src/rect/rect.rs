use crate::{
    shape::shape::{SampleSettings, Shape},
    traits::rotate90::Rotate90,
    vec2::V2,
};

#[derive(Debug, Clone)]
pub struct Rect {
    bot_left: V2,
    top_right: V2,
}

impl Rect {
    pub fn new(bl: V2, tr: V2) -> Self {
        Self {
            bot_left: bl.min(&tr),
            top_right: tr.max(&bl),
        }
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

    pub fn size(&self) -> V2 {
        self.top_right - self.bot_left
    }
    pub fn height(&self) -> f32 {
        self.top_right.y - self.bot_left.y
    }
    pub fn width(&self) -> f32 {
        self.top_right.x - self.bot_left.x
    }

    pub fn center(&self) -> V2 {
        self.bot_left + (self.size() * 0.5)
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
}

impl Shape for Rect {
    fn get_points(&self, _: &SampleSettings) -> Vec<V2> {
        vec![self.bl(), self.tl(), self.tr(), self.br(), self.bl()]
    }

    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }

    fn length(&self) -> f32 {
        self.width() * 2.0 + self.height() * 2.0
    }

    fn is_closed(&self) -> bool {
        true
    }
}

impl Rotate90 for Rect {
    fn rotate_90(&self) -> Self {
        Rect::new(self.bot_left.rotate_90(), self.top_right.rotate_90())
    }
    fn rotate_180(&self) -> Self {
        Rect::new(self.bot_left.rotate_180(), self.top_right.rotate_180())
    }
    fn rotate_270(&self) -> Self {
        Rect::new(self.bot_left.rotate_270(), self.top_right.rotate_270())
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        Rect::new(
            self.bot_left.rotate_90_around(pivot),
            self.top_right.rotate_90_around(pivot),
        )
    }
    fn rotate_180_around(&self, pivot: &V2) -> Self {
        Rect::new(
            self.bot_left.rotate_180_around(pivot),
            self.top_right.rotate_180_around(pivot),
        )
    }
    fn rotate_270_around(&self, pivot: &V2) -> Self {
        Rect::new(
            self.bot_left.rotate_270_around(pivot),
            self.top_right.rotate_270_around(pivot),
        )
    }
}
