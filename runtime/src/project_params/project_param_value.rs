use anyhow::Result;
use plottery_lib::{ColorRgb, Easing, FontStyle, NoiseSettings, Paper, Rect, SystemFont, V2i, V2};

use super::angle_unit::AngleUnit;
use super::project_param::ProjectParam;
use super::project_param_file::ProjectParamFile;
use super::seed::ProjectParamSeed;
use serde::{Deserialize, Serialize};

pub use super::curve_2d::Curve2D;
pub use super::curve_2d_norm::Curve2DNorm;
use super::project_param_data_enum::ProjectParamDataEnum;
use super::project_param_enum::ProjectParamEnum;
use super::project_param_optional::ProjectParamOptional;
use super::project_param_struct::ProjectParamStruct;
use super::project_param_vec::ProjectParamVec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectParamValue {
    Float(f32),
    FloatRanged {
        val: f32,
        min: f32,
        max: f32,
    },
    Int(i32),
    IntRanged {
        val: i32,
        min: i32,
        max: i32,
    },
    Bool(bool),
    Curve2DNorm(Curve2DNorm),
    Curve2D(Curve2D),
    Struct(ProjectParamStruct),
    Optional(ProjectParamOptional),
    Vec(ProjectParamVec),
    Enum(ProjectParamEnum),
    String(String),
    Paper(Paper),
    RangeF32 {
        min: f32,
        max: f32,
        bounds: Option<(f32, f32)>,
    },
    RangeI32 {
        min: i32,
        max: i32,
        bounds: Option<(i32, i32)>,
    },
    Angle {
        rad: f32,
        unit: AngleUnit,
    },
    V2(V2),
    Seed(ProjectParamSeed),
    Image(ProjectParamFile),
    /// What a `NoiseSettings` param was before it became a `Struct`, only read from saved files.
    Noise(NoiseSettings),
    Easing(Easing),
    Color(ColorRgb),
    V2i(V2i),
    Svg(ProjectParamFile),
    /// `BundledFont` variant name.
    Font(String),
    SystemFont(SystemFont),
    /// A `PlotteryEnum` whose variants have fields.
    DataEnum(ProjectParamDataEnum),
    /// `val` and its bounds in percent.
    Percent {
        val: f32,
        min: f32,
        max: f32,
    },
    Rect(Rect),
}

fn same_schema_list(left: &[ProjectParam], right: &[ProjectParam]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(l, r)| l.same_schema(r))
}

