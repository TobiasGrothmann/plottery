#![allow(non_snake_case)]
use crate::{
    components::remote::{PlotterRemote, RemoteOpen},
    model::window_state::WindowState,
    routes::Route,
};
use dioxus::{
    desktop::{
        tao::event::{Event, WindowEvent},
        use_window, use_wry_event_handler, Config, LogicalSize, WindowBuilder,
        WindowCloseBehaviour,
    },
    prelude::*,
};
use dioxus_router::*;

mod components;
mod model;
mod router;
mod routes;
mod util;

fn app() -> Element {
    let window = use_window();
    let history = dioxus::history::history();
    use_hook(|| {
        let Some(state) = WindowState::load() else {
            return;
        };
        // The builder position is taken as the content origin; set the frame position properly.
        window.set_outer_position(state.position());
        if let Some(route) = state.route.filter(|route| route.parse::<Route>().is_ok()) {
            history.push(route);
        }
    });
    let history = dioxus::history::history();
    use_wry_event_handler(move |event, _| {
        let closing = matches!(
            event,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } | Event::LoopDestroyed
        );
        if closing {
            if let Some(mut state) = WindowState::capture(&window.window) {
                state.route = Some(history.current_route());
                state.save();
            }
        }
    });
    use_context_provider(|| RemoteOpen(Signal::new(false)));

    rsx! {
        style { { include_str!("./main.css") } }
        style { { util::font_face_css() } }
        body {
            Router::<Route> {}
            PlotterRemote {}
        }
    }
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::DEBUG).expect("failed to init logger");

    let window = WindowBuilder::new()
        .with_title("Plottery Editor")
        .with_focused(true);
    // An explicit position and size also make dioxus skip its own debug-only restore.
    let window = match WindowState::load() {
        Some(state) => window
            .with_position(state.position())
            .with_inner_size(state.size()),
        None => window.with_inner_size(LogicalSize {
            width: 1400.0,
            height: 950.0,
        }),
    };

    let desktop_config = Config::default()
        .with_window(window)
        .with_close_behaviour(WindowCloseBehaviour::WindowCloses);

    LaunchBuilder::new()
        .with_cfg(desktop! {
            desktop_config
        })
        .launch(app);
}
