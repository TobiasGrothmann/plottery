//! The stored variations of a project: `variations/` with the params, SVG and PNG of every roll.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use plottery_lib::Layer;
use plottery_runtime::ProjectParamsListWrapper;
use resvg::tiny_skia;
use serde::{Deserialize, Serialize};

use crate::project::{render_svg, write_atomic};
use crate::{rolled_values, Project};

/// Longest side of a roll's PNG in pixels.
pub const ROLL_THUMBNAIL_SIZE: u32 = 768;

const META_FILE: &str = "variations.json";
const ROLL_EXTENSIONS: [&str; 3] = ["plotp", "svg", "png"];

/// What a set of rolls was made of.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariationsMeta {
    pub created: DateTime<Utc>,
    /// Dotted paths of the rolled params.
    pub params: Vec<String>,
}

/// One roll. Roll 0 has the params the others were rolled from.
#[derive(Debug, Clone)]
pub struct Roll {
    pub index: usize,
    pub params: ProjectParamsListWrapper,
    /// The rolled params and their values as text.
    pub values: Vec<(String, String)>,
    /// Its SVG and PNG are written.
    pub done: bool,
}

/// The stored set of rolls of a project.
#[derive(Debug, Clone)]
pub struct Variations {
    pub meta: VariationsMeta,
    pub rolls: Vec<Roll>,
}

/// The index of a file of a roll, `7` for `007.png`, when its extension is one of `extensions`.
fn roll_index(path: &Path, extensions: &[&str]) -> Option<usize> {
    let extension = path.extension()?.to_str()?;
    if !extensions.contains(&extension) {
        return None;
    }
    path.file_stem()?.to_str()?.parse().ok()
}

fn files_in(dir: &Path) -> Result<Vec<PathBuf>> {
    match std::fs::read_dir(dir) {
        Ok(entries) => Ok(entries.filter_map(|e| e.ok().map(|e| e.path())).collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

fn remove_if_exists(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

impl Project {
    /// `variations/` in the project folder. It ignores itself in git.
    pub fn get_variations_dir(&self) -> PathBuf {
        self.dir.join("variations")
    }

    pub fn get_variations_meta_path(&self) -> PathBuf {
        self.get_variations_dir().join(META_FILE)
    }

    /// A file of roll `index`, like `variations/007.png`.
    pub fn get_roll_path(&self, index: usize, extension: &str) -> PathBuf {
        self.get_variations_dir()
            .join(format!("{index:03}.{extension}"))
    }

    /// The stored rolls in order, `None` when there are none.
    pub fn load_variations(&self) -> Result<Option<Variations>> {
        let meta_path = self.get_variations_meta_path();
        let meta: VariationsMeta = match std::fs::read(&meta_path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("Failed to read '{}'", meta_path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut indices: Vec<usize> = files_in(&self.get_variations_dir())?
            .iter()
            .filter_map(|path| roll_index(path, &["plotp"]))
            .collect();
        indices.sort();

        let mut rolls = Vec::new();
        for index in indices {
            // a new roll may be deleting the files of this one
            let bytes = match std::fs::read(self.get_roll_path(index, "plotp")) {
                Ok(bytes) => bytes,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            let (params, _) = ProjectParamsListWrapper::from_file_bytes(&bytes)
                .with_context(|| format!("Failed to read the params of roll {index}"))?;
            rolls.push(Roll {
                index,
                values: rolled_values(&params, &meta.params),
                done: self.get_roll_path(index, "png").exists(),
                params,
            });
        }
        Ok(Some(Variations { meta, rolls }))
    }

    /// The params of roll `index`.
    pub fn load_roll_params(&self, index: usize) -> Result<ProjectParamsListWrapper> {
        let path = self.get_roll_path(index, "plotp");
        let bytes = std::fs::read(&path).with_context(|| {
            match self.get_variations_meta_path().exists() {
                true => format!("There is no roll {index}"),
                false => format!(
                    "Project '{}' has no variations yet, roll some first",
                    self.config.name
                ),
            }
        })?;
        Ok(ProjectParamsListWrapper::from_file_bytes(&bytes)?.0)
    }

    /// Replaces the stored rolls with `sets`, not run yet. The description is written last, so a
    /// reader sees either no set or all of its params.
    pub(crate) fn start_variations(
        &self,
        meta: &VariationsMeta,
        sets: &[ProjectParamsListWrapper],
    ) -> Result<()> {
        let dir = self.get_variations_dir();
        std::fs::create_dir_all(&dir)?;
        let gitignore = dir.join(".gitignore");
        if !gitignore.exists() {
            std::fs::write(gitignore, "*\n")?;
        }
        remove_if_exists(&self.get_variations_meta_path())?;
        for path in files_in(&dir)? {
            let temp = path.extension().is_some_and(|extension| extension == "tmp");
            if temp || roll_index(&path, &ROLL_EXTENSIONS).is_some() {
                remove_if_exists(&path)?;
            }
        }
        for (index, params) in sets.iter().enumerate() {
            write_atomic(&self.get_roll_path(index, "plotp"), params.to_file_bytes()?)?;
        }
        write_atomic(
            &self.get_variations_meta_path(),
            serde_json::to_vec_pretty(meta)?,
        )
    }

    /// Writes the SVG and then the PNG of roll `index`. The PNG marks the roll as done.
    pub(crate) fn save_roll_output(&self, index: usize, layer: &Layer) -> Result<()> {
        let svg = layer.to_svg();
        write_atomic(&self.get_roll_path(index, "svg"), &svg)?;
        let png = render_svg(
            svg.as_bytes(),
            Some(ROLL_THUMBNAIL_SIZE),
            Some(tiny_skia::Color::WHITE),
        )?
        .encode_png()?;
        write_atomic(&self.get_roll_path(index, "png"), png)
    }

    /// Removes the rolls that have no PNG, so only finished ones are left.
    pub(crate) fn remove_unfinished_rolls(&self) -> Result<()> {
        for path in files_in(&self.get_variations_dir())? {
            let unfinished = roll_index(&path, &ROLL_EXTENSIONS)
                .is_some_and(|index| !self.get_roll_path(index, "png").exists());
            let temp = path.extension().is_some_and(|extension| extension == "tmp");
            if unfinished || temp {
                remove_if_exists(&path)?;
            }
        }
        Ok(())
    }
}
