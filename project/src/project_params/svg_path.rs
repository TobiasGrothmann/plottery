use std::path::PathBuf;

use anyhow::Result;
use plottery_lib::Layer;
use serde::{Deserialize, Serialize};

use super::project_param_file::{existing_param_path, resolve_param_path};

/// Path to an svg file chosen in the editor.
///
/// Relative paths are resolved against the current directory, which the editor and CLI set to
/// the project directory, so svgs copied into `resources/svgs/` travel with the project.
/// ```no_run
/// # use plottery_project::SvgPath;
/// let svg_path = SvgPath::new("resources/svgs/logo.svg");
/// let layer = svg_path.load().expect("svg is missing");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SvgPath {
    pub path: Option<String>,
}

impl SvgPath {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: Some(path.into()),
        }
    }
    pub fn none() -> Self {
        Self { path: None }
    }
    pub fn is_set(&self) -> bool {
        self.path.is_some()
    }

    /// Absolute path of the svg, without checking that it exists.
    pub fn resolve(&self) -> Result<PathBuf> {
        resolve_param_path(self.path.as_deref(), "svg")
    }

    /// Loads the svg as a layer, see [`Layer::new_from_svg`].
    pub fn load(&self) -> Result<Layer> {
        Layer::new_from_svg(&existing_param_path(self.path.as_deref(), "svg")?)
    }
}
