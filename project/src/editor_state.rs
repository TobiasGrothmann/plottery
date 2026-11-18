use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::plottery_config::plottery_dirs;

/// The file the editor keeps its state in, `roots` among it.
pub fn editor_app_state_path() -> Result<PathBuf> {
    Ok(plottery_dirs()?.data_dir().join("app_state.json"))
}

#[derive(Deserialize)]
struct Roots {
    #[serde(default)]
    roots: Vec<PathBuf>,
}

/// The folders the editor's browser scans for projects. Empty when the editor never ran.
pub fn editor_roots() -> Result<Vec<PathBuf>> {
    let path = editor_app_state_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read '{}'", path.display()))?;
    let state: Roots = serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse '{}'", path.display()))?;
    Ok(state.roots)
}
