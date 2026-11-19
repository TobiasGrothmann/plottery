use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Overrides `plottery_home` of the config file when set.
pub const PLOTTERY_HOME_ENV: &str = "PLOTTERY_HOME";

pub(crate) fn plottery_dirs() -> Result<directories::ProjectDirs> {
    directories::ProjectDirs::from("de", "tobiasgrothmann", "plottery")
        .context("Could not determine the user's home directory")
}

/// Settings shared by the CLI and the editor, `config.toml` in the user's config directory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlotteryConfig {
    /// The local Plottery checkout that `LibSource::PlotteryHome` projects depend on.
    #[serde(default)]
    pub plottery_home: Option<PathBuf>,
}

impl PlotteryConfig {
    /// `~/Library/Application Support/de.tobiasgrothmann.plottery/config.toml` on macOS,
    /// `~/.config/plottery/config.toml` on Linux.
    pub fn path() -> Result<PathBuf> {
        Ok(plottery_dirs()?.config_dir().join("config.toml"))
    }

    /// A missing file is the default config.
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::path()?)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::path()?)
    }

    pub(crate) fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read '{}'", path.display()))?;
        toml::from_str(&text).with_context(|| format!("Failed to parse '{}'", path.display()))
    }

    pub(crate) fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)
            .with_context(|| format!("Failed to write '{}'", path.display()))
    }

    /// `env_home` (the value of `PLOTTERY_HOME`) wins over the file unless it is empty.
    pub(crate) fn home_with_override(&self, env_home: Option<String>) -> Option<PathBuf> {
        env_home
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
            .or_else(|| self.plottery_home.clone())
    }
}

/// The local Plottery checkout: `PLOTTERY_HOME` if set, else `plottery_home` of the config file.
pub fn plottery_home() -> Result<Option<PathBuf>> {
    Ok(PlotteryConfig::load()?.home_with_override(std::env::var(PLOTTERY_HOME_ENV).ok()))
}

/// Writes `plottery_home` to the config file and returns that file.
pub fn set_plottery_home(home: &Path) -> Result<PathBuf> {
    let mut config = PlotteryConfig::load()?;
    config.plottery_home = Some(home.to_path_buf());
    config.save()?;
    PlotteryConfig::path()
}
