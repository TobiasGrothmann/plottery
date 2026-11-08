use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use plottery_project::Project;

/// Loads the project at `path` (a `.plottery` file or a folder), by default from the current folder.
pub fn find_project(path: Option<PathBuf>) -> Result<Project> {
    let file = match path {
        Some(path) if path.is_file() => path,
        Some(path) if path.is_dir() => find_project_file(&path)?,
        Some(path) => bail!("'{}' does not exist", path.display()),
        None => find_project_file(&std::env::current_dir()?)?,
    };
    Project::load_from_file(file.clone())
        .with_context(|| format!("Failed to load project '{}'", file.display()))
}

/// The single `.plottery` file in `start` or the closest parent folder that has one.
pub fn find_project_file(start: &Path) -> Result<PathBuf> {
    let start = std::path::absolute(start)?;
    for dir in start.ancestors() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "plottery"))
            .collect();
        files.sort();
        match files.as_slice() {
            [] => continue,
            [file] => return Ok(file.clone()),
            _ => bail!(
                "Several projects in '{}': {}. Pass one, e.g. plottery run <file>",
                dir.display(),
                files
                    .iter()
                    .map(|file| file.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
    bail!(
        "No project found in '{}' or its parent folders. Pass the project folder, e.g. plottery run <folder>",
        start.display()
    )
}
