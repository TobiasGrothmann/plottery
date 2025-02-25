use itertools::Itertools;
use plottery_lib::*;
use plottery_server_lib::{plot_settings::PlotSettings, task::Task};
use tokio::sync::mpsc;
use tokio::task;

use crate::accelleration_path::AccellerationPath;
use crate::hardware::Hardware;
use crate::pins::PIN_SETTINGS;
use crate::speed_delay_handler::SpeedDelayHandler;

pub async fn start_server(mut receiver: mpsc::Receiver<Task>) {
    task::spawn(async move {
        let mut hardware = Hardware::new(PIN_SETTINGS);

        while let Some(task) = receiver.recv().await {
            println!("Received task: {:?}", task);
            match task {
                Task::Plot {
                    layer,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    plot_layer(&mut hardware, &layer, &sample_settings, &plot_settings).await;
                    hardware.set_enabled(false);
                }
                Task::PlotShape {
                    shape,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    plot_shape(&mut hardware, &shape, &sample_settings, &plot_settings).await;
                    hardware.set_enabled(false);
                }
                Task::SetEnabled(enabled) => {
                    hardware.set_enabled(enabled);
                }
                Task::Abort => {
                    todo!()
                }
            }
        }
    });
}

pub async fn plot_layer(
    hardware: &mut Hardware,
    layer: &Layer,
    sample_settings: &SampleSettings,
    plot_settings: &PlotSettings,
) {
    println!("Plotting layer: {:?}", layer);
    for shape in layer.shapes.iter() {
        plot_shape(hardware, shape, sample_settings, plot_settings).await;
    }
}

pub async fn plot_shape(
    hardware: &mut Hardware,
    shape: &Shape,
    sample_settings: &SampleSettings,
    plot_settings: &PlotSettings,
) {
    let accelleration_path = AccellerationPath::new(
        &shape.get_points(sample_settings),
        plot_settings.speed_draw.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );
    let accelleration_path_to_start = AccellerationPath::new(
        &vec![hardware.get_pos(), accelleration_path.points[0].point],
        plot_settings.speed_travel.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );

    let speed_draw = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_draw,
        PIN_SETTINGS.dist_per_step_axis_cm,
    );
    let speed_travel = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_travel,
        PIN_SETTINGS.dist_per_step_axis_cm,
    );
    let speed_head_down = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_down,
        PIN_SETTINGS.dist_per_step_head_cm,
    );
    let speed_head_up = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_up,
        PIN_SETTINGS.dist_per_step_head_cm,
    );

    // travel to start
    hardware.set_head(
        false,
        plot_settings.head_pressure,
        plot_settings.speed_head_up.accelleration_distance,
        speed_head_up,
    );
    for (from, to) in accelleration_path_to_start.points.iter().tuple_windows() {
        hardware.move_to(from.speed, to, &speed_travel);
    }

    // draw
    hardware.set_head(
        true,
        plot_settings.head_pressure,
        plot_settings.speed_head_down.accelleration_distance,
        speed_head_down,
    );
    for (from, to) in accelleration_path.points.iter().tuple_windows() {
        hardware.move_to(from.speed, to, &speed_draw);
    }
    hardware.set_head(
        false,
        plot_settings.head_pressure,
        plot_settings.speed_head_up.accelleration_distance,
        speed_head_up,
    );
}
