use serde::{Deserialize, Serialize};

use super::project_param::ProjectParam;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectParamStruct {
    pub fields: Vec<ProjectParam>,
}

impl ProjectParamStruct {
    pub fn new(fields: Vec<ProjectParam>) -> Self {
        Self { fields }
    }
}
