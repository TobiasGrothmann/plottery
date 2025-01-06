use serde::{Deserialize, Serialize};

use crate::ProjectParam;

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
        let mut out_list = Vec::new();
        for old_el in old.iter() {
            if new.contains(old_el) {
                out_list.push(old_el.clone());
            }
        }
        for new_el in new.iter() {
            if !out_list.contains(new_el) {
                out_list.push(new_el.clone());
            }
        }

        Self::new(out_list)
    }
}
