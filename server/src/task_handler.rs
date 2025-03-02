use itertools::Itertools;
use plottery_lib::*;
use plottery_server_lib::plot_setting::PlotSettings;
use plottery_server_lib::task::Task;
use tokio::sync::mpsc;
use tokio::task;

use crate::accelleration_path::AccellerationPath;
use crate::hardware::Hardware;
use crate::pins::PIN_SETTINGS;
use crate::speed_delay_handler::SpeedDelayHandler;

pub async fn start_server(mut receiver: mpsc::Receiver<Task>) -> anyhow::Result<()> {
    let mut hardware = Hardware::new(PIN_SETTINGS)?;

    task::spawn(async move {
        while let Some(task) = receiver.recv().await {
            println!("...received task");
            match task {
                Task::Plot {
                    layer,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    plot_layer(&mut hardware, &layer, &sample_settings, &plot_settings).await;
                    travel_to(&mut hardware, V2::zero(), &plot_settings).await;
                    hardware.set_enabled(false);
                }
                Task::PlotShape {
                    shape,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    plot_shape(&mut hardware, &shape, &sample_settings, &plot_settings).await;
                    travel_to(&mut hardware, V2::zero(), &plot_settings).await;
                    hardware.set_enabled(false);
                }
                Task::SetEnabled(enabled) => {
                    hardware.set_enabled(enabled);
                }
                Task::Abort => {
                    todo!()
                }
                Task::SetHead(head_down) => {
                    let settings = PlotSettings::default();
                    let speed_range = if head_down {
                        settings.speed_head_down.clone()
                    } else {
                        settings.speed_head_up.clone()
                    };

                    hardware.set_enabled(true);
                    hardware.set_head(
                        head_down,
                        settings.head_pressure,
                        speed_range.accelleration_distance,
                        SpeedDelayHandler::new_from_speed_range(
                            &speed_range,
                            PIN_SETTINGS.dist_per_step_head_cm,
                        ),
                    );
                    hardware.set_enabled(false);
                }
                Task::MoveTo(pos, plot_setting) => {
                    hardware.set_enabled(true);
                    travel_to(&mut hardware, pos, &plot_setting).await;
                    hardware.set_enabled(false);
                }
                Task::Move(delta, plot_settings) => {
                    let target_pos = hardware.get_pos() + delta;

                    hardware.set_enabled(true);
                    travel_to(&mut hardware, target_pos, &plot_settings).await;
                    hardware.set_enabled(false);
                }
            }
        }
    });

    Ok(())
}

pub async fn plot_layer(
    hardware: &mut Hardware,
    layer: &Layer,
    sample_settings: &SampleSettings,
    plot_settings: &PlotSettings,
) {
    for shape in layer.shapes.iter() {
        plot_shape(hardware, shape, sample_settings, plot_settings).await;
    }
}

pub async fn travel_to(hardware: &mut Hardware, target_pos: V2, plot_settings: &PlotSettings) {
    let acc_path = AccellerationPath::new(
        &[hardware.get_pos(), target_pos],
        plot_settings.speed_travel.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );
    let speed_travel = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_travel,
        PIN_SETTINGS.dist_per_step_axis_cm,
    );
    let speed_head_up = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_up,
        PIN_SETTINGS.dist_per_step_head_cm,
    );

    hardware.set_head(
        false,
        plot_settings.head_pressure,
        plot_settings.speed_head_up.accelleration_distance,
        speed_head_up,
    );
    for (from, to) in acc_path.points.iter().tuple_windows() {
        hardware.move_to(from.speed, to, &speed_travel);
    }
}

pub async fn plot_shape(
    hardware: &mut Hardware,
    shape: &Shape,
    sample_settings: &SampleSettings,
    plot_settings: &PlotSettings,
) {
    let accelleration_path = AccellerationPath::new(
        &shape.get_points_from(&hardware.get_pos(), sample_settings),
        plot_settings.speed_draw.accelleration_distance,
        plot_settings.corner_slowdown_power,
    );

    let speed_draw = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_draw,
        PIN_SETTINGS.dist_per_step_axis_cm,
    );
    let speed_head_down = SpeedDelayHandler::new_from_speed_range(
        &plot_settings.speed_head_down,
        PIN_SETTINGS.dist_per_step_head_cm,
    );

    // travel to start
    travel_to(hardware, accelleration_path.points[0].point, plot_settings).await;

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
}
