use serde::{Deserialize, Serialize};

use super::project_param_value::ProjectParamValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectParamOptional {
    pub enabled: bool,
    pub value: Box<ProjectParamValue>,
}

impl ProjectParamOptional {
    pub fn new(enabled: bool, value: ProjectParamValue) -> Self {
        Self {
            enabled,
            value: Box::new(value),
        }
    }
}
