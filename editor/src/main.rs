#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour};

fn main() {
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
