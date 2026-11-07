use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::imageops::FilterType;

/// Longest side after downscaling.
pub const DOWNSCALE_MAX_PX: u32 = 2000;

/// Folder inside the project where copied images live, relative to the project dir.
pub const IMAGES_DIR: &str = "resources/images";

/// Copies `source` into the project's image folder, optionally downscaled, and returns the
/// path relative to the project dir. An identical file already present is reused.
pub fn import_image(project_dir: &Path, source: &Path, downscale: bool) -> Result<String> {
    let images_dir = project_dir.join(IMAGES_DIR);
    std::fs::create_dir_all(&images_dir)
        .with_context(|| format!("Failed to create {}", images_dir.display()))?;

    let bytes = if downscale {
        downscaled_bytes(source)?
    } else {
        std::fs::read(source).with_context(|| format!("Failed to read {}", source.display()))?
    };

    let file_name = source
        .file_name()
        .context("Image path has no file name")?
        .to_string_lossy()
        .to_string();
    let target = unique_target(&images_dir, &file_name, &bytes)?;
    if !target.exists() {
        std::fs::write(&target, &bytes)
            .with_context(|| format!("Failed to write {}", target.display()))?;
    }

    let relative = target
        .strip_prefix(project_dir)
        .unwrap_or(&target)
        .to_string_lossy()
        .replace('\\', "/");
    Ok(relative)
}

/// Re-encodes the image with its longest side at most [`DOWNSCALE_MAX_PX`]. Images that are
/// already small are returned unchanged, so lossy jpegs are not re-encoded needlessly.
fn downscaled_bytes(source: &Path) -> Result<Vec<u8>> {
    let image = image::open(source)
        .with_context(|| format!("Failed to open image {}", source.display()))?;
    if image.width().max(image.height()) <= DOWNSCALE_MAX_PX {
        return std::fs::read(source)
            .with_context(|| format!("Failed to read {}", source.display()));
    }
    let resized = image.resize(DOWNSCALE_MAX_PX, DOWNSCALE_MAX_PX, FilterType::Triangle);
    let format = image::ImageFormat::from_path(source).unwrap_or(image::ImageFormat::Png);
    let mut out = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut out, format)
        .context("Failed to encode downscaled image")?;
    Ok(out.into_inner())
}

/// `name.ext`, or `name_1.ext`, `name_2.ext`, … if a different file already uses the name.
fn unique_target(dir: &Path, file_name: &str, bytes: &[u8]) -> Result<PathBuf> {
    let (stem, ext) = match file_name.rsplit_once('.') {
        Some((stem, ext)) => (stem.to_string(), format!(".{ext}")),
        None => (file_name.to_string(), String::new()),
    };
    for i in 0.. {
        let candidate = if i == 0 {
            dir.join(format!("{stem}{ext}"))
        } else {
            dir.join(format!("{stem}_{i}{ext}"))
        };
        if !candidate.exists() {
            return Ok(candidate);
        }
        let existing = std::fs::read(&candidate)
            .with_context(|| format!("Failed to read {}", candidate.display()))?;
        if existing == bytes {
            return Ok(candidate);
        }
    }
    unreachable!()
}

/// Resolves a stored image path against the project dir, as the running project will.
pub fn resolve_image_path(project_dir: &Path, stored: &str) -> PathBuf {
    let path = Path::new(stored);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    }
}
