#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour};
use log::LevelFilter;
use plottery_cli::Project;
use std::path::PathBuf;

use crate::{components::project_overview::ProjectOverview, model::app_state::AppState};

mod components;
mod model;

fn main() {
    dioxus_logger::DioxusLogger::new(LevelFilter::Info)
        .use_format("[{LEVEL}] {PATH} - {ARGS}")
        .build()
        .expect("failed to init logger");

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
    log::info!("Loading app state");
    let mut app_state = AppState::load().unwrap_or_else(|| {
        log::info!("App state file does not exist. Creating new app state.");
        let new_state = AppState::new();
        new_state.save();
        new_state
    });
    log::info!("App state contains {} projects", app_state.projects.len());

    app_state.projects.push(
        Project::load_from_file(PathBuf::from(
            "/Users/admin/Dropbox/rust/plottery/cli/test/test_project/test_project.plottery",
        ))
        .unwrap(),
    );
    // app_state.save();

    cx.render(rsx! {
        style { include_str!("./main.css") }
        div {
            h1 { "Projects" }
            main {
                app_state.projects.iter().map(|project| {
                    rsx! {
                        ProjectOverview {
                            project: project.clone()
                        }
                    }
                })
            }
        }
    })
}
