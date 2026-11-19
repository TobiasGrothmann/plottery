use serde::{Deserialize, Serialize};

pub mod acceleration;
#[cfg(feature = "client")]
mod client;
pub mod discovery;
pub mod estimation;
pub mod hardware;
mod hardware_test;
pub mod midi;
pub mod pins;
pub mod plot_execution;
pub mod plot_settings;
mod plot_settings_test;
pub mod server_state;
pub mod task;
mod task_test;
pub mod telemetry;

#[cfg(feature = "client")]
pub use client::*;
pub use estimation::*;
pub use midi::*;
pub use pins::*;
pub use plot_settings::*;
pub use server_state::*;
pub use task::*;
pub use telemetry::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Head,
}

impl Axis {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Head => 2,
        }
    }
}
