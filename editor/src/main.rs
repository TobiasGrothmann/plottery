#![allow(non_snake_case)]
use crate::routes::Route;
use dioxus::prelude::*;
use dioxus_desktop::{
    tao::menu::{MenuBar, MenuItem},
    Config, LogicalSize, WindowBuilder, WindowCloseBehaviour,
};
use dioxus_router::prelude::*;
use log::LevelFilter;

mod components;
mod model;
mod router_components;
mod routes;
mod util;
mod util_test;

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        style { include_str!("./main.css") }
        Router::<Route> {
        }
    })
}

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
                        width: 1400.0,
                        height: 950.0,
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
