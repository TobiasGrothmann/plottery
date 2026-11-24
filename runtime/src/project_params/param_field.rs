use std::collections::HashMap;

pub use anyhow::Error as ParamFieldError;
use plottery_lib::{
    Angle, BundledFont, ColorRgb, Easing, NoiseSettings, NoiseType, Paper, Ramp, Range, Rect,
    SystemFont, V2i, V2,
};
use plottery_runtime_macros::PlotteryParams;

use super::angle_unit::AngleUnit;

use super::curve_2d::Curve2D;
use super::curve_2d_norm::Curve2DNorm;
use super::image_path::ImagePath;
use super::percent::Percent;
use super::project_param::ProjectParam;
use super::project_param_enum::ProjectParamEnum;
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

/// `ParamField` for a type that one `ProjectParamValue` variant holds as it is.
macro_rules! value_param_field {
    ($ty:ty, $variant:ident, $default:expr) => {
        impl ParamField for $ty {
            fn default_param_value() -> ProjectParamValue {
                ProjectParamValue::$variant($default)
            }
            fn to_param_value(&self) -> ProjectParamValue {
                ProjectParamValue::$variant(Clone::clone(self))
            }
            fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
                match value {
                    ProjectParamValue::$variant(inner) => Ok(Clone::clone(inner)),
                    other => Err(param_type_mismatch(
                        &Self::default_param_value().type_name(),
                        other,
                    )),
                }
            }
        }
    };
}

value_param_field!(bool, Bool, false);
value_param_field!(String, String, String::new());
value_param_field!(Paper, Paper, Paper::default());
value_param_field!(V2, V2, V2::zero());
value_param_field!(V2i, V2i, V2i::default());
// A 10 cm square at the origin, as an empty rect would be invisible.
value_param_field!(Rect, Rect, Rect::new(V2::zero(), V2::xy(10.0)));
value_param_field!(Easing, Easing, Easing::default());
value_param_field!(ColorRgb, Color, ColorRgb::black());
// Not checked against the installed fonts: params have to load on a machine without the font.
value_param_field!(SystemFont, SystemFont, SystemFont::default());
value_param_field!(Curve2DNorm, Curve2DNorm, Curve2DNorm::default());
value_param_field!(Curve2D, Curve2D, Curve2D::default());

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
            ProjectParamValue::Angle { rad, .. } | ProjectParamValue::AngleRanged { rad, .. } => {
                Ok(Angle::from_rad(*rad))
            }
            other => Err(param_type_mismatch("angle", other)),
        }
    }
}

/// Bounds are angles, so the attribute names the unit: `#[range(Angle::from_degrees(0.0), ...)]`.
impl RangedParamField for Angle {
    type Bound = Angle;
    fn default_ranged_param_value(min: Angle, max: Angle) -> ProjectParamValue {
        Angle::zero().to_ranged_param_value(min, max)
    }
    fn to_ranged_param_value(&self, min: Angle, max: Angle) -> ProjectParamValue {
        let (min, max) = (min.to_rad(), max.to_rad());
        ProjectParamValue::AngleRanged {
            rad: self.to_rad().clamp(min, max),
            unit: AngleUnit::default(),
            min,
            max,
        }
    }
}

impl ParamField for Percent {
    fn default_param_value() -> ProjectParamValue {
        Self::default().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        self.to_ranged_param_value(0.0, 100.0)
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Percent { val, .. } => Ok(Percent::new(*val)),
            other => Err(param_type_mismatch("percent", other)),
        }
    }
}

