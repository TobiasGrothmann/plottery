use plottery_lib::*;
use plottery_server_lib::{plot_settings::PlotSettings, task::Task};
use tokio::sync::mpsc;
use tokio::task;

use crate::accelleration_path::AccellerationPath;

pub async fn start_server(mut receiver: mpsc::Receiver<Task>) {
    task::spawn(async move {
        while let Some(task) = receiver.recv().await {
            println!("Received task: {:?}", task);
            match task {
                Task::PlotShape {
                    shape,
                    sample_settings,
                    plot_settings,
                } => {
                    plot_shape(&shape, &sample_settings, &plot_settings).await;
                }
                Task::Plot {
                    layer,
                    sample_settings,
                    plot_settings,
                } => {
                    plot_layer(&layer, &sample_settings, &plot_settings).await;
                }
                Task::Abort => {
                    todo!()
                }
            }
        }
    });
}

pub async fn plot_layer(
    layer: &Layer,
    sample_settings: &SampleSettings,
    plot_settings: &PlotSettings,
) {
    println!("Plotting layer: {:?}", layer);
    for shape in layer.shapes.iter() {
        plot_shape(shape, sample_settings, plot_settings).await;
    }
}

pub async fn plot_shape(
    shape: &Shape,
    sample_settings: &SampleSettings,
    plot_settings: &PlotSettings,
) {
    println!(
        "Plotting shape: {:?} with settings {:?}",
        shape, plot_settings
    );
    let points = shape.get_points(sample_settings);
    let accelleration_path = AccellerationPath::new(&points, 0.1, 0.5);
    println!("num points: {:?}", points.len());
    println!("num speed points: {:?}", accelleration_path.points.len());
}
