use itertools::Itertools;
use plottery_lib::*;

use crate::{
    accelleration::{
        accelleration_path::AccellerationPath, speed_delay_handler::SpeedDelayHandler,
    },
    hardware::{Hardware, HardwareExecutor},
    plot_setting::PlotSettings,
};

pub fn plot_layer<E: HardwareExecutor>(
    hardware: &mut Hardware<E>,
    layer: &Layer,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) {
    for shape in layer.iter_flattened() {
        plot_shape(hardware, shape, sample_settings, plot_settings);
    }
}

pub fn travel_to<E: HardwareExecutor>(
    hardware: &mut Hardware<E>,
    target_pos: V2,
    plot_settings: &PlotSettings,
) {
    let hardware_profile = *hardware.get_hardware_profile();

    let acc_path = AccellerationPath::new(
        &[hardware.get_pos(), target_pos],
        plot_settings.speed_travel.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );

    let speed_travel = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_travel,
        hardware_profile.dist_per_step_axis_cm,
    );
    let speed_head_up = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_up,
        hardware_profile.dist_per_step_head_cm,
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

pub fn plot_shape<E: HardwareExecutor>(
    hardware: &mut Hardware<E>,
    shape: &Shape,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) {
    let points = shape.get_points_from(hardware.get_pos(), sample_settings);
    if points.len() < 2 {
        return;
    }

    let hardware_profile = *hardware.get_hardware_profile();

    let accelleration_path = AccellerationPath::new(
        &points,
        plot_settings.speed_draw.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );

    let speed_draw = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_draw,
        hardware_profile.dist_per_step_axis_cm,
    );
    let speed_head_down = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_down,
        hardware_profile.dist_per_step_head_cm,
    );

    if accelleration_path.points.len() < 2 {
        return;
    }

    // travel to start
    travel_to(hardware, accelleration_path.points[0].point, plot_settings);

    // draw
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
