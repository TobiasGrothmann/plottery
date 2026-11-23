//! The stored variations of a project: `variations/` with the params, layer, SVG and PNG of every roll.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use plottery_lib::Layer;
use plottery_runtime::ProjectParamsListWrapper;
use resvg::tiny_skia;
use serde::{Deserialize, Serialize};

use crate::project::{render_svg, write_atomic};
use crate::{rolled_values, Project, RollRanges};

/// Longest side of a roll's PNG in pixels.
pub const ROLL_THUMBNAIL_SIZE: u32 = 768;

const META_FILE: &str = "variations.json";
const ROLL_EXTENSIONS: [&str; 4] = ["plotp", "plotl", "png", "skipped"];

/// What a set of rolls was made of.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariationsMeta {
    pub created: DateTime<Utc>,
    /// Dotted paths of the rolled params.
    pub params: Vec<String>,
    /// The ranges given to rolled numbers.
    #[serde(default, skip_serializing_if = "RollRanges::is_empty")]
    pub ranges: RollRanges,
    /// Rolls asked for, roll 0 left out. `None` in sets from before it was stored.
    #[serde(default)]
    pub total: Option<usize>,
    /// The roll is over: done, stopped or failed. A hard-killed roll never gets here.
    #[serde(default)]
    pub ended: bool,
}

/// One roll. Roll 0 has the params the others were rolled from.
#[derive(Debug, Clone)]
pub struct Roll {
    pub index: usize,
    pub params: ProjectParamsListWrapper,
    /// The rolled params and their values as text.
    pub values: Vec<(String, String)>,
    /// It ran too long and was killed, there is no drawing.
    pub skipped: bool,
}

/// The stored set of rolls of a project: the finished ones, also while more are running.
#[derive(Debug, Clone)]
pub struct Variations {
    pub meta: VariationsMeta,
    pub rolls: Vec<Roll>,
}

impl Variations {
    /// Rolls done (skipped ones included, roll 0 left out) and asked for, while the roll goes on.
    pub fn progress(&self) -> Option<(usize, usize)> {
        let total = self.meta.total.filter(|_| !self.meta.ended)?;
        let done = self.rolls.iter().filter(|roll| roll.index > 0).count();
        Some((done, total))
    }
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
                skipped: self.get_roll_path(index, "skipped").exists(),
                params,
            });
        }
        Ok(Some(Variations { meta, rolls }))
    }

    /// The drawing of roll `index`.
    pub fn load_roll_layer(&self, index: usize) -> Result<Layer> {
        if self.get_roll_path(index, "skipped").exists() {
            bail!("Roll {index} ran too long and was skipped, it has no drawing");
        }
        Layer::new_from_file(&self.get_roll_path(index, "plotl"))
            .with_context(|| format!("There is no roll {index}"))
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

    /// Removes the stored rolls and starts an empty set.
    pub(crate) fn start_variations(&self, meta: &VariationsMeta) -> Result<()> {
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
        write_atomic(
            &self.get_variations_meta_path(),
            serde_json::to_vec_pretty(meta)?,
        )
    }

    /// Marks the set as ended, so nobody waits for more rolls.
    pub(crate) fn end_variations(&self, meta: &VariationsMeta) -> Result<()> {
        let meta = VariationsMeta {
            ended: true,
            ..meta.clone()
        };
        write_atomic(
            &self.get_variations_meta_path(),
            serde_json::to_vec_pretty(&meta)?,
        )
    }

    /// Stores finished roll `index`. Its params are written last: a roll with params is complete.
    pub(crate) fn save_roll(
        &self,
        index: usize,
        params: &ProjectParamsListWrapper,
        layer: &Layer,
    ) -> Result<()> {
        write_atomic(&self.get_roll_path(index, "plotl"), layer.to_binary()?)?;
        let png = render_svg(
            layer.to_svg().as_bytes(),
            Some(ROLL_THUMBNAIL_SIZE),
            Some(tiny_skia::Color::WHITE),
        )?
        .encode_png()?;
        write_atomic(&self.get_roll_path(index, "png"), png)?;
        write_atomic(&self.get_roll_path(index, "plotp"), params.to_file_bytes()?)
    }

    /// Stores roll `index` as skipped: its params and a marker, no drawing.
    pub(crate) fn save_skipped_roll(
        &self,
        index: usize,
        params: &ProjectParamsListWrapper,
    ) -> Result<()> {
        write_atomic(&self.get_roll_path(index, "skipped"), "ran too long\n")?;
        write_atomic(&self.get_roll_path(index, "plotp"), params.to_file_bytes()?)
    }
}