impl RangedParamField for Percent {
    type Bound = f32;
    fn default_ranged_param_value(min: f32, max: f32) -> ProjectParamValue {
        Self::default().to_ranged_param_value(min, max)
    }
    fn to_ranged_param_value(&self, min: f32, max: f32) -> ProjectParamValue {
        ProjectParamValue::Percent {
            val: self.percent().clamp(min, max),
            min,
            max,
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

/// `ParamField` as an enum param for a lib enum, which can't derive `PlotteryEnum`.
macro_rules! lib_enum_param_field {
    ($ty:ident: $($variant:ident),+) => {
        impl ParamField for $ty {
            fn default_param_value() -> ProjectParamValue {
                Self::default().to_param_value()
            }
            fn to_param_value(&self) -> ProjectParamValue {
                let selected = match self {
                    $(Self::$variant => stringify!($variant),)+
                };
                ProjectParamValue::Enum(ProjectParamEnum::new(
                    selected,
                    &[$(stringify!($variant)),+],
                ))
            }
            fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
                let expected = concat!("enum ", stringify!($ty));
                match value {
                    ProjectParamValue::Enum(param_enum) => match param_enum.selected.as_str() {
                        $(stringify!($variant) => Ok(Self::$variant),)+
                        other => Err(param_type_mismatch(expected, value)
                            .context(format!("unknown variant '{}'", other))),
                    },
                    other => Err(param_type_mismatch(expected, other)),
                }
            }
        }
    };
}

lib_enum_param_field!(NoiseType: Perlin, Simplex, Worley);

/// The fields a [`NoiseSettings`] param has, as the lib type can't derive `PlotteryParams`.
#[derive(PlotteryParams)]
struct NoiseFields {
    noise_type: NoiseType,
    /// features per cm, or across what the project names
    frequency: f32,
    /// fractal layers, 1 is plain noise
    #[range(1, 8)]
    octaves: i32,
}

impl ParamField for NoiseSettings {
    fn default_param_value() -> ProjectParamValue {
        Self::default().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        NoiseFields {
            noise_type: self.noise_type,
            frequency: self.frequency,
            octaves: i32::try_from(self.octaves).unwrap_or(i32::MAX),
        }
        .to_param_value()
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        let fields = NoiseFields::from_param_value(value)?;
        Ok(NoiseSettings::new(
            fields.noise_type,
            fields.frequency,
            fields.octaves.max(1) as u32,
        ))
    }
}

/// The fields a [`Ramp`] param has, as the lib type can't derive `PlotteryParams`.
#[derive(PlotteryParams)]
struct RampFields {
    /// value at the start
    from: f32,
    /// value at the end
    #[value(1.0)]
    to: f32,
    /// how it gets from `from` to `to`
    easing: Easing,
}

impl From<Ramp> for RampFields {
    fn from(ramp: Ramp) -> Self {
        Self {
            from: ramp.from,
            to: ramp.to,
            easing: ramp.easing,
        }
    }
}

impl ParamField for Ramp {
    fn default_param_value() -> ProjectParamValue {
        Self::default().to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        RampFields::from(*self).to_param_value()
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        let fields = RampFields::from_param_value(value)?;
        Ok(Ramp::new(fields.from, fields.to).with_easing(fields.easing))
    }
}

/// `#[range]` turns `from` and `to` into sliders.
impl RangedParamField for Ramp {
    type Bound = f32;
    fn default_ranged_param_value(min: f32, max: f32) -> ProjectParamValue {
        Self::default().to_ranged_param_value(min, max)
    }
    fn to_ranged_param_value(&self, min: f32, max: f32) -> ProjectParamValue {
        let mut value = self.to_param_value();
        if let ProjectParamValue::Struct(fields) = &mut value {
            for field in &mut fields.fields {
                if let Ok(number) = field.value.get_f32() {
                    field.value = number.to_ranged_param_value(min, max);
                }
            }
        }
        value
    }
}

impl ParamField for BundledFont {
    fn default_param_value() -> ProjectParamValue {
        BundledFont::EmsReadability.to_param_value()
    }
    fn to_param_value(&self) -> ProjectParamValue {
        ProjectParamValue::Font(self.name().to_string())
    }
    fn from_param_value(value: &ProjectParamValue) -> Result<Self, ParamFieldError> {
        match value {
            ProjectParamValue::Font(name) => BundledFont::from_name(name)
                .ok_or_else(|| anyhow::anyhow!("unknown bundled font '{}'", name)),
            other => Err(param_type_mismatch("font", other)),
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
