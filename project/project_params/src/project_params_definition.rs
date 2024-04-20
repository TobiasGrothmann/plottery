use std::collections::HashMap;

use crate::ProjectParam;

pub trait PlotteryParamsDefinition {
    fn param_defaults_list() -> Vec<ProjectParam>;

    fn new_with_defaults() -> Self;
    fn new_from_map(params: &HashMap<String, ProjectParam>) -> Self;
    fn new_from_list(parms: Vec<ProjectParam>) -> Self;
}
