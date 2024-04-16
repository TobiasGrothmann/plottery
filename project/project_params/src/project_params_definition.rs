use std::collections::HashMap;

use crate::ProjectParam;

pub trait PlotteryParamsDefinition {
    fn get_params(&self) -> Vec<ProjectParam>;
    fn new_from_list(params: &HashMap<String, ProjectParam>) -> Self;
}
