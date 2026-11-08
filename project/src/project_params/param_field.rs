use std::collections::HashMap;

pub use anyhow::Error as ParamFieldError;
use plottery_lib::{Angle, ColorRgb, NoiseSettings, Paper, Range, V2i, V2};

use super::angle_unit::AngleUnit;

use super::curve_2d::Curve2D;
use super::curve_2d_norm::Curve2DNorm;
use super::image_path::ImagePath;
use super::project_param::ProjectParam;
use super::project_param_file::ProjectParamFile;
use super::project_param_optional::ProjectParamOptional;
use super::project_param_value::ProjectParamValue;
use super::project_param_vec::ProjectParamVec;
use super::seed::{ProjectParamSeed, Seed};
use super::svg_path::SvgPath;

/// A type that can be used as a field of a `#[derive(PlotteryParams)]` struct.
pub trait ParamField: Sized {
    fn default_param_value() -> ProjectParamValue;
    fn to_param_value(&self) -> ProjectParamValue;
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError>;
}

/// A [`ParamField`] that accepts `#[range(min, max)]`.
pub trait RangedParamField: ParamField {
    type Bound: Copy;
    fn default_ranged_param_value(min: Self::Bound, max: Self::Bound) -> ProjectParamValue;
    /// Clamps `self` into `min..=max`.
    fn to_ranged_param_value(&self, min: Self::Bound, max: Self::Bound) -> ProjectParamValue;
}

pub fn param_type_mismatch(expected: &str, found: &ProjectParamValue) -> ParamFieldError {
    anyhow::anyhow!(
        "Expected param of type '{}' but found '{}'",
        expected,
        found.type_name()
    )
}

/// Looks up `name` in `params` and converts it. Used by the generated `try_new_from_map`.
pub fn param_field_from_map<T: ParamField>(
    params: &HashMap<String, ProjectParam>,
    name: &str,
) -> Result<T, ParamFieldError> {
    let param = params
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Field '{}' is missing in params.", name))?;
    T::from_param_value(&param.value).map_err(|e| e.context(format!("field '{}'", name)))
}

impl ParamField for f32 {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Float(0.0)
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Float(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        value.get_f32()
    }
}

impl RangedParamField for f32 {
    type Bound = f32;
    fn default_ranged_param_value(min: f32, max: f32) -> ProjectParamValue {
        0.0f32.to_ranged_param_value(min, max)
    }
    fn to_ranged_param_value(&self, min: f32, max: f32) -> ProjectParamValue {
        ProjectParamValue::FloatRanged {
            val: self.clamp(min, max),
            min,
            max,
        }
    }
}

impl ParamField for i32 {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Int(0)
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Int(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        value.get_i32()
    }
}

impl RangedParamField for i32 {
    type Bound = i32;
    fn default_ranged_param_value(min: i32, max: i32) -> ProjectParamValue {
        0i32.to_ranged_param_value(min, max)
    }
    fn to_ranged_param_value(&self, min: i32, max: i32) -> ProjectParamValue {
        ProjectParamValue::IntRanged {
            val: (*self).clamp(min, max),
            min,
            max,
        }
    }
}

impl ParamField for bool {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Bool(false)
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Bool(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        value.get_bool()
    }
}

impl ParamField for String {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::String(String::new())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::String(self.clone())
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::String(val) => Ok(val.clone()),
            other => Err(param_type_mismatch("string", other)),
        }
    }
}

impl ParamField for Paper {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Paper(Self::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Paper(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Paper(paper) => Ok(*paper),
            other => Err(param_type_mismatch("paper", other)),
        }
    }
}

impl ParamField for Range<f32> {
    fn default_param_value() -> ProjectParamValue {
        Self::default().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::RangeF32 {
            min: self.min,
            max: self.max,
            bounds: None,
        }
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::RangeF32 { min, max, .. } => Ok(Range::new(*min, *max)),
            other => Err(param_type_mismatch("range<f32>", other)),
        }
    }
}

impl RangedParamField for Range<f32> {
    type Bound = f32;
    fn default_ranged_param_value(lo: f32, hi: f32) -> ProjectParamValue {
        Range::new(lo, hi).to_ranged_param_value(lo, hi)
    }
    fn to_ranged_param_value(&self, lo: f32, hi: f32) -> ProjectParamValue {
        ProjectParamValue::RangeF32 {
            min: self.min.clamp(lo, hi),
            max: self.max.clamp(lo, hi),
            bounds: Some((lo, hi)),
        }
    }
}

impl ParamField for Range<i32> {
    fn default_param_value() -> ProjectParamValue {
        Self::default().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::RangeI32 {
            min: self.min,
            max: self.max,
            bounds: None,
        }
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::RangeI32 { min, max, .. } => Ok(Range::new(*min, *max)),
            other => Err(param_type_mismatch("range<i32>", other)),
        }
    }
}

