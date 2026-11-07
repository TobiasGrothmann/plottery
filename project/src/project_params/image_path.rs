use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
        let path = self
            .path
            .as_deref()
            .map(Path::new)
            .context("No image selected")?;
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        Ok(cwd.join(path))
    }

    /// Loads the image, see [`plottery_lib::Image::load`].
    pub fn load(&self) -> Result<plottery_lib::Image> {
        let path = self.resolve()?;
        if !path.exists() {
            anyhow::bail!("Image file '{}' does not exist", path.display());
        }
        plottery_lib::Image::load(path)
    }
}

/// Editor value of an [`ImagePath`] param: the current path plus recently used ones, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProjectParamImage {
    pub path: Option<String>,
    pub recent: Vec<String>,
}

impl ProjectParamImage {
    pub const RECENT_CAP: usize = 5;

    pub fn new(path: Option<String>) -> Self {
        Self {
            path,
            recent: Vec::new(),
        }
    }

    /// Sets a new path and remembers the previous one.
    pub fn set(&mut self, new: Option<String>) {
        if new == self.path {
            return;
        }
        let previous = std::mem::replace(&mut self.path, new);
        if let Some(previous) = previous {
            self.recent
                .retain(|p| *p != previous && Some(p) != self.path.as_ref());
            self.recent.insert(0, previous);
            self.recent.truncate(Self::RECENT_CAP);
        } else if let Some(current) = &self.path {
            self.recent.retain(|p| p != current);
        }
    }

    pub fn file_name(&self) -> Option<String> {
        self.path.as_deref().map(file_name_of)
    }
}

pub fn file_name_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
