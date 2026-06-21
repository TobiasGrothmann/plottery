use serde::{Deserialize, Serialize};

use super::project_param_value::ProjectParamValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectParamVec {
    pub item_prototype: Box<ProjectParamValue>,
    pub items: Vec<ProjectParamValue>,
}

impl ProjectParamVec {
    pub fn new(item_prototype: ProjectParamValue, items: Vec<ProjectParamValue>) -> Self {
        Self {
            item_prototype: Box::new(item_prototype),
            items,
        }
    }
}