impl RangedParamField for Range<i32> {
    type Bound = i32;
    fn default_ranged_param_value(lo: i32, hi: i32) -> ProjectParamValue {
        Range::new(lo, hi).to_ranged_param_value(lo, hi)
    }
    fn to_ranged_param_value(&self, lo: i32, hi: i32) -> ProjectParamValue {
        ProjectParamValue::RangeI32 {
            min: self.min.clamp(lo, hi),
            max: self.max.clamp(lo, hi),
            bounds: Some((lo, hi)),
        }
    }
}

impl ParamField for Angle {
    fn default_param_value() -> ProjectParamValue {
        Angle::zero().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Angle {
            rad: self.to_rad(),
            unit: AngleUnit::default(),
        }
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Angle { rad, .. } => Ok(Angle::from_rad(*rad)),
            other => Err(param_type_mismatch("angle", other)),
        }
    }
}

impl ParamField for V2 {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::V2(V2::zero())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::V2(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::V2(v) => Ok(*v),
            other => Err(param_type_mismatch("v2", other)),
        }
    }
}

impl ParamField for Seed {
    fn default_param_value() -> ProjectParamValue {
        Self::default().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Seed(ProjectParamSeed::new(self.value))
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Seed(seed) => Ok(Seed::new(seed.current)),
            other => Err(param_type_mismatch("seed", other)),
        }
    }
}

impl ParamField for ImagePath {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Image(ProjectParamFile::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Image(ProjectParamFile::new(self.path.clone()))
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Image(image) => Ok(ImagePath {
                path: image.path.clone(),
            }),
            other => Err(param_type_mismatch("image", other)),
        }
    }
}

impl ParamField for SvgPath {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Svg(ProjectParamFile::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Svg(ProjectParamFile::new(self.path.clone()))
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Svg(svg) => Ok(SvgPath {
                path: svg.path.clone(),
            }),
            other => Err(param_type_mismatch("svg", other)),
        }
    }
}

impl ParamField for NoiseSettings {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Noise(Self::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Noise(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Noise(noise) => Ok(*noise),
            other => Err(param_type_mismatch("noise", other)),
        }
    }
}

impl ParamField for ColorRgb {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Color(ColorRgb::black())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Color(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Color(color) => Ok(*color),
            other => Err(param_type_mismatch("color", other)),
        }
    }
}

impl ParamField for V2i {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::V2i(V2i::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::V2i(*self)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::V2i(v) => Ok(*v),
            other => Err(param_type_mismatch("v2i", other)),
        }
    }
}

impl ParamField for Curve2DNorm {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Curve2DNorm(Self::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Curve2DNorm(self.clone())
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Curve2DNorm(curve) => Ok(curve.clone()),
            other => Err(param_type_mismatch("curve2d_norm", other)),
        }
    }
}

impl ParamField for Curve2D {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Curve2D(Self::default())
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Curve2D(self.clone())
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Curve2D(curve) => Ok(curve.clone()),
            other => Err(param_type_mismatch("curve2d", other)),
        }
    }
}

impl<T: ParamField> ParamField for Option<T> {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Optional(ProjectParamOptional::new(false, T::default_param_value()))
    }
    fn to_param_value(&self) -> ProjectParamValue {
        let (enabled, inner) = match self {
            Some(value) => (true, value.to_param_value()),
            None => (false, T::default_param_value()),
        };
        ProjectParamValue::Optional(ProjectParamOptional::new(enabled, inner))
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Optional(optional) if optional.enabled => {
                T::from_param_value(&optional.value).map(Some)
            }
            ProjectParamValue::Optional(_) => Ok(None),
            other => Err(param_type_mismatch("option", other)),
        }
    }
}

impl<T: RangedParamField> RangedParamField for Option<T> {
    type Bound = T::Bound;
    fn default_ranged_param_value(min: T::Bound, max: T::Bound) -> ProjectParamValue {
        ProjectParamValue::Optional(ProjectParamOptional::new(
            false,
            T::default_ranged_param_value(min, max),
        ))
    }
    fn to_ranged_param_value(&self, min: T::Bound, max: T::Bound) -> ProjectParamValue {
        let (enabled, inner) = match self {
            Some(value) => (true, value.to_ranged_param_value(min, max)),
            None => (false, T::default_ranged_param_value(min, max)),
        };
        ProjectParamValue::Optional(ProjectParamOptional::new(enabled, inner))
    }
}

impl<T: ParamField> ParamField for Vec<T> {
    fn default_param_value() -> ProjectParamValue {
        ProjectParamValue::Vec(ProjectParamVec::new(T::default_param_value(), vec![]))
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Vec(ProjectParamVec::new(
            T::default_param_value(),
            self.iter().map(T::to_param_value).collect(),
        ))
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Vec(vec_value) => {
                vec_value.items.iter().map(T::from_param_value).collect()
            }
            other => Err(param_type_mismatch("vec", other)),
        }
    }
}
