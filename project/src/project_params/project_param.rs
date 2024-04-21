use serde::{Deserialize, Serialize};

use crate::ProjectParamValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParam {
    pub name: String,
    pub value: ProjectParamValue,
}

impl ProjectParam {
    pub fn new(name: &str, value: ProjectParamValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}
