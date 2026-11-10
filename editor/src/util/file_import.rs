use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, ImageDecoder, ImageReader};

/// Longest side after downscaling.
pub const DOWNSCALE_MAX_PX: u32 = 2000;

/// Folder inside the project where copied images live, relative to the project dir.
pub const IMAGES_DIR: &str = "resources/images";

/// Folder inside the project where copied svgs live, relative to the project dir.
pub const SVGS_DIR: &str = "resources/svgs";

/// Copies `source` into the project's image folder, optionally downscaled, and returns the
/// path relative to the project dir. An identical file already present is reused.
pub fn import_image(project_dir: &Path, source: &Path, downscale: bool) -> Result<String> {
    let bytes = if downscale {
        downscaled_bytes(source)?
    } else {
        read(source)?
    };
    import_bytes(project_dir, IMAGES_DIR, source, &bytes)
}

/// Copies `source` into the project's svg folder, see [`import_image`].
pub fn import_svg(project_dir: &Path, source: &Path) -> Result<String> {
    import_bytes(project_dir, SVGS_DIR, source, &read(source)?)
}

fn read(source: &Path) -> Result<Vec<u8>> {
    std::fs::read(source).with_context(|| format!("Failed to read {}", source.display()))
}

/// Writes `bytes` into `project_dir/dir` under the name of `source`, returns the relative path.
fn import_bytes(project_dir: &Path, dir: &str, source: &Path, bytes: &[u8]) -> Result<String> {
    let target_dir = project_dir.join(dir);
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create {}", target_dir.display()))?;

    let file_name = source
        .file_name()
        .context("File path has no file name")?
        .to_string_lossy()
        .to_string();
    let target = unique_target(&target_dir, &file_name, bytes)?;
    if !target.exists() {
        std::fs::write(&target, bytes)
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
/// Resized images are turned upright first, since the re-encode drops EXIF.
fn downscaled_bytes(source: &Path) -> Result<Vec<u8>> {
    let image = open_upright(source)
        .with_context(|| format!("Failed to open image {}", source.display()))?;
    if image.width().max(image.height()) <= DOWNSCALE_MAX_PX {
        return read(source);
    }
    let resized = image.resize(DOWNSCALE_MAX_PX, DOWNSCALE_MAX_PX, FilterType::Triangle);
    let format = image::ImageFormat::from_path(source).unwrap_or(image::ImageFormat::Png);
    let mut out = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut out, format)
        .context("Failed to encode downscaled image")?;
    Ok(out.into_inner())
}

fn open_upright(source: &Path) -> Result<DynamicImage> {
    let mut decoder = ImageReader::open(source)?
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    Ok(image)
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

/// Resolves a stored file param path against the project dir, as the running project will.
pub fn resolve_stored_path(project_dir: &Path, stored: &str) -> PathBuf {
    let path = Path::new(stored);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    }
}
