use serde::{Deserialize, Serialize};

pub mod midi;
pub mod plot_setting;
pub mod server_state;
pub mod task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Head,
}

// TODO: config
pub static HOST_PORT: u16 = 29797;
pub static HOST_NAME: &str = "otter.local";
