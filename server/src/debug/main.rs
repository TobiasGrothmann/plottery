use clap::{Parser, Subcommand};
use plottery_lib::*;
use plottery_server_lib::{plot_settings::PlotSettings, task::Task};
use reqwest::Client;

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Rect,
    Circle,
    Layer,
}

#[derive(Parser, Debug)]
#[command(about="Debugging tool for plottery_server", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

static URL: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = Client::new();
    let sample_settings = SampleSettings::default();
    let plot_settings = PlotSettings {
        accelleration_dist: 0.1,
        corner_slowdown_power: 0.5,
    };

    match args.command {
        Command::Rect => {
            let shape = Rect::new_shape(V2::xy(0.0), V2::xy(1.0));
            let task = Task::PlotShape {
                shape,
                sample_settings,
                plot_settings,
            };
            send_task(&client, task).await;
        }
        Command::Circle => {
            let shape = Circle::new_shape(V2::xy(0.0), 1.0);
            let task = Task::PlotShape {
                shape,
                sample_settings,
                plot_settings,
            };
            send_task(&client, task).await;
        }
        Command::Layer => {
            let layer = Layer::new_from(vec![
                Rect::new_shape(V2::xy(0.0), V2::xy(1.0)),
                Circle::new_shape(V2::xy(0.0), 1.0),
            ]);
            let task = Task::Plot {
                layer,
                sample_settings,
                plot_settings,
            };
            send_task(&client, task).await;
        }
    }
}

async fn send_task(client: &Client, task: Task) {
    let body = task.to_base64().unwrap();
    client
        .post(&format!("http://{}/task", URL))
        .body(body)
        .send()
        .await
        .unwrap();
}
