use plottery_project::Project;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    pub projects: Vec<Project>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    pub fn get_save_file_path() -> PathBuf {
        let mut path = directories::ProjectDirs::from("de", "tobiasgrothmann", "plottery")
            .unwrap()
            .data_dir()
            .to_path_buf();
        path.push("app_state.json");
        path
    }

    pub fn load() -> Option<Self> {
        let path = Self::get_save_file_path();
        if path.exists() {
            let file = File::open(path).unwrap();
            serde_json::from_reader(file).unwrap()
        } else {
            None
        }
    }

    pub fn save(&self) {
        let path = Self::get_save_file_path();
        if !path.parent().unwrap().exists() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        }
        let file = std::fs::File::create(path).unwrap();
        serde_json::to_writer_pretty(file, self).unwrap();
    }
}
