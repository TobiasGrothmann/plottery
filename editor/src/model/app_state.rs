use dioxus_logger::tracing;
use path_absolutize::Absolutize;
use plottery_project::Project;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::File,
    path::{Path, PathBuf},
};

pub const TILE_SIZE_MIN: u32 = 140;
pub const TILE_SIZE_MAX: u32 = 600;
pub const TILE_SIZE_DEFAULT: u32 = 320;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    /// Folders scanned for projects, absolutized and deduplicated.
    #[serde(default)]
    pub roots: Vec<PathBuf>,
    #[serde(default)]
    pub selected_folder: Option<PathBuf>,
    #[serde(default)]
    pub expanded_folders: BTreeSet<PathBuf>,
    /// Minimum width of a project tile in the browser grid, in pixels.
    #[serde(default = "default_tile_size")]
    pub tile_size: u32,
    /// v1 project list, only read to migrate it into `roots`.
    #[serde(rename = "projects", default, skip_serializing)]
    legacy_projects: Vec<Project>,
}

fn default_tile_size() -> u32 {
    TILE_SIZE_DEFAULT
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            selected_folder: None,
            expanded_folders: BTreeSet::new(),
            tile_size: TILE_SIZE_DEFAULT,
            legacy_projects: Vec::new(),
        }
    }
}

fn absolutize(path: &Path) -> PathBuf {
    path.absolutize()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| path.to_path_buf())
}

impl AppState {
    pub fn get_save_file_path() -> PathBuf {
        let mut path = directories::ProjectDirs::from("de", "tobiasgrothmann", "plottery")
            .unwrap()
            .data_dir()
            .to_path_buf();
        path.push("app_state.json");
        path
    }

    /// Loads the state file, migrating the v1 format; a missing or broken file yields a default.
    pub fn load_or_default() -> Self {
        let path = Self::get_save_file_path();
        if !path.exists() {
            tracing::info!("App state file does not exist. Creating new app state.");
            let state = Self::default();
            state.save();
            return state;
        }

        let parsed = File::open(&path)
            .map_err(anyhow::Error::from)
            .and_then(|file| serde_json::from_reader::<_, Self>(file).map_err(Into::into));
        let mut state = match parsed {
            Ok(state) => state,
            Err(err) => {
                tracing::error!("Failed to read app state, starting fresh: {err}");
                let backup = path.with_extension("json.bak");
                if let Err(err) = std::fs::rename(&path, &backup) {
                    tracing::error!("Failed to back up broken app state: {err}");
                }
                Self::default()
            }
        };
        state.migrate_v1();
        state
    }

    fn migrate_v1(&mut self) {
        if !self.roots.is_empty() || self.legacy_projects.is_empty() {
            self.legacy_projects.clear();
            return;
        }
        let parents: Vec<PathBuf> = self
            .legacy_projects
            .iter()
            .filter_map(|p| p.dir.parent().map(Path::to_path_buf))
            .collect();
        for parent in parents {
            self.add_root(parent);
        }
        tracing::info!(
            "Migrated {} projects into roots {:?}",
            self.legacy_projects.len(),
            self.roots
        );
        self.legacy_projects.clear();
        self.save();
    }

    pub fn save(&self) {
        let path = Self::get_save_file_path();
        let dir = path.parent().unwrap();
        if let Err(err) = std::fs::create_dir_all(dir) {
            tracing::error!("Failed to create app state dir: {err}");
            return;
        }
        let result = File::create(&path)
            .map_err(anyhow::Error::from)
            .and_then(|file| serde_json::to_writer_pretty(file, self).map_err(Into::into));
        if let Err(err) = result {
            tracing::error!("Failed to save app state: {err}");
        }
    }

    /// Returns `false` when the root was already registered.
    pub fn add_root(&mut self, root: PathBuf) -> bool {
        let root = absolutize(&root);
        if self.roots.contains(&root) {
            return false;
        }
        self.expanded_folders.insert(root.clone());
        self.roots.push(root);
        true
    }

    pub fn remove_root(&mut self, root: &Path) {
        let root = absolutize(root);
        self.roots.retain(|r| *r != root);
        if self
            .selected_folder
            .as_ref()
            .is_some_and(|f| f.starts_with(&root))
        {
            self.selected_folder = None;
        }
        self.expanded_folders.retain(|f| !f.starts_with(&root));
    }

    pub fn set_tile_size(&mut self, size: u32) {
        self.tile_size = size.clamp(TILE_SIZE_MIN, TILE_SIZE_MAX);
    }

    /// Adds `dir`'s parent as a root unless some root already contains `dir`.
    pub fn ensure_root_covers(&mut self, dir: &Path) {
        let dir = absolutize(dir);
        if self.roots.iter().any(|r| dir.starts_with(r)) {
            return;
        }
        if let Some(parent) = dir.parent() {
            self.add_root(parent.to_path_buf());
        }
    }
}
