use std::path::{Path, PathBuf};

use dioxus::prelude::*;

use crate::router::editor::variations::variations_state::EditorView;

/// A file macOS asked the editor to open: a project's `.plottery` file shows the project,
/// its `variations/variations.json` shows its variations.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenRequest {
    pub config_path: PathBuf,
    pub view: EditorView,
}

impl OpenRequest {
    pub fn from_file(path: &Path) -> Option<Self> {
        if path
            .extension()
            .is_some_and(|extension| extension == "plottery")
        {
            return Some(Self {
                config_path: path.to_path_buf(),
                view: EditorView::Edit,
            });
        }
        let variations_dir = path.parent()?;
        if path.file_name()? != "variations.json" || variations_dir.file_name()? != "variations" {
            return None;
        }
        let config_path = std::fs::read_dir(variations_dir.parent()?)
            .ok()?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .find(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "plottery")
            })?;
        Some(Self {
            config_path,
            view: EditorView::Variations,
        })
    }
}

/// The view an open request asked for, until the editor of that project shows it.
#[derive(Clone, Copy)]
pub struct RequestedView(pub Signal<Option<(PathBuf, EditorView)>>);
