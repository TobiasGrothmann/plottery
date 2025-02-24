use itertools::Itertools;
use plottery_lib::*;
use plottery_server_lib::{plot_settings::PlotSettings, task::Task};
use tokio::sync::mpsc;
use tokio::task;

use crate::accelleration_path::AccellerationPath;
use crate::hardware::Hardware;

pub async fn start_server(mut receiver: mpsc::Receiver<Task>) {
    task::spawn(async move {
        let mut hardware = Hardware::new();

        while let Some(task) = receiver.recv().await {
            println!("Received task: {:?}", task);
            match task {
                Task::PlotShape {
                    shape,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    plot_shape(&mut hardware, &shape, &sample_settings, &plot_settings).await;
                    hardware.set_enabled(false);
                }
                Task::Plot {
                    layer,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    plot_layer(&mut hardware, &layer, &sample_settings, &plot_settings).await;
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
    println!(
        "Plotting shape: {:?} with settings {:?}",
        shape, plot_settings
    );
    let points = shape.get_points(sample_settings);
    let accelleration_path = AccellerationPath::new(&points, 0.1, 0.5); // TODO
    println!("num points: {:?}", points.len());
    println!("num speed points: {:?}", accelleration_path.points.len());

    let head_pressure = 0.5;

    // travel to start
    // TODO: travelToWithAccelleration
    hardware.set_head(false, head_pressure); // TODO
    hardware.move_to(0.0, &accelleration_path.points[0]); // TODO

    // draw
    hardware.set_head(true, head_pressure); // TODO
    hardware.set_enabled(true);
    for (from, to) in accelleration_path.points.iter().tuple_windows() {
        hardware.move_to(from.speed, to);
    }
    hardware.set_head(false, head_pressure); // TODO
}
