use serde::{Deserialize, Serialize};

use super::project_param::ProjectParam;
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

    fn combine_lists(old: &[ProjectParam], new: &[ProjectParam]) -> Vec<ProjectParam> {
        let mut out_list = Vec::with_capacity(old.len() + new.len());

        for old_el in old {
            if let Some(new_el) = new.iter().find(|new_el| new_el.name == old_el.name) {
                match (&old_el.value, &new_el.value) {
                    (
                        ProjectParamValue::Struct(old_struct),
                        ProjectParamValue::Struct(new_struct),
                    ) => {
                        let merged_children =
                            Self::combine_lists(&old_struct.fields, &new_struct.fields);
                        out_list.push(ProjectParam::new(
                            old_el.name.as_str(),
                            ProjectParamValue::Struct(ProjectParamStruct::new(merged_children)),
                        ));
                    }
                    _ => {
                        if new.contains(old_el) {
                            out_list.push(old_el.clone());
                        }
                    }
                }
            }
        }

        for new_el in new {
            if !out_list.iter().any(|out_el| out_el.name == new_el.name) {
                out_list.push(new_el.clone());
            }
        }

        out_list
    }
}
