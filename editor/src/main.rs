#![allow(non_snake_case)]
use crate::routes::Route;
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour},
    prelude::*,
};
use dioxus_router::*;

mod components;
mod model;
mod router;
mod routes;
mod util;

fn app() -> Element {
    rsx! {
        style { { include_str!("./main.css") } }
        body {
            Router::<Route> {}
        }
    }
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::DEBUG).expect("failed to init logger");

    log::info!("🚀 Plottery Editor starting - logging is working!");

    let desktop_config = Config::default()
        .with_window(
            WindowBuilder::new()
                .with_title("Plottery Editor")
                .with_inner_size(LogicalSize {
                    width: 1400.0,
                    height: 950.0,
                })
                .with_focused(true),
        )
        .with_close_behaviour(WindowCloseBehaviour::WindowCloses);

    LaunchBuilder::new()
        .with_cfg(desktop! {
            desktop_config
        })
        .launch(app);
}
