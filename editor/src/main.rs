#![allow(non_snake_case)]
use dioxus::{html::GlobalAttributes, prelude::*};
use dioxus_desktop::{
    tao::menu::{MenuBar, MenuItem},
    Config, LogicalSize, WindowBuilder, WindowCloseBehaviour,
};
use log::LevelFilter;
use plottery_project::Project;
use std::path::PathBuf;

use crate::{components::project_overview::ProjectOverview, model::app_state::AppState};

mod components;
mod model;
mod util;
mod util_test;

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
                    .with_focused(true)
                    .with_menu({
                        let mut menu = MenuBar::new();

                        let mut app_menu = MenuBar::new();
                        app_menu.add_native_item(MenuItem::Minimize);
                        app_menu.add_native_item(MenuItem::Hide);
                        app_menu.add_native_item(MenuItem::EnterFullScreen);
                        app_menu.add_native_item(MenuItem::Quit);

                        menu.add_submenu("Plottery Editor", true, app_menu);
                        menu
                    }),
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

    app_state.projects.push(
        Project::load_from_file(PathBuf::from(
            "/Users/admin/Dropbox/rust/plottery/project/test/test_project/test_project.plottery",
        ))
        .unwrap(),
    );
    // app_state.save();

    cx.render(rsx! {
        style { include_str!("./main.css") }
        div {
            h1 { "Projects" }
            main {
                div { class: "project_list",
                    app_state.projects.iter().map(|project| {
                        rsx! {
                            ProjectOverview {
                                project: project.clone()
                            }
                        }
                    })
                }
            }
        }
    })
}
