use std::collections::HashMap;

use super::param_field::ParamFieldError;
use super::project_param::ProjectParam;

/// Implemented by `#[derive(PlotteryParams)]`.
pub trait PlotteryParams: Sized {
    fn param_defaults_list() -> Vec<ProjectParam>;
    fn to_params_list(&self) -> Vec<ProjectParam>;
    fn try_new_from_map(params: &HashMap<String, ProjectParam>) -> Result<Self, ParamFieldError>;

    fn try_new_from_list(params: Vec<ProjectParam>) -> Result<Self, ParamFieldError> {
        let map = params
            .into_iter()
            .map(|param| (param.name.clone(), param))
            .collect();
        Self::try_new_from_map(&map)
    }

    fn new_from_map(params: &HashMap<String, ProjectParam>) -> Self {
        Self::try_new_from_map(params).unwrap_or_else(|e| panic!("Failed to read params: {:#}", e))
    }
    fn new_from_list(params: Vec<ProjectParam>) -> Self {
        Self::try_new_from_list(params).unwrap_or_else(|e| panic!("Failed to read params: {:#}", e))
    }
    fn new_with_defaults() -> Self {
        Self::new_from_list(Self::param_defaults_list())
    }
}
