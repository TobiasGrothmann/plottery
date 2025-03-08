use crate::{
    generate_cargo_project_to_disk,
    project_params_list_wrapper::ProjectParamsListWrapper,
    project_util::{build_cargo_project_async, run_project_executable_async},
    read_object_from_stdout, LibSource, ProjectConfig,
};

use plottery_lib::*;

use anyhow::{Error, Ok, Result};
use async_process::Child;
use path_absolutize::Absolutize;
use resvg::{tiny_skia, usvg};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub config: ProjectConfig,
    pub dir: PathBuf,
}

impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        if self.config != other.config {
            return false;
        }

        let self_dir = self.dir.canonicalize();
        let other_dir = other.dir.canonicalize();
        if let (core::result::Result::Ok(self_dir), core::result::Result::Ok(other_dir)) =
            (self_dir, other_dir)
        {
            return self_dir == other_dir;
        }
        self.dir.to_string_lossy() == other.dir.to_string_lossy()
    }
}

impl Project {
    pub fn new(parent: PathBuf, name: &str) -> Self {
        let mut project_dir = parent.clone();
        project_dir.push(name);
        Self {
            config: ProjectConfig::new(name),
            dir: project_dir,
        }
    }

    pub fn exists(&self) -> bool {
        let dir_exsists = self.dir.exists();

        let mut project_config_path = self.dir.clone();
        project_config_path.push(format!("{}.plottery", self.config.name));
        let project_config_file_exists = project_config_path.exists();

        let cargo_project_exists = self.get_cargo_path().is_ok();

        dir_exsists && project_config_file_exists && cargo_project_exists
    }

    pub fn load_from_file(project_config_path: PathBuf) -> Result<Self> {
        let project_config_file = ProjectConfig::new_from_file(&project_config_path)?;
        let mut project_config_dir = project_config_path.clone();
        project_config_dir.pop();
        let loaded_project = Self {
            config: project_config_file,
            dir: project_config_dir,
        };
        assert!(loaded_project.exists());
        Ok(loaded_project)
    }

    pub fn get_dir(&self) -> PathBuf {
        self.dir.clone()
    }

    pub fn get_resource_dir(&self) -> PathBuf {
        let mut resource_dir = self.dir.clone();
        resource_dir.push("resources");
        resource_dir
    }

    pub fn get_resource_dir_asset_path(&self, asset_name: &str) -> PathBuf {
        let mut resource_path = self.get_resource_dir();
        resource_path.push(asset_name);
        resource_path
    }

    pub fn get_editor_preview_image_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor.svg")
    }
    pub fn get_editor_layer_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor.plotl")
    }
    pub fn get_editor_params_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor.plotp")
    }

    pub fn get_project_config_path(&self) -> PathBuf {
        let mut project_config_path = self.dir.clone();
        project_config_path.push(format!("{}.plottery", self.config.name));
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

    pub fn generate_to_disk(&self, lib_source: LibSource) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        self.save_config()?;

        // generate cargo project from template
        if self.get_cargo_path().is_err() {
            let cargo_dir = self.dir.join(&self.config.name);
            generate_cargo_project_to_disk(cargo_dir, &self.config.name, lib_source)?;
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
        self.config.save_to_file(&project_config_path)?;
        Ok(())
    }

    pub fn get_plottery_target_dir(&self) -> Result<PathBuf> {
        let mut target_dir = self.get_cargo_path()?;
        target_dir.push("target/plottery");
        target_dir = target_dir.absolutize()?.to_path_buf();
        Ok(target_dir)
    }

    pub fn get_plottery_binary_path(&self, release: bool) -> Result<PathBuf> {
        let cargo_name = self.get_cargo_toml_name()?;
        let mut bin_path = self.get_plottery_target_dir()?;
        if release {
            bin_path.push("release");
        } else {
            bin_path.push("debug");
        }
        bin_path.push(cargo_name); // TODO: get name from cargo.toml if it has been set?
        Ok(bin_path)
    }

    pub fn build(&self, release: bool) -> Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let mut child = rt.block_on(self.build_async(release))?;
        let build_status = rt.block_on(child.status())?;

        if build_status.success() {
            Ok(())
        } else {
            Err(Error::msg("Failed to build project"))
        }
    }

    pub async fn build_async(&self, release: bool) -> Result<Child> {
        let child_process = build_cargo_project_async(
            self.get_cargo_path()?,
            self.get_plottery_target_dir()?,
            release,
        )
        .await?;
        Ok(child_process)
    }

    pub fn run(&self, release: bool, params: &ProjectParamsListWrapper) -> Result<Layer> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let mut child = rt.block_on(self.run_async(release, params))?;
        let layer = rt.block_on(read_object_from_stdout::<Layer>(&mut child))?;
        Ok(layer)
    }

    pub async fn run_async(
        &self,
        release: bool,
        params: &ProjectParamsListWrapper,
    ) -> Result<Child> {
        let bin_path = self.get_plottery_binary_path(release)?;
        if !bin_path.exists() {
            return Err(Error::msg(format!(
                "Executable does not exist at '{}'.",
                bin_path.to_string_lossy()
            )));
        }
        let arguments = vec!["std-out", "--piped-params", "true"];
        Ok(run_project_executable_async(&bin_path, &arguments, Some(params)).await?)
    }

    pub fn run_get_params(&self, release: bool) -> Result<ProjectParamsListWrapper> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let mut child = rt.block_on(self.run_get_params_async(release))?;
        let params = rt.block_on(read_object_from_stdout::<ProjectParamsListWrapper>(
            &mut child,
        ))?;
        Ok(params)
    }

    pub async fn run_get_params_async(&self, release: bool) -> Result<Child> {
        let bin_path = self.get_plottery_binary_path(release)?;
        if !bin_path.exists() {
            return Err(Error::msg(format!(
                "Executable does not exist at '{}'.",
                bin_path.to_string_lossy()
            )));
        }
        let arguments = vec!["params"];
        Ok(run_project_executable_async(&bin_path, &arguments, None).await?)
    }

    pub fn write_svg(
        &self,
        path: PathBuf,
        release: bool,
        params: &ProjectParamsListWrapper,
    ) -> Result<()> {
        let layer = self.run(release, params)?;
        layer.write_svg(path.clone(), 10.0)
    }

    pub fn write_png(
        &self,
        path: PathBuf,
        release: bool,
        params: &ProjectParamsListWrapper,
    ) -> Result<()> {
        let layer = self.run(release, params)?;
        let temp_dir = tempfile::tempdir()?;
        let temp_svg_path = temp_dir.path().join("test.svg");
        layer.write_svg(temp_svg_path.clone(), 10.0)?;

        let rtree = {
            let opt = usvg::Options::default();
            let mut fontdb = fontdb::Database::new();
            fontdb.load_system_fonts();

            let svg_data = std::fs::read(&temp_svg_path).expect("Failed to read svg file");
            let mut tree = usvg::Tree::from_data(&svg_data, &opt)?;
            tree.convert_text(&fontdb);
            resvg::Tree::from_usvg(&tree)
        };

        let pixmap_size = rtree.size.to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .expect("Failed to create pixmap");
        rtree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());
        pixmap.save_png(path)?;

        Ok(())
    }
}
