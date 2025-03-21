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
    pub pen_width_cm: Inheritable<f32>,
}

impl LayerPropsInheritable {
    pub fn inherit_all() -> Self {
        Self {
            color: Inheritable::Inherit,
            pen_width_cm: Inheritable::Inherit,
        }
    }

    pub fn with_color(&self, color: ColorRgb) -> Self {
        Self {
            color: Inheritable::Specified(color),
            pen_width_cm: self.pen_width_cm,
        }
    }
    pub fn with_pen_width_cm(&self, pen_width_cm: f32) -> Self {
        Self {
            color: self.color,
            pen_width_cm: Inheritable::Specified(pen_width_cm),
        }
    }
}

impl LayerPropsInheritable {
    pub fn overwrite_with(&self, child: &Inheritable<Self>) -> Self {
        match child {
            Inheritable::Inherit => self.clone(),
            Inheritable::Specified(child_props) => Self {
                color: self.color.overwrite_with(&child_props.color),
                pen_width_cm: self.pen_width_cm.overwrite_with(&child_props.pen_width_cm),
            },
        }
    }
}

impl Default for LayerPropsInheritable {
    fn default() -> Self {
        Self {
            color: Inheritable::Specified(ColorRgb::black()),
            pen_width_cm: Inheritable::Specified(0.05),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LayerProps {
    pub name: Option<String>,
}

impl LayerProps {
    pub fn with_name(&self, name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
        }
    }
}
