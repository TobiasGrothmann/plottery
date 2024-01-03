#![allow(non_snake_case)]
use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour};
use plottery_cli::Project;

use crate::{app_state::AppState, project_overview::ProjectOverview};
use log::LevelFilter;

mod app_state;
mod project_overview;

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
    let app_state = AppState::load().unwrap_or_else(|| {
        log::info!("App state file does not exist. Creating new app state.");
        let new_state = AppState::new();
        new_state.save();
        new_state
    });
    log::info!("App state contains {} projects", app_state.projects.len());

    cx.render(rsx! {
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
