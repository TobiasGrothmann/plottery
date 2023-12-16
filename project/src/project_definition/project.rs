use std::path::PathBuf;

use crate::ProjectConfig;
use anyhow::Result;

use super::generate_cargo_project;

#[derive(Debug, Clone)]
pub struct Project {
    pub project_config: ProjectConfig,
    pub dir: PathBuf,
}

impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        self.project_config == other.project_config
            && self.dir.to_path_buf().canonicalize().unwrap()
                == other.dir.to_path_buf().canonicalize().unwrap()
    }
}

impl Project {
    pub fn new(parent: PathBuf, name: String) -> Self {
        let mut project_dir = parent.clone();
        project_dir.push(name.clone());
        Self {
            project_config: ProjectConfig::new(name),
            dir: project_dir,
        }
    }

    pub fn exists(&self) -> bool {
        let dir_exsists = self.dir.exists();

        // also check if a .plottery file exists in dir
        let mut project_config_path = self.dir.clone();
        project_config_path.push(format!("{}.plottery", self.project_config.name));
        let project_config_file_exists = project_config_path.exists();

        dir_exsists && project_config_file_exists
    }

    pub fn load_from_file(project_config_path: PathBuf) -> Result<Self> {
        let project_config_file = ProjectConfig::new_from_file(&project_config_path)?;
        let mut project_config_dir = project_config_path.clone();
        project_config_dir.pop();
        Ok(Self {
            project_config: project_config_file,
            dir: project_config_dir,
        })
    }

    pub fn get_resource_dir(&self) -> PathBuf {
        let mut resource_dir = self.dir.clone();
        resource_dir.push("resources");
        resource_dir
    }

    pub fn get_project_config_path(&self) -> PathBuf {
        let mut project_config_path = self.dir.clone();
        project_config_path.push(format!("{}.plottery", self.project_config.name));
        project_config_path
    }

    pub fn first_time_generation(&self) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        self.save_config()?;

        // generate cargo project from template
        generate_cargo_project(self.dir.clone(), self.project_config.name.clone())?;

        // create resource dir
        let resource_dir = self.get_resource_dir();
        if !resource_dir.exists() {
            std::fs::create_dir_all(resource_dir)?;
        }

        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let project_config_path = self.get_project_config_path();
        self.project_config.save_to_file(&project_config_path)?;
        Ok(())
    }
}
