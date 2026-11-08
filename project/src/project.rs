use crate::{
    generate_cargo_project_to_disk, init_git_repo,
    project_params_list_wrapper::ProjectParamsListWrapper,
    project_util::{build_cargo_project_async, run_project_executable_async},
    read_object_from_stdout, write_agents_md, LibSource, ProjectConfig,
};

use plottery_lib::*;

use anyhow::{Error, Ok, Result};
use async_process::Child;
use path_absolutize::Absolutize;
use resvg::{tiny_skia, usvg};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

/// Longest side of the browser thumbnail in pixels.
pub const EDITOR_THUMBNAIL_SIZE: u32 = 1024;

/// Rasterizes SVG data to a PNG, scaled so the longest side is `max_size` when given.
fn render_svg_to_png(svg_data: &[u8], path: &std::path::Path, max_size: Option<u32>) -> Result<()> {
    let rtree = {
        let opt = usvg::Options::default();
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        let mut tree = usvg::Tree::from_data(svg_data, &opt)?;
        tree.convert_text(&fontdb);
        resvg::Tree::from_usvg(&tree)
    };

    let size = rtree.size;
    let scale = match max_size {
        Some(max) => max as f32 / size.width().max(size.height()),
        None => 1.0,
    };
    let width = ((size.width() * scale).ceil() as u32).max(1);
    let height = ((size.height() * scale).ceil() as u32).max(1);
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| Error::msg("Failed to create pixmap"))?;
    rtree.render(
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.save_png(path)?;
    Ok(())
}

