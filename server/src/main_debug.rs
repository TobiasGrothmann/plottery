pub mod base64;
use clap::Parser;
use clap::Subcommand;
use plottery_lib::*;

#[derive(Debug, Clone, Subcommand)]
enum Command {
    Rect,
    Circle,
    Layer,
}

#[derive(Parser, Debug)]
#[command(about="Debugging tool", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

static URL: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = reqwest::Client::new();

    match args.command {
        Command::Rect => {
            let shape = Rect::new_shape(V2::xy(0.0), V2::xy(1.0))
                .to_base64()
                .unwrap();
            client
                .post(&format!("http://{}/plot_shape", URL))
                .body(shape)
                .send()
                .await
                .unwrap();
        }
        Command::Circle => {
            let shape = Circle::new_shape(V2::xy(0.0), 1.0).to_base64().unwrap();
            client
                .post(&format!("http://{}/plot_shape", URL))
                .body(shape)
                .send()
                .await
                .unwrap();
        }
        Command::Layer => {
            let layer = Layer::new_from(vec![
                Rect::new_shape(V2::xy(0.0), V2::xy(1.0)),
                Circle::new_shape(V2::xy(0.0), 1.0),
            ])
            .to_base64()
            .unwrap();
            client
                .post(&format!("http://{}/plot", URL))
                .body(layer)
                .send()
                .await
                .unwrap();
        }
    }
}
