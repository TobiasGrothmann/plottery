use std::path::PathBuf;

use crate::{project_definition::compile_cargo::compile_cargo_project, ProjectConfig};
use anyhow::{Error, Result};
use libloading::Library;
use plottery_lib::Layer;

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

        let mut project_config_path = self.dir.clone();
        project_config_path.push(format!("{}.plottery", self.project_config.name));
        let project_config_file_exists = project_config_path.exists();

        let cargo_project_exists = self.get_cargo_path().is_ok();

        dir_exsists && project_config_file_exists && cargo_project_exists
    }

    pub fn load_from_file(project_config_path: PathBuf) -> Result<Self> {
        let project_config_file = ProjectConfig::new_from_file(&project_config_path)?;
        let mut project_config_dir = project_config_path.clone();
        project_config_dir.pop();
        let loaded_project = Self {
            project_config: project_config_file,
            dir: project_config_dir,
        };
        assert!(loaded_project.exists());
        Ok(loaded_project)
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

    pub fn get_cargo_path(&self) -> Result<PathBuf> {
        if !self.dir.exists() {
            return Err(Error::msg("Project dir does not exist"));
        }
        for entry in std::fs::read_dir(&self.dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let mut cargo_toml_path = path.clone();
                cargo_toml_path.push("Cargo.toml");
                if cargo_toml_path.exists() {
                    return Ok(path);
                }
            }
        }
        Err(Error::msg("Cargo.toml not found"))
    }

    pub fn get_cargo_toml_name(&self) -> Result<String> {
        let caro_path = self.get_cargo_path()?;
        let mut cargo_toml_path = caro_path.clone();
        cargo_toml_path.push("Cargo.toml");

        let toml_str = std::fs::read_to_string(cargo_toml_path)?;
        let toml: toml::Value = toml::from_str(&toml_str)?;

        let package = toml.get("package").unwrap();
        let name = package.get("name").unwrap();
        let name = name.as_str().unwrap();
        Ok(name.to_string())
    }

    pub fn generate_to_disk(&self) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        self.save_config()?;

        // generate cargo project from template
        if self.get_cargo_path().is_err() {
            generate_cargo_project(self.dir.clone(), self.project_config.name.clone())?;
        }

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

    pub fn get_plottery_target_dir(&self) -> Result<PathBuf> {
        let mut target_dir = self.get_cargo_path()?;
        target_dir.push("target/plottery");
        Ok(target_dir)
    }

    pub fn compile(&self, release: bool) -> Result<()> {
        let cargo_path = self.get_cargo_path().unwrap();
        let build_status =
            compile_cargo_project(cargo_path.clone(), self.get_plottery_target_dir()?, release)?;
        assert!(build_status.success());
        Ok(())
    }

    pub fn run_code(&self, release: bool) -> Result<Layer> {
        let cargo_name = self.get_cargo_toml_name()?;

        let mut lib_path = self.get_plottery_target_dir()?;
        if release {
            lib_path.push("release");
        } else {
            lib_path.push("debug");
        }
        lib_path.push(format!("lib{}.dylib", cargo_name));

        if !lib_path.exists() {
            return Err(Error::msg(format!(
                "library does not exist at '{}'",
                lib_path.to_string_lossy()
            )));
        }

        unsafe {
            let lib = Library::new(lib_path).expect("Failed to load library");

            let generate_func: libloading::Symbol<fn() -> Layer> = lib
                .get(b"generate")
                .expect("Failed to get symbol for generation function in library");

            let generated_layer: Layer = generate_func();

            Ok(generated_layer)
        }
    }
}
