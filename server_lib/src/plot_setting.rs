use std::path::Path;

use anyhow::{bail, Context, Result};
use bincode::Options;
use plottery_lib::{split_file_header, with_file_header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedRange {
    pub min: f32,
    pub max: f32,
    pub accelleration_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSettings {
    pub corner_slowdown_power: f32,       // the lower the slower
    pub head_travel_beyond_paper_cm: f32, // extra head travel after the pen touches the paper

    pub speed_draw: SpeedRange,
    pub speed_travel: SpeedRange,
    pub speed_head_down: SpeedRange,
    pub speed_head_up: SpeedRange,
}

impl Default for PlotSettings {
    fn default() -> Self {
        // These are the speeds the machine physically ran at before the
        // 2026-10 timing fix: the old settings (draw 0.4-4.5, travel
        // 0.6-7.0, head down 0.5-4.0, head up 1.5-7.0) were tuned against a
        // hidden 62 µs per step, which is now gone. Real limits are found in
        // the retune (docs/step_timing.md, 0.5).
        PlotSettings {
            corner_slowdown_power: 0.2,
            head_travel_beyond_paper_cm: 0.3,
            speed_draw: SpeedRange {
                min: 0.39,
                max: 3.41,
                accelleration_distance: 0.14,
            },
            speed_travel: SpeedRange {
                min: 0.58,
                max: 4.68,
                accelleration_distance: 1.0,
            },
            speed_head_down: SpeedRange {
                min: 0.47,
                max: 2.68,
                accelleration_distance: 0.22,
            },
            speed_head_up: SpeedRange {
                min: 1.27,
                max: 3.75,
                accelleration_distance: 0.06,
            },
        }
    }
}

impl PlotSettings {
    const FILE_MAGIC: [u8; 4] = *b"PLTS";
    const FILE_VERSION: u16 = 1;

    /// The `.plots` format: a file header followed by the bincode of the settings.
    pub fn to_file_bytes(&self) -> Result<Vec<u8>> {
        Ok(with_file_header(
            &Self::FILE_MAGIC,
            Self::FILE_VERSION,
            &bincode::serialize(self)?,
        ))
    }

    pub fn from_file_bytes(bytes: &[u8]) -> Result<Self> {
        // leftover bytes are an error
        let bincode = bincode::options().with_fixint_encoding();
        let Some((version, payload)) = split_file_header(&Self::FILE_MAGIC, bytes) else {
            // written before files had a header
            return bincode
                .deserialize(bytes)
                .context("not a plot settings file of this or an older Plottery version");
        };
        // a changed `PlotSettings` gets a new version, and the old one an arm here that migrates it
        match version {
            Self::FILE_VERSION => bincode
                .deserialize(payload)
                .context("the plot settings file is damaged"),
            _ => bail!(
                "the plot settings file has format version {version}, this Plottery reads up to {}. Update Plottery.",
                Self::FILE_VERSION
            ),
        }
    }

    /// The settings saved at `path`, the defaults when there is no file.
    pub fn load_file(path: &Path) -> Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => Self::from_file_bytes(&bytes)
                .with_context(|| format!("Failed to read plot settings '{}'", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Fails instead of replacing a file that can't be read.
    pub fn save_file(&self, path: &Path) -> Result<()> {
        Self::load_file(path).context("Plot settings not saved, fix or delete the file")?;
        std::fs::write(path, self.to_file_bytes()?)?;
        Ok(())
    }
}
