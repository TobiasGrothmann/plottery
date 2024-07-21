#![allow(non_snake_case)]
use crate::routes::Route;
use dioxus::desktop::{launch::launch, Config, LogicalSize, WindowBuilder, WindowCloseBehaviour};
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use dioxus_router::prelude::*;

mod components;
mod model;
mod router;
mod routes;
mod util;

fn App() -> Element {
    rsx! {
        style { { include_str!("./main.css") } }
        body {
            Router::<Route> {}
        }
    }
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("Failed to initialize logger");

    let config = Config::default()
        .with_window(
            WindowBuilder::new()
                .with_title("Plottery Editor")
                .with_inner_size(LogicalSize {
                    width: 1400.0,
                    height: 950.0,
                })
                .with_focused(true),
        )
        .with_close_behaviour(WindowCloseBehaviour::CloseWindow);

    launch(App, vec![], config);
}
