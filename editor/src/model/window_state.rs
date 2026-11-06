use std::{fs::File, path::PathBuf};

use dioxus::desktop::tao::{
    dpi::{LogicalPosition, LogicalSize},
    window::Window,
};
use dioxus_logger::tracing;
use serde::{Deserialize, Serialize};

use crate::model::app_state::AppState;

const TITLE_BAR_HEIGHT: f64 = 28.0;

/// Window frame position and inner size in logical desktop coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Route shown when the window was closed, reopened on the next start.
    #[serde(default)]
    pub route: Option<String>,
}

impl WindowState {
    fn path() -> PathBuf {
        AppState::get_save_file_path().with_file_name("window_state.json")
    }

    pub fn load() -> Option<Self> {
        let file = File::open(Self::path()).ok()?;
        serde_json::from_reader(file)
            .map_err(|err| tracing::warn!("Ignoring broken window state: {err}"))
            .ok()
    }

    pub fn save(&self) {
        let result = File::create(Self::path())
            .map_err(anyhow::Error::from)
            .and_then(|file| serde_json::to_writer_pretty(file, self).map_err(Into::into));
        if let Err(err) = result {
            tracing::error!("Failed to save window state: {err}");
        }
    }

    /// Uses the outer size: `inner_size` stays at its initial value on wry windows (tao #889).
    pub fn capture(window: &Window) -> Option<Self> {
        let scale = window.scale_factor();
        let position = window.outer_position().ok()?.to_logical::<f64>(scale);
        let size = window.outer_size().to_logical::<f64>(scale);
        Some(Self {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height - TITLE_BAR_HEIGHT,
            route: None,
        })
    }

    pub fn position(&self) -> LogicalPosition<f64> {
        LogicalPosition::new(self.x, self.y)
    }

    pub fn size(&self) -> LogicalSize<f64> {
        LogicalSize::new(self.width, self.height)
    }
}
