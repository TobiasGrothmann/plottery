//! # Plottery project
//!
//! This library contains all necessary functionality to create and manage a **Plottery** project.
//!
//! A **Plottery** project is a wrapper around a cargo package:
//!
//! <img alt="folder structure" src="https://github.com/user-attachments/assets/d0066b16-5b75-4563-b625-88cc7d206a77" />

mod cargo_project_template;
mod project;
mod project_config;
mod util;

mod project_main;
mod project_params;
mod project_test;
mod project_util;

pub use cargo_project_template::*;
pub use plottery_project_macros::*;
pub use project::*;
pub use project_config::*;
pub use project_main::*;
pub use project_params::*;
pub use project_util::*;

pub use util::*;
