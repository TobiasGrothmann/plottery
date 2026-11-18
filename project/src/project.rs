use crate::{
    generate_cargo_project_to_disk, init_git_repo,
    project_util::{build_cargo_project_async, path_hash, run_project_executable_async},
    protocol::check_binary_protocol,
    read_object_from_stdout,
    run_stream::RunSocket,
    write_agents_md, LibSource, ProjectConfig,
};

use plottery_lib::*;
use plottery_runtime::ProjectParamsListWrapper;

use anyhow::{Context, Error, Ok, Result};
use async_process::Child;
use path_absolutize::Absolutize;
use resvg::{tiny_skia, usvg};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

/// Longest side of the browser thumbnail in pixels.
pub const EDITOR_THUMBNAIL_SIZE: u32 = 1024;

/// Rasterizes SVG data, scaled so the longest side is `max_size` when given.
fn render_svg(
    svg_data: &[u8],
    max_size: Option<u32>,
    background: Option<tiny_skia::Color>,
) -> Result<tiny_skia::Pixmap> {
    let rtree = {
        let opt = usvg::Options::default();
        let mut fontdb = fontdb::Database::new();
        // Scanning the system fonts is slow and only text needs them, which layers never contain.
        if svg_data.windows(5).any(|bytes| bytes == b"<text") {
            fontdb.load_system_fonts();
        }
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
    if let Some(background) = background {
        pixmap.fill(background);
    }
    rtree.render(
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Rasterizes SVG data to a PNG, scaled so the longest side is `max_size` when given.
fn render_svg_to_png(
    svg_data: &[u8],
    path: &std::path::Path,
    max_size: Option<u32>,
    background: Option<tiny_skia::Color>,
) -> Result<()> {
    render_svg(svg_data, max_size, background)?.save_png(path)?;
    Ok(())
}

/// The pixels of a rasterized layer.
pub struct Raster {
    pub width: u32,
    pub height: u32,
    /// Four bytes per pixel, rows from the top.
    pub rgba: Vec<u8>,
}

/// Rasterizes `layer` on white, its longest side `max_size` pixels. Shows exactly `view` instead
/// of the layer's bounding box, so that layers of different extent give images that line up.
pub fn rasterize(layer: &Layer, view: Rect, max_size: u32) -> Result<Raster> {
    let svg_data = layer.to_svg_in(view).to_string();
    let pixmap = render_svg(
        svg_data.as_bytes(),
        Some(max_size),
        Some(tiny_skia::Color::WHITE),
    )?;
    Ok(Raster {
        width: pixmap.width(),
        height: pixmap.height(),
        rgba: pixmap.take(),
    })
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

/// Plottery's folder in the user's cache directory, for builds and previews.
fn plottery_cache_dir() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("de", "tobiasgrothmann", "plottery")
        .ok_or_else(|| Error::msg("Could not determine the user's cache directory"))?;
    Ok(project_dirs.cache_dir().to_path_buf())
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// True when `cached` exists and is not older than the `layer` file it was rendered from.
pub(crate) fn is_up_to_date(layer: &Path, cached: &Path) -> bool {
    match (modified(layer), modified(cached)) {
        (Some(layer), Some(cached)) => cached >= layer,
        _ => false,
    }
}

/// True when `layer` exists but `thumbnail` is missing, older, or below [`EDITOR_THUMBNAIL_SIZE`].
pub(crate) fn thumbnail_is_stale(layer: &Path, thumbnail: &Path) -> bool {
    let usable = is_up_to_date(layer, thumbnail)
        && png_longest_side(thumbnail).is_some_and(|side| side >= EDITOR_THUMBNAIL_SIZE);
    layer.exists() && !usable
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
        let dir_exists = self.dir.exists();

        let mut project_config_path = self.dir.clone();
        project_config_path.push(format!("{}.plottery", self.config.name));
        let project_config_file_exists = project_config_path.exists();

        let cargo_project_exists = self.get_cargo_path().is_ok();

        dir_exists && project_config_file_exists && cargo_project_exists
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

    /// A file in the cache's `previews/`, named after the project and a hash of its folder.
    fn get_editor_cache_path(&self, extension: &str) -> Result<PathBuf> {
        let file_name = format!("{}-{}.{extension}", self.config.name, path_hash(&self.dir));
        Ok(plottery_cache_dir()?.join("previews").join(file_name))
    }
    /// The layer of the last run as SVG, a cache outside the project.
    pub fn get_editor_preview_image_path(&self) -> Result<PathBuf> {
        self.get_editor_cache_path("svg")
    }
    /// Rasterized version of the editor preview, used by the project browser.
    pub fn get_editor_thumbnail_path(&self) -> Result<PathBuf> {
        self.get_editor_cache_path("png")
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
        Ok(plottery_cache_dir()?.join("target"))
    }

    pub fn get_plottery_binary_path(&self, release: bool) -> Result<PathBuf> {
        let cargo_name = self.get_cargo_toml_name()?;
        let mut bin_path = self.get_plottery_target_dir()?;
        if release {
            bin_path.push("release");
        } else {
            bin_path.push("debug");
        }
        bin_path.push(cargo_name);
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
        let layer = rt.block_on(self.run_with_params_async(
            release,
            params,
            None,
            |_| {},
            std::future::pending(),
        ))?;
        Ok(layer)
    }

    /// Starts the built project. It sends its final layer through the returned socket, see
    /// [`RunSocket::receive`], and before that every `frame_every`-th frame it passes to
    /// [`plottery_lib::snapshot`] when `frame_every` is given.
    pub async fn run_async(
        &self,
        release: bool,
        params: &ProjectParamsListWrapper,
        frame_every: Option<usize>,
    ) -> Result<(Child, RunSocket)> {
        let bin_path = self.get_plottery_binary_path(release)?;
        if !bin_path.exists() {
            return Err(Error::msg(format!(
                "Executable does not exist at '{}'.",
                bin_path.to_string_lossy()
            )));
        }
        check_binary_protocol(&bin_path).await?;

        let socket = RunSocket::bind()?;
        let mut arguments = vec![
            "socket".to_string(),
            socket.path().to_string_lossy().to_string(),
            "--piped-params".to_string(),
        ];
        if let Some(every) = frame_every {
            arguments.extend(["--frame-every".to_string(), every.to_string()]);
        }
        let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
        let child =
            run_project_executable_async(&bin_path, &arguments, Some(params), &self.dir).await?;
        Ok((child, socket))
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
        check_binary_protocol(&bin_path).await?;
        let arguments = vec!["params"];
        Ok(run_project_executable_async(&bin_path, &arguments, None, &self.dir).await?)
    }

    pub fn write_svg(&self, path: PathBuf, layer: &Layer) -> Result<()> {
        layer.write_svg(path)
    }

    /// Writes a PNG on white, its longest side `max_size` pixels.
    pub fn write_png(&self, path: PathBuf, layer: &Layer, max_size: u32) -> Result<()> {
        let svg_data = layer.to_svg().to_string();
        render_svg_to_png(
            svg_data.as_bytes(),
            &path,
            Some(max_size),
            Some(tiny_skia::Color::WHITE),
        )
    }

    /// Params last used by the editor, empty when none were saved, and the names of saved params
    /// that could not be decoded and are left out.
    pub fn load_editor_params(&self) -> Result<(ProjectParamsListWrapper, Vec<String>)> {
        let path = self.get_editor_params_path();
        match std::fs::read(&path) {
            core::result::Result::Ok(binary) => ProjectParamsListWrapper::from_file_bytes(&binary)
                .with_context(|| format!("Failed to read saved params '{}'", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok((ProjectParamsListWrapper::empty(), Vec::new()))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Writes a temp file and renames it, so a watching editor never reads half a file.
    /// Fails instead of replacing a file that can't be read.
    pub fn save_editor_params(&self, params: &ProjectParamsListWrapper) -> Result<()> {
        self.load_editor_params()
            .context("Params not saved, fix or delete the file")?;
        let path = self.get_editor_params_path();
        let temp_path = path.with_extension("plotp.tmp");
        std::fs::write(&temp_path, params.to_file_bytes()?)?;
        std::fs::rename(temp_path, path)?;
        Ok(())
    }

    pub fn save_editor_layer(&self, layer: &Layer) -> Result<()> {
        std::fs::write(self.get_editor_layer_path(), layer.to_binary()?)?;
        Ok(())
    }

    /// Writes the editor preview SVG and the browser thumbnail rendered from it into the cache.
    pub fn save_editor_preview(&self, svg: &str) -> Result<()> {
        let preview_path = self.get_editor_preview_image_path()?;
        let thumbnail_path = self.get_editor_thumbnail_path()?;
        if let Some(dir) = preview_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(preview_path, svg)?;
        render_svg_to_png(
            svg.as_bytes(),
            &thumbnail_path,
            Some(EDITOR_THUMBNAIL_SIZE),
            None,
        )
    }

    /// Writes params, layer, preview and thumbnail, like a run in the editor.
    pub fn save_editor_outputs(
        &self,
        params: &ProjectParamsListWrapper,
        layer: &Layer,
    ) -> Result<()> {
        self.save_editor_params(params)?;
        self.save_editor_layer(layer)?;
        self.save_editor_preview(&layer.to_svg().to_string())
    }

    /// The cached preview SVG, unless it is missing or older than the layer.
    pub fn fresh_editor_preview(&self) -> Option<PathBuf> {
        self.get_editor_preview_image_path()
            .ok()
            .filter(|preview| is_up_to_date(&self.get_editor_layer_path(), preview))
    }

    /// True when the thumbnail has to be rendered again from the layer.
    pub fn editor_thumbnail_is_stale(&self) -> bool {
        self.get_editor_thumbnail_path()
            .is_ok_and(|thumbnail| thumbnail_is_stale(&self.get_editor_layer_path(), &thumbnail))
    }

    /// Renders the preview and the thumbnail again from the layer on disk.
    pub fn update_editor_thumbnail(&self) -> Result<()> {
        let layer = Layer::new_from_file(&self.get_editor_layer_path())?;
        self.save_editor_preview(&layer.to_svg().to_string())
    }
}
