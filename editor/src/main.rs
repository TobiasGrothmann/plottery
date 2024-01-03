#![allow(non_snake_case)]
use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour};
use plottery_cli::Project;

use crate::app_state::AppState;
use log::LevelFilter;

mod app_state;

fn main() {
    dioxus_logger::DioxusLogger::new(LevelFilter::Info)
        .use_format("[{LEVEL}] {PATH} - {ARGS}")
        .build()
        .expect("failed to init logger");

    log::info!("Loading app state");
    let mut app_state = AppState::load().unwrap_or_else(|| {
        log::info!("App state file does not exist. Creating new app state.");
        let new_state = AppState::new();
        new_state.save();
        new_state
    });

    app_state
        .projects
        .push(Project::new(PathBuf::from("."), "nonexisting".to_string()));

    dioxus_desktop::launch_cfg(
        App,
        Config::default()
            .with_window(
                WindowBuilder::new()
                    .with_title("Plottery Editor")
                    .with_inner_size(LogicalSize {
                        width: 1300.0,
                        height: 800.0,
                    })
                    .with_focused(true),
            )
            .with_close_behaviour(WindowCloseBehaviour::CloseWindow),
    );
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            h1 { "Projects" }
            ul {
                li { "Project 1" }
                li { "Project 2" }
                li { "Project 3" }
                li { "Project 4" }
                li { "Project 5" }
                li { "Project 6" }
                li { "Project 7" }
                li { "Project 8" }
            }
        }
    })
}