impl ProjectParamValue {
    /// Same type, whatever the values and `#[range]`s.
    pub fn same_schema(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Float(_) | Self::FloatRanged { .. },
                Self::Float(_) | Self::FloatRanged { .. },
            ) => true,
            (Self::Int(_) | Self::IntRanged { .. }, Self::Int(_) | Self::IntRanged { .. }) => true,
            (Self::Struct(left), Self::Struct(right)) => {
                same_schema_list(&left.fields, &right.fields)
            }
            (Self::Optional(left), Self::Optional(right)) => left.value.same_schema(&right.value),
            (Self::Vec(left), Self::Vec(right)) => {
                left.item_prototype.same_schema(&right.item_prototype)
            }
            (Self::Enum(left), Self::Enum(right)) => left.variants == right.variants,
            (Self::DataEnum(left), Self::DataEnum(right)) => {
                left.variants.len() == right.variants.len()
                    && left
                        .variants
                        .iter()
                        .zip(&right.variants)
                        .all(|(l, r)| l.name == r.name && same_schema_list(&l.fields, &r.fields))
            }
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }

    pub fn get_f32(&self) -> Result<f32> {
        match self {
            ProjectParamValue::Float(val) => Ok(*val),
            ProjectParamValue::FloatRanged {
                val,
                min: _,
                max: _,
            } => Ok(*val),
            _ => Err(anyhow::anyhow!(
                "Failed to get_f32 - Expected value to be of type 'float' but found {}",
                self.type_name()
            )),
        }
    }
    pub fn get_i32(&self) -> Result<i32> {
        match self {
            ProjectParamValue::Int(val) => Ok(*val),
            ProjectParamValue::IntRanged {
                val,
                min: _,
                max: _,
            } => Ok(*val),
            _ => Err(anyhow::anyhow!(
                "Failed to get_i32 - Expected value to be of type 'int' but found {}",
                self.type_name()
            )),
        }
    }
    pub fn type_name(&self) -> String {
        match self {
            ProjectParamValue::Float(_) => "f32".to_string(),
            ProjectParamValue::FloatRanged { .. } => "f32 (ranged)".to_string(),
            ProjectParamValue::Int(_) => "i32".to_string(),
            ProjectParamValue::IntRanged { .. } => "i32 (ranged)".to_string(),
            ProjectParamValue::Bool(_) => "bool".to_string(),
            ProjectParamValue::Curve2DNorm(_) => "curve2d_norm".to_string(),
            ProjectParamValue::Curve2D(_) => "curve2d".to_string(),
            ProjectParamValue::Struct(_) => "struct".to_string(),
            ProjectParamValue::Optional(optional) => {
                format!("option<{}>", optional.value.type_name())
            }
            ProjectParamValue::Vec(vec_value) => {
                format!("vec<{}>", vec_value.item_prototype.type_name())
            }
            ProjectParamValue::Enum(_) => "enum".to_string(),
            ProjectParamValue::String(_) => "string".to_string(),
            ProjectParamValue::Paper(_) => "paper".to_string(),
            ProjectParamValue::RangeF32 { .. } => "range<f32>".to_string(),
            ProjectParamValue::RangeI32 { .. } => "range<i32>".to_string(),
            ProjectParamValue::Angle { .. } => "angle".to_string(),
            ProjectParamValue::V2(_) => "v2".to_string(),
            ProjectParamValue::Seed(_) => "seed".to_string(),
            ProjectParamValue::Image(_) => "image".to_string(),
            ProjectParamValue::Noise(_) => "noise".to_string(),
            ProjectParamValue::Easing(_) => "easing".to_string(),
            ProjectParamValue::Color(_) => "color".to_string(),
            ProjectParamValue::V2i(_) => "v2i".to_string(),
            ProjectParamValue::Svg(_) => "svg".to_string(),
            ProjectParamValue::Font(_) => "font".to_string(),
            ProjectParamValue::SystemFont(_) => "system_font".to_string(),
            ProjectParamValue::DataEnum(_) => "enum".to_string(),
            ProjectParamValue::Percent { .. } => "percent".to_string(),
            ProjectParamValue::Rect(_) => "rect".to_string(),
        }
    }

    pub fn value_as_string(&self) -> String {
        match self {
            ProjectParamValue::Float(val) => val.to_string(),
            ProjectParamValue::FloatRanged { val, .. } => val.to_string(),
            ProjectParamValue::Int(val) => val.to_string(),
            ProjectParamValue::IntRanged { val, .. } => val.to_string(),
            ProjectParamValue::Bool(val) => val.to_string(),
            ProjectParamValue::Curve2DNorm(graph) => format!("{} points", graph.len()),
            ProjectParamValue::Curve2D(curve) => format!("{} points", curve.len()),
            ProjectParamValue::Struct(param_struct) => {
                format!("{} fields", param_struct.fields.len())
            }
            ProjectParamValue::Optional(optional) => {
                if optional.enabled {
                    format!("Some({})", optional.value.value_as_string())
                } else {
                    "None".to_string()
                }
            }
            ProjectParamValue::Vec(vec_value) => {
                format!("{} items", vec_value.items.len())
            }
            ProjectParamValue::Enum(param_enum) => param_enum.selected.clone(),
            ProjectParamValue::String(val) => val.clone(),
            ProjectParamValue::Paper(paper) => paper.to_string(),
            ProjectParamValue::RangeF32 { min, max, .. } => format!("{min} – {max}"),
            ProjectParamValue::RangeI32 { min, max, .. } => format!("{min} – {max}"),
            ProjectParamValue::Angle { rad, unit } => format!(
                "{} {}",
                unit.from_angle(plottery_lib::Angle::from_rad(*rad)),
                unit.short_name()
            ),
            ProjectParamValue::V2(v) => format!("({}, {})", v.x, v.y),
            ProjectParamValue::Seed(seed) => seed.current.to_string(),
            ProjectParamValue::Image(file) | ProjectParamValue::Svg(file) => {
                file.file_name().unwrap_or_else(|| "none".to_string())
            }
            ProjectParamValue::Noise(noise) => format!(
                "{:?} f={} o={} {:?}",
                noise.noise_type, noise.frequency, noise.octaves, noise.scale
            ),
            ProjectParamValue::Easing(easing) => {
                format!("{:?} {:.2}", easing.kind, easing.amount).to_lowercase()
            }
            ProjectParamValue::Color(color) => color.hex(),
            ProjectParamValue::V2i(v) => format!("({}, {})", v.x, v.y),
            ProjectParamValue::Font(name) => name.clone(),
            ProjectParamValue::SystemFont(font) => {
                let weight = (font.style.weight != FontStyle::default().weight)
                    .then(|| format!(":{}", font.style.weight));
                let italic = font.style.italic.then(|| ":italic".to_string());
                [Some(font.family.clone()), weight, italic]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            ProjectParamValue::DataEnum(data_enum) => data_enum.selected.clone(),
            ProjectParamValue::Percent { val, .. } => format!("{val}%"),
            ProjectParamValue::Rect(rect) => format!(
                "({}, {}) – ({}, {})",
                rect.bl().x,
                rect.bl().y,
                rect.tr().x,
                rect.tr().y
            ),
        }
    }
}
