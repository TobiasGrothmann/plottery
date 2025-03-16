use crate::V2;

pub trait Translate {
    fn translate(&self, dist: V2) -> Self;
    fn translate_mut(&mut self, dist: V2);
}
