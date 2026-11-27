//! Values as `.plotp` format version 2 stored them, migrated to the current ones on reading.
//!
//! Version 3 removed `Noise` (variant 19), the noise settings before they were a struct, so every
//! later variant moved down one, also inside structs, options, lists and enum variants. A `Noise`
//! becomes the struct the noise settings are today. Everything else is the same as today: these
//! types mirror [`ProjectParamValue`] and its containers field by field, frozen at version 2. Don't
//! change them with the current ones.

use plottery_lib::{ColorRgb, Easing, NoiseSettings, NoiseType, Paper, Rect, SystemFont, V2i, V2};
use serde::{Deserialize, Serialize};

use super::angle_unit::AngleUnit;
use super::curve_2d::Curve2D;
use super::curve_2d_norm::Curve2DNorm;
use super::param_field::ParamField;
use super::project_param::ProjectParam;
use super::project_param_data_enum::{ProjectParamDataEnum, ProjectParamVariant};
use super::project_param_enum::ProjectParamEnum;
use super::project_param_file::ProjectParamFile;
use super::project_param_optional::ProjectParamOptional;
use super::project_param_struct::ProjectParamStruct;
use super::project_param_value::ProjectParamValue;
use super::project_param_vec::ProjectParamVec;
use super::seed::ProjectParamSeed;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ValueV2 {
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
    Struct(StructV2),
    Optional(OptionalV2),
    Vec(VecV2),
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
    /// A struct since version 3.
    Noise(NoiseV2),
    Easing(Easing),
    Color(ColorRgb),
    V2i(V2i),
    Svg(ProjectParamFile),
    Font(String),
    SystemFont(SystemFont),
    DataEnum(DataEnumV2),
    Percent {
        val: f32,
        min: f32,
        max: f32,
    },
    Rect(Rect),
    AngleRanged {
        rad: f32,
        unit: AngleUnit,
        min: f32,
        max: f32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ParamV2 {
    pub name: String,
    pub value: ValueV2,
    pub section: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StructV2 {
    pub fields: Vec<ParamV2>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OptionalV2 {
    pub enabled: bool,
    pub value: Box<ValueV2>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct VecV2 {
    pub item_prototype: Box<ValueV2>,
    pub items: Vec<ValueV2>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DataEnumV2 {
    pub selected: String,
    pub variants: Vec<VariantV2>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct VariantV2 {
    pub name: String,
    pub fields: Vec<ParamV2>,
}

/// `NoiseSettings` as `Noise` held it. The scale is gone, the code chooses it now.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NoiseV2 {
    pub noise_type: NoiseTypeV2,
    pub frequency: f32,
    pub octaves: u32,
    pub scale: NoiseScaleV2,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum NoiseTypeV2 {
    Perlin,
    Simplex,
    Worley,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum NoiseScaleV2 {
    PerCm,
    PerPaper,
}

impl ValueV2 {
    pub(crate) fn migrate(self) -> ProjectParamValue {
        use ProjectParamValue as V;
        match self {
            ValueV2::Float(val) => V::Float(val),
            ValueV2::FloatRanged { val, min, max } => V::FloatRanged { val, min, max },
            ValueV2::Int(val) => V::Int(val),
            ValueV2::IntRanged { val, min, max } => V::IntRanged { val, min, max },
            ValueV2::Bool(val) => V::Bool(val),
            ValueV2::Curve2DNorm(curve) => V::Curve2DNorm(curve),
            ValueV2::Curve2D(curve) => V::Curve2D(curve),
            ValueV2::Struct(param_struct) => {
                V::Struct(ProjectParamStruct::new(migrate_list(param_struct.fields)))
            }
            ValueV2::Optional(optional) => V::Optional(ProjectParamOptional::new(
                optional.enabled,
                optional.value.migrate(),
            )),
            ValueV2::Vec(param_vec) => V::Vec(ProjectParamVec::new(
                param_vec.item_prototype.migrate(),
                param_vec.items.into_iter().map(ValueV2::migrate).collect(),
            )),
            ValueV2::Enum(param_enum) => V::Enum(param_enum),
            ValueV2::String(text) => V::String(text),
            ValueV2::Paper(paper) => V::Paper(paper),
            ValueV2::RangeF32 { min, max, bounds } => V::RangeF32 { min, max, bounds },
            ValueV2::RangeI32 { min, max, bounds } => V::RangeI32 { min, max, bounds },
            ValueV2::Angle { rad, unit } => V::Angle { rad, unit },
            ValueV2::V2(v) => V::V2(v),
            ValueV2::Seed(seed) => V::Seed(seed),
            ValueV2::Image(file) => V::Image(file),
            ValueV2::Noise(noise) => {
                let noise_type = match noise.noise_type {
                    NoiseTypeV2::Perlin => NoiseType::Perlin,
                    NoiseTypeV2::Simplex => NoiseType::Simplex,
                    NoiseTypeV2::Worley => NoiseType::Worley,
                };
                NoiseSettings::new(noise_type, noise.frequency, noise.octaves).to_param_value()
            }
            ValueV2::Easing(easing) => V::Easing(easing),
            ValueV2::Color(color) => V::Color(color),
            ValueV2::V2i(v) => V::V2i(v),
            ValueV2::Svg(file) => V::Svg(file),
            ValueV2::Font(font) => V::Font(font),
            ValueV2::SystemFont(font) => V::SystemFont(font),
            ValueV2::DataEnum(data_enum) => V::DataEnum(ProjectParamDataEnum::new(
                &data_enum.selected,
                data_enum
                    .variants
                    .into_iter()
                    .map(|variant| {
                        ProjectParamVariant::new(&variant.name, migrate_list(variant.fields))
                    })
                    .collect(),
            )),
            ValueV2::Percent { val, min, max } => V::Percent { val, min, max },
            ValueV2::Rect(rect) => V::Rect(rect),
            ValueV2::AngleRanged {
                rad,
                unit,
                min,
                max,
            } => V::AngleRanged {
                rad,
                unit,
                min,
                max,
            },
        }
    }
}

fn migrate_list(params: Vec<ParamV2>) -> Vec<ProjectParam> {
    params
        .into_iter()
        .map(|param| ProjectParam {
            name: param.name,
            value: param.value.migrate(),
            section: param.section,
            description: param.description,
            no_pick: false,
        })
        .collect()
}
