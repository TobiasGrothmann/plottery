use anyhow::Result;
use plottery_lib::{ColorRgb, Easing, Paper, Rect, SystemFont, V2i, V2};

use super::angle_unit::AngleUnit;
use super::param_field::param_type_mismatch;
use super::project_param::ProjectParam;
use super::project_param_file::ProjectParamFile;
use super::seed::ProjectParamSeed;
use serde::{Deserialize, Serialize};

use super::curve_2d::Curve2D;
use super::curve_2d_norm::Curve2DNorm;
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
    /// An `Angle` with `#[range]`, all in radians.
    AngleRanged {
        rad: f32,
        unit: AngleUnit,
        min: f32,
        max: f32,
    },
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
            (
                Self::Angle { .. } | Self::AngleRanged { .. },
                Self::Angle { .. } | Self::AngleRanged { .. },
            ) => true,
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
            ProjectParamValue::Float(val) | ProjectParamValue::FloatRanged { val, .. } => Ok(*val),
            other => Err(param_type_mismatch("f32", other)),
        }
    }

    pub fn get_i32(&self) -> Result<i32> {
        match self {
            ProjectParamValue::Int(val) | ProjectParamValue::IntRanged { val, .. } => Ok(*val),
            other => Err(param_type_mismatch("i32", other)),
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            ProjectParamValue::Float(_) => "f32".to_string(),
            ProjectParamValue::FloatRanged { .. } => "f32".to_string(),
            ProjectParamValue::Int(_) => "i32".to_string(),
            ProjectParamValue::IntRanged { .. } => "i32".to_string(),
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
            ProjectParamValue::Angle { .. } | ProjectParamValue::AngleRanged { .. } => {
                "angle".to_string()
            }
            ProjectParamValue::V2(_) => "v2".to_string(),
            ProjectParamValue::Seed(_) => "seed".to_string(),
            ProjectParamValue::Image(_) => "image".to_string(),
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
}
