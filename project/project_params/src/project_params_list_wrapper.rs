use serde::{Deserialize, Serialize};

use crate::ProjectParam;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectParamsListWrapper {
    pub list: Vec<ProjectParam>,
}

impl ProjectParamsListWrapper {
    pub fn new(list: Vec<ProjectParam>) -> Self {
        Self { list }
    }
}
