mod cargo_project_template;
mod project;
mod project_config;

mod project_test;
mod project_util;

pub use cargo_project_template::{generate_cargo_project_to_disk, LibSource};
pub use plottery_project_macros::PlotteryParamsDefinition;
pub use plottery_project_params::{
    PlotteryParamsDefinition, ProjectParam, ProjectParamValue, ProjectParamsListWrapper,
};
pub use project::Project;
pub use project_config::ProjectConfig;
pub use project_util::{
    build_cargo_project_async, read_object_from_stdout, run_project_executable_async,
};
