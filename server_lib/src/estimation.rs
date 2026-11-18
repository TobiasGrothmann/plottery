use std::time::Duration;

use plottery_lib::{Layer, SampleSettings, V2};

use crate::{
    hardware::{Hardware, HardwareExecutor},
    pins::HARDWARE_CONSTS,
    plot_execution::{plot_layer, travel_to},
    plot_settings::PlotSettings,
    Axis,
};

pub fn estimate_plot_layer_duration(
    layer: &Layer,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) -> Duration {
    PlotTimeline::build(layer, sample_settings, plot_settings).total()
}

/// Steps taken and time passed since the start of a plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark {
    steps: u64,
    nanos: u64,
}

impl Mark {
    fn of(hardware: &Hardware<TimeEstimatorExecutor>) -> Self {
        Self {
            steps: hardware.steps(),
            nanos: hardware.executor().nanos(),
        }
    }
}

/// When a plot reaches each of its shapes, estimated step-exactly before it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotTimeline {
    /// One mark before each shape, one after the last shape and one back at the origin.
    marks: Vec<Mark>,
}

impl PlotTimeline {
    /// Timeline of a plot that starts at the origin with the head up.
    pub fn build(
        layer: &Layer,
        sample_settings: SampleSettings,
        plot_settings: &PlotSettings,
    ) -> Self {
        let hardware = Hardware::new(
            HARDWARE_CONSTS.hardware_profile,
            TimeEstimatorExecutor::new(),
        );
        Self::record(hardware, layer, sample_settings, plot_settings)
    }

    /// Timeline of a plot that starts where `machine` is. Aborting `machine` stops the build, which leaves the timeline incomplete.
    pub fn build_from<E: HardwareExecutor>(
        machine: &Hardware<E>,
        layer: &Layer,
        sample_settings: SampleSettings,
        plot_settings: &PlotSettings,
    ) -> Self {
        let hardware = Hardware::at_state_of(machine, TimeEstimatorExecutor::new())
            .with_abort_flag(machine.abort_flag());
        Self::record(hardware, layer, sample_settings, plot_settings)
    }

    fn record(
        mut hardware: Hardware<TimeEstimatorExecutor>,
        layer: &Layer,
        sample_settings: SampleSettings,
        plot_settings: &PlotSettings,
    ) -> Self {
        let mut marks = Vec::with_capacity(layer.len() + 2);

        hardware.set_enabled(true);
        plot_layer(
            &mut hardware,
            layer,
            sample_settings,
            plot_settings,
            |_, hardware| marks.push(Mark::of(hardware)),
        );
        marks.push(Mark::of(&hardware));
        travel_to(&mut hardware, V2::zero(), plot_settings);
        marks.push(Mark::of(&hardware));

        Self { marks }
    }

    pub fn shapes(&self) -> u32 {
        self.marks.len().saturating_sub(2) as u32
    }

    /// Duration of the whole plot, including the return to the origin.
    pub fn total(&self) -> Duration {
        Duration::from_nanos(self.marks.last().map_or(0, |mark| mark.nanos))
    }

    /// Time into the plot after `steps` steps, interpolated between the marks around it.
    pub fn elapsed_at(&self, steps: u64) -> Duration {
        let next = self.marks.partition_point(|mark| mark.steps <= steps);
        let (Some(before), Some(after)) = (
            next.checked_sub(1).and_then(|i| self.marks.get(i)),
            self.marks.get(next),
        ) else {
            return self.total();
        };
        let fraction = (steps - before.steps) as f64 / (after.steps - before.steps) as f64;
        let nanos = before.nanos + ((after.nanos - before.nanos) as f64 * fraction) as u64;
        Duration::from_nanos(nanos)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TimeEstimatorExecutor {
    last_steps_timestamp_nanos: [u128; Axis::COUNT],
    now_nanos: u128,
}

impl TimeEstimatorExecutor {
    pub(crate) fn new() -> Self {
        Self {
            last_steps_timestamp_nanos: [0; Axis::COUNT],
            now_nanos: 0,
        }
    }

    fn nanos(&self) -> u64 {
        u64::try_from(self.now_nanos).unwrap_or(u64::MAX)
    }

    #[cfg(test)]
    pub(crate) fn duration(&self) -> Duration {
        Duration::from_nanos(self.nanos())
    }
}

impl HardwareExecutor for TimeEstimatorExecutor {
    fn set_dir(&mut self, _axis: Axis, _forward: bool) {}

    fn step(&mut self, axis: Axis) {
        self.last_steps_timestamp_nanos[axis.index()] = self.now_nanos;
    }

    fn wait_axis(&mut self, axis: Axis, delay_nanos: u32) {
        let axis_i = axis.index();
        let delay_nanos = delay_nanos as u128;
        let delay_until = self.last_steps_timestamp_nanos[axis_i] + delay_nanos;
        self.now_nanos = self.now_nanos.max(delay_until);
    }

    fn wait_xy(&mut self, delay_nanos: u32) {
        let last_xy = self.last_steps_timestamp_nanos[Axis::X.index()]
            .max(self.last_steps_timestamp_nanos[Axis::Y.index()]);
        self.now_nanos = self.now_nanos.max(last_xy + delay_nanos as u128);
    }

    fn set_enabled(&mut self, _enabled: bool) {}
}