/// Reads the longest side from a PNG header without decoding the image.
fn png_longest_side(path: &std::path::Path) -> Option<u32> {
    use std::io::Read;
    let mut header = [0u8; 24];
    std::fs::File::open(path)
        .ok()?
        .read_exact(&mut header)
        .ok()?;
    if &header[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(header[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(header[20..24].try_into().ok()?);
    Some(width.max(height))
}

#[derive(Debug, Clone)]
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
        let parent = parent
            .absolutize()
            .map(|p| p.to_path_buf())
            .unwrap_or(parent);

        let mut project_dir = parent;
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
        let mut project_config_dir = project_config_path.absolutize()?.to_path_buf();
        project_config_dir.pop();
        let loaded_project = Self {
            config: project_config_file,
            dir: project_config_dir,
        };
        if !loaded_project.exists() {
            return Err(Error::msg(format!(
                "Project '{}' is incomplete: missing config file or cargo package",
                loaded_project.dir.display()
            )));
        }
        Ok(loaded_project)
    }

    pub fn normalize_paths_mut(&mut self) {
        if let core::result::Result::Ok(path) = self.dir.absolutize() {
            self.dir = path.to_path_buf();
        }
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
    /// Rasterized version of the editor preview, used by the project browser.
    pub fn get_editor_thumbnail_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor_thumb.png")
    }
    pub fn get_editor_layer_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor.plotl")
    }
    pub fn get_editor_params_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor.plotp")
    }
    pub fn get_editor_plot_settings_path(&self) -> PathBuf {
        self.get_resource_dir_asset_path("latest_editor.plots")
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
        for entry in std::fs::read_dir(&self.dir)?.flatten() {
            let path = entry.path();
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
        let cargo_toml_path = self.get_cargo_path()?.join("Cargo.toml");
        let toml_str = std::fs::read_to_string(&cargo_toml_path)?;
        let toml: toml::Value = toml::from_str(&toml_str)?;
        toml.get("package")
            .and_then(|package| package.get("name"))
            .and_then(|name| name.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::msg(format!(
                    "No package name in '{}'",
                    cargo_toml_path.display()
                ))
            })
    }

    pub fn generate_to_disk(&self, lib_source: LibSource, init_git: bool) -> Result<()> {
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

        write_agents_md(&self.dir, &self.config.name)?;

        if init_git {
            init_git_repo(&self.dir)?;
        }

        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let project_config_path = self.get_project_config_path();
        self.config.save_to_file(&project_config_path)?;
        Ok(())
    }

    /// The cargo target directory all projects build into, so that dependencies are compiled
    /// once and shared: `plottery/target` in the user's cache directory (`~/Library/Caches` on
    /// macOS). See [`build_cargo_project_async`] for how projects stay apart in there.
    pub fn get_plottery_target_dir(&self) -> Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("de", "tobiasgrothmann", "plottery")
            .ok_or_else(|| Error::msg("Could not determine the user's cache directory"))?;
        Ok(project_dirs.cache_dir().join("target"))
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
        let (mut child, named_pipe) = rt.block_on(self.run_async(release, params))?;
        let status = rt.block_on(child.status())?;

        if !status.success() {
            return Err(Error::msg(format!("Failed to run project: {}", status)));
        }

        let layer = Layer::new_from_file(&named_pipe.path().to_path_buf())?;
        Ok(layer)
    }

    pub async fn run_async(
        &self,
        release: bool,
        params: &ProjectParamsListWrapper,
    ) -> Result<(Child, NamedTempFile)> {
        let bin_path = self.get_plottery_binary_path(release)?;
        if !bin_path.exists() {
            return Err(Error::msg(format!(
                "Executable does not exist at '{}'.",
                bin_path.to_string_lossy()
            )));
        }

        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_named_pipe_path = temp_file.path().to_path_buf().to_string_lossy().to_string();

        let arguments = vec![
            "named-pipe",
            &temp_named_pipe_path,
            "--piped-params",
            "true",
        ];
        Ok((
            run_project_executable_async(&bin_path, &arguments, Some(params), &self.dir).await?,
            temp_file,
        ))
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
        Ok(run_project_executable_async(&bin_path, &arguments, None, &self.dir).await?)
    }

    pub fn write_svg(&self, path: PathBuf, layer: &Layer) -> Result<()> {
        layer.write_svg(path, 10.0)
    }

    pub fn write_png(&self, path: PathBuf, layer: &Layer) -> Result<()> {
        let svg_data = layer.to_svg(10.0).to_string();
        render_svg_to_png(svg_data.as_bytes(), &path, None)
    }

    /// Params last used by the editor, empty when none were saved.
    pub fn load_editor_params(&self) -> Result<ProjectParamsListWrapper> {
        match std::fs::read(self.get_editor_params_path()) {
            core::result::Result::Ok(binary) => Ok(bincode::deserialize(&binary)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(ProjectParamsListWrapper::empty())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_editor_params(&self, params: &ProjectParamsListWrapper) -> Result<()> {
        std::fs::write(self.get_editor_params_path(), bincode::serialize(params)?)?;
        Ok(())
    }

    pub fn save_editor_layer(&self, layer: &Layer) -> Result<()> {
        std::fs::write(self.get_editor_layer_path(), layer.to_binary()?)?;
        Ok(())
    }

    /// Writes the editor preview SVG and the browser thumbnail rendered from it.
    pub fn save_editor_preview(&self, svg: &str) -> Result<()> {
        std::fs::write(self.get_editor_preview_image_path(), svg)?;
        self.write_editor_thumbnail(svg.as_bytes())
    }

    /// Writes params, layer, preview and thumbnail, like a run in the editor.
    pub fn save_editor_outputs(
        &self,
        params: &ProjectParamsListWrapper,
        layer: &Layer,
    ) -> Result<()> {
        self.save_editor_params(params)?;
        self.save_editor_layer(layer)?;
        self.save_editor_preview(&layer.to_svg(1.0).to_string())
    }

    pub fn write_editor_thumbnail(&self, svg_data: &[u8]) -> Result<()> {
        render_svg_to_png(
            svg_data,
            &self.get_editor_thumbnail_path(),
            Some(EDITOR_THUMBNAIL_SIZE),
        )
    }

    /// True when the editor preview exists but the thumbnail is missing, older than it, or
    /// smaller than [`EDITOR_THUMBNAIL_SIZE`].
    pub fn editor_thumbnail_is_stale(&self) -> bool {
        let svg_modified =
            std::fs::metadata(self.get_editor_preview_image_path()).and_then(|m| m.modified());
        let png_path = self.get_editor_thumbnail_path();
        let png_modified = std::fs::metadata(&png_path).and_then(|m| m.modified());
        match (svg_modified, png_modified) {
            (core::result::Result::Ok(svg), core::result::Result::Ok(png)) => {
                png < svg
                    || png_longest_side(&png_path).is_none_or(|side| side < EDITOR_THUMBNAIL_SIZE)
            }
            (core::result::Result::Ok(_), Err(_)) => true,
            (Err(_), _) => false,
        }
    }

    /// Renders the thumbnail from the editor preview file on disk.
    pub fn update_editor_thumbnail(&self) -> Result<()> {
        let svg_data = std::fs::read(self.get_editor_preview_image_path())?;
        self.write_editor_thumbnail(&svg_data)
    }
}
