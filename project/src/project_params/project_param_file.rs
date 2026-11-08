use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Editor value of a file param (e.g. [`crate::ImagePath`]): the current path plus recently used
/// ones, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProjectParamFile {
    pub path: Option<String>,
    pub recent: Vec<String>,
}

impl ProjectParamFile {
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

/// Absolute path of a stored file param path, relative ones resolved against the current dir.
pub(crate) fn resolve_param_path(path: Option<&str>, kind: &str) -> Result<PathBuf> {
    let path = path
        .map(Path::new)
        .with_context(|| format!("No {kind} selected"))?;
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    Ok(cwd.join(path))
}

/// Like [`resolve_param_path`], but fails if the file does not exist.
pub(crate) fn existing_param_path(path: Option<&str>, kind: &str) -> Result<PathBuf> {
    let path = resolve_param_path(path, kind)?;
    if !path.exists() {
        anyhow::bail!("The {kind} file '{}' does not exist", path.display());
    }
    Ok(path)
}
