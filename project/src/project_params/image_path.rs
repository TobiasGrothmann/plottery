use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::project_param_file::{existing_param_path, resolve_param_path};

/// Path to a raster image chosen in the editor.
///
/// Relative paths are resolved against the current directory, which the editor and CLI set to
/// the project directory, so images copied into `resources/images/` travel with the project.
/// ```no_run
/// # use plottery_project::ImagePath;
/// let image_path = ImagePath::new("resources/images/photo.jpg");
/// let image = image_path.load().expect("image is missing");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImagePath {
    pub path: Option<String>,
}

impl ImagePath {
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

    /// Absolute path of the image, without checking that it exists.
    pub fn resolve(&self) -> Result<PathBuf> {
        resolve_param_path(self.path.as_deref(), "image")
    }

    /// Loads the image, see [`plottery_lib::Image::load`].
    pub fn load(&self) -> Result<plottery_lib::Image> {
        plottery_lib::Image::load(existing_param_path(self.path.as_deref(), "image")?)
    }
}
