use serde::{Deserialize, Serialize};

use super::ColorRgb;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Inheritable<T>
where
    T: Clone,
{
    Inherit,
    Specified(T),
}

impl<T> Inheritable<T>
where
    T: Clone,
{
    #[must_use]
    pub fn overwrite_with(&self, child: &Self) -> Self {
        match child {
            Inheritable::Inherit => self.clone(),
            Inheritable::Specified(child_value) => Inheritable::Specified(child_value.clone()),
        }
    }

    pub fn unwrap(&self) -> T {
        match self {
            Inheritable::Inherit => panic!("Inheritable::unwrap() called on Inherit variant"),
            Inheritable::Specified(value) => value.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayerPropsInheritable {
    pub color: Inheritable<ColorRgb>,
    pub pen_width: Inheritable<f32>,
}

impl LayerPropsInheritable {
    pub fn inherit_all() -> Self {
        Self {
            color: Inheritable::Inherit,
            pen_width: Inheritable::Inherit,
        }
    }

    #[must_use]
    pub fn with_color(&self, color: ColorRgb) -> Self {
        Self {
            color: Inheritable::Specified(color),
            pen_width: self.pen_width,
        }
    }
    #[must_use]
    pub fn with_pen_width(&self, pen_width: f32) -> Self {
        Self {
            color: self.color,
            pen_width: Inheritable::Specified(pen_width),
        }
    }
}

impl LayerPropsInheritable {
    #[must_use]
    pub fn overwrite_with(&self, child: &Inheritable<Self>) -> Self {
        match child {
            Inheritable::Inherit => self.clone(),
            Inheritable::Specified(child_props) => Self {
                color: self.color.overwrite_with(&child_props.color),
                pen_width: self.pen_width.overwrite_with(&child_props.pen_width),
            },
        }
    }
}

impl Default for LayerPropsInheritable {
    fn default() -> Self {
        Self {
            color: Inheritable::Specified(ColorRgb::black()),
            pen_width: Inheritable::Specified(0.05),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LayerProps {
    pub name: Option<String>,
}

impl LayerProps {
    #[must_use]
    pub fn with_name(&self, name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
        }
    }
}
