use std::path::PathBuf;

use crate::{FailedToOpenProjectError, ProjectConfig};

#[derive(Debug, Clone)]
pub struct Project {
    pub project_file: ProjectConfig,
    pub dir: PathBuf,
}

impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        self.project_file == other.project_file
            && self.dir.to_path_buf().canonicalize().unwrap()
                == other.dir.to_path_buf().canonicalize().unwrap()
    }
}

impl Project {
    pub fn new(parent: PathBuf, name: String) -> Self {
        let mut parent_dir = parent.clone();
        parent_dir.push(name.clone());
        Self {
            project_file: ProjectConfig::new(name),
            dir: parent_dir,
        }
    }

    pub fn new_from_file(project_config_path: PathBuf) -> Result<Self, FailedToOpenProjectError> {
        let project_config_file = ProjectConfig::new_from_file(&project_config_path)?;
        let mut project_config_dir = project_config_path.clone();
        project_config_dir.pop();
        Ok(Self {
            project_file: project_config_file,
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
        project_config_path.push(format!("{}.plottery", self.project_file.name));
        project_config_path
    }

    pub fn save(&self) -> Result<(), crate::FailedToSaveProjectError> {
        let resource_dir = self.get_resource_dir();
        if !resource_dir.exists() {
            std::fs::create_dir_all(resource_dir)?;
        }

        let project_config_path = self.get_project_config_path();
        self.project_file.save_to_file(&project_config_path)?;

        Ok(())
    }
}
