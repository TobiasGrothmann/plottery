use std::time::Duration;

use plottery_lib::{Layer, SampleSettings, V2};

use crate::{
    hardware::{Hardware, HardwareExecutor},
    pins::HARDWARE_CONSTS,
    plot_execution::{plot_layer, travel_to},
    plot_setting::PlotSettings,
    Axis,
};

pub fn estimate_plot_layer_duration(
    layer: &Layer,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) -> Duration {
    let executor = TimeEstimatorExecutor::new();
    let mut hardware = Hardware::new(HARDWARE_CONSTS.hardware_profile, executor);

    hardware.set_enabled(true);
    plot_layer(&mut hardware, layer, sample_settings, plot_settings);
    travel_to(&mut hardware, V2::zero(), plot_settings);
    hardware.set_enabled(false);

    hardware.executor().duration()
}

#[derive(Debug, Clone, Copy)]
struct TimeEstimatorExecutor {
    last_steps_timestamp_nanos: [u128; 3],
    now_nanos: u128,
}

impl TimeEstimatorExecutor {
    fn new() -> Self {
        Self {
            last_steps_timestamp_nanos: [0, 0, 0],
            now_nanos: 0,
        }
    }

    fn duration(&self) -> Duration {
        let nanos = u64::try_from(self.now_nanos).unwrap_or(u64::MAX);
        Duration::from_nanos(nanos)
    }
}

impl HardwareExecutor for TimeEstimatorExecutor {
    fn set_dir(&mut self, _axis: Axis, _forward: bool) {}

    fn step(&mut self, axis: Axis) {
        self.last_steps_timestamp_nanos[axis_to_index(axis)] = self.now_nanos;
    }

    fn wait_axis(&mut self, axis: Axis, delay_nanos: u32) {
        let axis_i = axis_to_index(axis);
        let delay_nanos = delay_nanos as u128;
        let delay_until = self.last_steps_timestamp_nanos[axis_i] + delay_nanos;
        self.now_nanos = self.now_nanos.max(delay_until);
    }

    fn set_enabled(&mut self, _enabled: bool) {}
}

fn axis_to_index(axis: Axis) -> usize {
    match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Head => 2,
    }
}
