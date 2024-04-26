#![allow(non_snake_case)]
use crate::routes::Route;
use dioxus::{
    desktop::{launch::launch, Config, LogicalSize, WindowBuilder, WindowCloseBehaviour},
    prelude::*,
};
use dioxus_router::prelude::*;
use log::LevelFilter;

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
    dioxus_logger::DioxusLogger::new(LevelFilter::Info)
        .use_format("[{LEVEL}] {PATH} - {ARGS}")
        .build()
        .expect("failed to init logger");

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
