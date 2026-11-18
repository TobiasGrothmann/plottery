#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Containment {
    Full,
    Partial,
    None,
}

/// Expands to the `contains_shape` method of a type that has `contains_circle`, `contains_rect` and `contains_path`.
macro_rules! contains_shape_by_variant {
    () => {
        pub fn contains_shape(&self, other: &$crate::Shape) -> $crate::Containment {
            match other {
                $crate::Shape::Circle(c) => self.contains_circle(c),
                $crate::Shape::Rect(r) => self.contains_rect(r),
                $crate::Shape::Path(p) => self.contains_path(p),
            }
        }
    };
}
pub(crate) use contains_shape_by_variant;
