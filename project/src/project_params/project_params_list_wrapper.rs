use serde::{Deserialize, Serialize};

use super::project_param::ProjectParam;
use super::project_param_enum::ProjectParamEnum;
use super::project_param_optional::ProjectParamOptional;
use super::project_param_struct::ProjectParamStruct;
use super::project_param_value::ProjectParamValue;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectParamsListWrapper {
    pub list: Vec<ProjectParam>,
}

impl ProjectParamsListWrapper {
    pub fn new(list: Vec<ProjectParam>) -> Self {
        Self { list }
    }
    pub fn empty() -> Self {
        Self { list: Vec::new() }
    }

    pub fn new_combined(old: &[ProjectParam], new: &[ProjectParam]) -> Self {
        Self::new(Self::combine_lists(old, new))
    }

    /// Follows the order of `new` (the struct definition), keeping old values where the schema still matches.
    fn combine_lists(old: &[ProjectParam], new: &[ProjectParam]) -> Vec<ProjectParam> {
        new.iter()
            .map(|new_el| {
                old.iter()
                    .find(|old_el| old_el.name == new_el.name)
                    .and_then(|old_el| Self::combine_values(&old_el.value, &new_el.value))
                    .map(|merged| ProjectParam::new(new_el.name.as_str(), merged))
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
            (ProjectParamValue::Enum(old_enum), ProjectParamValue::Enum(new_enum)) => {
                let selected = if new_enum.variants.contains(&old_enum.selected) {
                    old_enum.selected.clone()
                } else {
                    new_enum.selected.clone()
                };
                Some(ProjectParamValue::Enum(ProjectParamEnum {
                    selected,
                    variants: new_enum.variants.clone(),
                }))
            }
            _ => {
                let old_schema = ProjectParam::new("__schema", old.clone());
                let new_schema = ProjectParam::new("__schema", new.clone());
                if old_schema == new_schema {
                    Some(old.clone())
                } else {
                    None
                }
            }
        }
    }
}
