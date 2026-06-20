use std::time::Duration;

use itertools::Itertools;
use plottery_lib::{geometry::v2i::V2i, *};

use crate::{
    accelleration::{
        accelleration_path::{AccellerationPath, V2Speed},
        speed_delay_handler::SpeedDelayHandler,
    },
    pins::{PinSettings, PIN_SETTINGS},
    plot_setting::PlotSettings,
    Axis,
};

pub fn estimate_plot_layer_duration(
    layer: &Layer,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) -> Duration {
    let mut hardware = HardwareEstimator::new(PIN_SETTINGS);

    plot_layer(&mut hardware, layer, sample_settings, plot_settings);
    travel_to(&mut hardware, V2::zero(), plot_settings);

    hardware.get_duration()
}

fn plot_layer(
    hardware: &mut HardwareEstimator,
    layer: &Layer,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) {
    for shape in layer.iter_flattened() {
        plot_shape(hardware, shape, sample_settings, plot_settings);
    }
}

fn travel_to(hardware: &mut HardwareEstimator, target_pos: V2, plot_settings: &PlotSettings) {
    let acc_path = AccellerationPath::new(
        &[hardware.get_pos(), target_pos],
        plot_settings.speed_travel.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );
    let speed_travel = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_travel,
        hardware.pin_settings.dist_per_step_axis_cm,
    );
    let speed_head_up = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_up,
        hardware.pin_settings.dist_per_step_head_cm,
    );

    hardware.set_head(
        false,
        plot_settings.head_pressure,
        plot_settings.speed_head_up.accelleration_distance,
        speed_head_up,
    );

    for (from, to) in acc_path.points.iter().tuple_windows() {
        hardware.move_to(from.speed, *to, &speed_travel);
    }
}

fn plot_shape(
    hardware: &mut HardwareEstimator,
    shape: &Shape,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) {
    let points = shape.get_points_from(hardware.get_pos(), sample_settings);
    if points.len() < 2 {
        return;
    }

    let accelleration_path = AccellerationPath::new(
        &points,
        plot_settings.speed_draw.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );

    let speed_draw = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_draw,
        hardware.pin_settings.dist_per_step_axis_cm,
    );
    let speed_head_down = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_down,
        hardware.pin_settings.dist_per_step_head_cm,
    );

    if accelleration_path.points.len() < 2 {
        return;
    }

    travel_to(hardware, accelleration_path.points[0].point, plot_settings);

    hardware.set_head(
        true,
        plot_settings.head_pressure,
        plot_settings.speed_head_down.accelleration_distance,
        speed_head_down,
    );

    for (from, to) in accelleration_path.points.iter().tuple_windows() {
        hardware.move_to(from.speed, *to, &speed_draw);
    }
}

#[derive(Debug, Clone, Copy)]
struct HardwareEstimator {
    x: i32,
    y: i32,
    head_down: bool,

    last_steps_timestamp_nanos: [u128; 3],
    now_nanos: u128,

    pin_settings: PinSettings,
}

impl HardwareEstimator {
    fn new(pin_settings: PinSettings) -> Self {
        Self {
            x: 0,
            y: 0,
            head_down: false,
            last_steps_timestamp_nanos: [0, 0, 0],
            now_nanos: 0,
            pin_settings,
        }
    }

    fn get_duration(&self) -> Duration {
        let nanos = u64::try_from(self.now_nanos).unwrap_or(u64::MAX);
        Duration::from_nanos(nanos)
    }

    fn get_pos(&self) -> V2 {
        V2::new(
            self.x as f32 * self.pin_settings.dist_per_step_axis_cm,
            self.y as f32 * self.pin_settings.dist_per_step_axis_cm,
        )
    }

    fn move_steps(
        &mut self,
        movement: V2i,
        speed_handler: &SpeedDelayHandler,
        speed_fraction_start: f32,
        speed_fraction_end: f32,
    ) {
        let directions_signs = V2i::new(
            if movement.x > 0 { 1 } else { -1 },
            if movement.y > 0 { 1 } else { -1 },
        );

        let mut stepped_x = 0;
        let mut stepped_y = 0;

        let movement_abs = movement.abs();
        let total_steps = movement_abs.x + movement_abs.y;
        if total_steps == 0 {
            return;
        }

        while stepped_x < movement_abs.x || stepped_y < movement_abs.y {
            let moved_fraction = (stepped_x as f32 + stepped_y as f32) / total_steps as f32;
            let speed_fraction =
                speed_fraction_start + (speed_fraction_end - speed_fraction_start) * moved_fraction;

            let delay_nanos = speed_handler
                .get_delay_nanos(speed_fraction.clamp(0.0, 1.0))
                .round() as u32;

            if Line::new(V2::new(0.0, 0.0), movement_abs.abs().to_float())
                .point_relation(V2::new(stepped_x as f32, stepped_y as f32))
                == PointLineRelation::Left
            {
                self.wait_for_axis(Axis::X, delay_nanos);
                stepped_x += 1;
                self.x += directions_signs.x;
            } else {
                self.wait_for_axis(Axis::Y, delay_nanos);
                stepped_y += 1;
                self.y += directions_signs.y;
            }
        }
    }

    fn move_to(
        &mut self,
        speed_fraction_start: f32,
        pos: V2Speed,
        speed_handler: &SpeedDelayHandler,
    ) {
        let delta =
            ((pos.point - self.get_pos()) / self.pin_settings.dist_per_step_axis_cm).round_to_int();
        if delta.is_zero() {
            return;
        }

        self.move_steps(delta, speed_handler, speed_fraction_start, pos.speed);
    }

    fn set_head(
        &mut self,
        down: bool,
        head_pressure: f32,
        accelleration_dist: f32,
        speed_handler: SpeedDelayHandler,
    ) {
        if self.head_down == down {
            return;
        }

        let head_travel_cm = self.pin_settings.head_travel_to_touch_cm
            + self.pin_settings.extra_head_travel_for_pressure_cm * head_pressure;
        let head_travel_steps = self.pin_settings.steps_for_cm_head(head_travel_cm);

        for i in 0..head_travel_steps {
            let fraction = i as f32 / head_travel_steps as f32;
            let current_head_travel_cm = head_travel_cm * fraction;
            let speed_fraction_acc: f32 = current_head_travel_cm / accelleration_dist;
            let speed_fraction_decc: f32 =
                (head_travel_cm - current_head_travel_cm) / accelleration_dist;
            let speed_fraction = speed_fraction_acc.min(speed_fraction_decc).clamp(0.0, 1.0);

            let delay_nanos = speed_handler.get_delay_nanos(speed_fraction.clamp(0.0, 1.0)) as u32;
            self.wait_for_axis(Axis::Head, delay_nanos);
        }

        self.head_down = down;
    }

    fn wait_for_axis(&mut self, axis: Axis, delay_nanos: u32) {
        let axis_i = axis_to_index(axis);
        let delay_nanos = delay_nanos as u128;
        let delay_until = self.last_steps_timestamp_nanos[axis_i] + delay_nanos;
        self.now_nanos = self.now_nanos.max(delay_until);
        self.last_steps_timestamp_nanos[axis_i] = self.now_nanos;
    }
}

fn axis_to_index(axis: Axis) -> usize {
    match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Head => 2,
    }
}
