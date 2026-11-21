use serde::{Deserialize, Serialize};

use super::param_field::ParamField;
use super::project_param::ProjectParam;
use super::project_param_data_enum::{ProjectParamDataEnum, ProjectParamVariant};
use super::project_param_enum::ProjectParamEnum;
use super::project_param_optional::ProjectParamOptional;
use super::project_param_struct::ProjectParamStruct;
use super::project_param_value::ProjectParamValue;
use super::project_param_vec::ProjectParamVec;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectParamsListWrapper {
    pub list: Vec<ProjectParam>,
    /// The project's defaults for `list`. Not saved, empty until `new_combined` fills it.
    #[serde(skip)]
    pub defaults: Vec<ProjectParam>,
}

impl ProjectParamsListWrapper {
    pub fn new(list: Vec<ProjectParam>) -> Self {
        Self {
            list,
            defaults: Vec::new(),
        }
    }
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn new_combined(old: &[ProjectParam], new: &[ProjectParam]) -> Self {
        Self {
            list: Self::combine_lists(old, new),
            defaults: new.to_vec(),
        }
    }

    /// Follows the order of `new` (the struct definition), keeping old values of the same type, clamped into the new `#[range]`.
    fn combine_lists(old: &[ProjectParam], new: &[ProjectParam]) -> Vec<ProjectParam> {
        new.iter()
            .map(|new_el| {
                old.iter()
                    .find(|old_el| old_el.name == new_el.name)
                    .and_then(|old_el| Self::combine_values(&old_el.value, &new_el.value))
                    .map(|value| ProjectParam {
                        value,
                        ..new_el.clone()
                    })
                    .unwrap_or_else(|| new_el.clone())
            })
            .collect()
    }

    fn combine_values(
        old: &ProjectParamValue,
        new: &ProjectParamValue,
    ) -> Option<ProjectParamValue> {
        match (old, new) {
            (ProjectParamValue::Struct(old_struct), ProjectParamValue::Struct(new_struct)) => {
                let merged_children = Self::combine_lists(&old_struct.fields, &new_struct.fields);
                Some(ProjectParamValue::Struct(ProjectParamStruct::new(
                    merged_children,
                )))
            }
            (
                ProjectParamValue::Optional(old_optional),
                ProjectParamValue::Optional(new_optional),
            ) => {
                let merged_inner = Self::combine_values(&old_optional.value, &new_optional.value)?;
                Some(ProjectParamValue::Optional(ProjectParamOptional::new(
                    old_optional.enabled,
                    merged_inner,
                )))
            }
            (
                ProjectParamValue::Enum(ProjectParamEnum { selected, .. })
                | ProjectParamValue::DataEnum(ProjectParamDataEnum { selected, .. }),
                ProjectParamValue::Enum(new_enum),
            ) => {
                let selected = if new_enum.variants.contains(selected) {
                    selected.clone()
                } else {
                    new_enum.selected.clone()
                };
                Some(ProjectParamValue::Enum(ProjectParamEnum {
                    selected,
                    variants: new_enum.variants.clone(),
                }))
            }
            (ProjectParamValue::Enum(old_enum), ProjectParamValue::DataEnum(new_enum)) => {
                let mut merged = new_enum.clone();
                if merged.variant(&old_enum.selected).is_some() {
                    merged.selected = old_enum.selected.clone();
                }
                Some(ProjectParamValue::DataEnum(merged))
            }
            (ProjectParamValue::DataEnum(old_enum), ProjectParamValue::DataEnum(new_enum)) => {
                let variants = new_enum
                    .variants
                    .iter()
                    .map(|new_variant| {
                        let fields = match old_enum.variant(&new_variant.name) {
                            Some(old_variant) => {
                                Self::combine_lists(&old_variant.fields, &new_variant.fields)
                            }
                            None => new_variant.fields.clone(),
                        };
                        ProjectParamVariant::new(&new_variant.name, fields)
                    })
                    .collect();
                let selected = match new_enum.variant(&old_enum.selected) {
                    Some(_) => &old_enum.selected,
                    None => &new_enum.selected,
                };
                Some(ProjectParamValue::DataEnum(ProjectParamDataEnum::new(
                    selected, variants,
                )))
            }
            // Noise was a value of its own before it became a struct.
            (ProjectParamValue::Noise(old_noise), ProjectParamValue::Struct(_)) => {
                Self::combine_values(&old_noise.to_param_value(), new)
            }
            (ProjectParamValue::Font(old_name), ProjectParamValue::Font(_)) => {
                let known = plottery_lib::BundledFont::from_name(old_name).is_some();
                Some(if known { old.clone() } else { new.clone() })
            }
            (
                ProjectParamValue::Float(_) | ProjectParamValue::FloatRanged { .. },
                ProjectParamValue::FloatRanged { min, max, .. },
            ) => Some(ProjectParamValue::FloatRanged {
                val: old.get_f32().ok()?.clamp(*min, *max),
                min: *min,
                max: *max,
            }),
            (ProjectParamValue::FloatRanged { val, .. }, ProjectParamValue::Float(_)) => {
                Some(ProjectParamValue::Float(*val))
            }
            (
                ProjectParamValue::Int(_) | ProjectParamValue::IntRanged { .. },
                ProjectParamValue::IntRanged { min, max, .. },
            ) => Some(ProjectParamValue::IntRanged {
                val: old.get_i32().ok()?.clamp(*min, *max),
                min: *min,
                max: *max,
            }),
            (ProjectParamValue::IntRanged { val, .. }, ProjectParamValue::Int(_)) => {
                Some(ProjectParamValue::Int(*val))
            }
            (
                ProjectParamValue::Percent { val, .. },
                ProjectParamValue::Percent { min, max, .. },
            ) => Some(ProjectParamValue::Percent {
                val: val.clamp(*min, *max),
                min: *min,
                max: *max,
            }),
            (
                ProjectParamValue::RangeF32 { min, max, .. },
                ProjectParamValue::RangeF32 { bounds, .. },
            ) => {
                let clamp = |v: f32| bounds.map_or(v, |(low, high)| v.clamp(low, high));
                Some(ProjectParamValue::RangeF32 {
                    min: clamp(*min),
                    max: clamp(*max),
                    bounds: *bounds,
                })
            }
            (
                ProjectParamValue::RangeI32 { min, max, .. },
                ProjectParamValue::RangeI32 { bounds, .. },
            ) => {
                let clamp = |v: i32| bounds.map_or(v, |(low, high)| v.clamp(low, high));
                Some(ProjectParamValue::RangeI32 {
                    min: clamp(*min),
                    max: clamp(*max),
                    bounds: *bounds,
                })
            }
            (ProjectParamValue::Vec(old_vec), ProjectParamValue::Vec(new_vec)) => {
                let prototype = new_vec.item_prototype.as_ref();
                // ProjectParam's equality compares the type only.
                if ProjectParam::new("", old.clone()) != ProjectParam::new("", new.clone()) {
                    return None;
                }
                let items = old_vec
                    .items
                    .iter()
                    .map(|item| Self::combine_values(item, prototype))
                    .collect::<Option<Vec<_>>>()?;
                Some(ProjectParamValue::Vec(ProjectParamVec::new(
                    prototype.clone(),
                    items,
                )))
            }
            _ => (std::mem::discriminant(old) == std::mem::discriminant(new)).then(|| old.clone()),
        }
    }
}
