#![doc = include_str!("../README.md")]

// Lets generated code use `::plottery_project::` paths inside this crate as well.
extern crate self as plottery_project;

mod cargo_project_template;
mod project;
mod project_config;
mod util;

mod project_main;
mod project_params;
mod project_pipeline;
mod project_scan;
mod project_scan_test;
mod project_test;
mod project_util;

pub use cargo_project_template::*;
pub use plottery_project_macros::*;
pub use project::*;
pub use project_config::*;
pub use project_main::*;
pub use project_params::*;
pub use project_pipeline::*;
pub use project_scan::*;
pub use project_util::*;

pub use util::*;
