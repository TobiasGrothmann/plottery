#![allow(non_snake_case)]
use crate::{
    components::remote::{PlotterRemote, PlotterState, PlotterStatus, RemoteOpen},
    model::{
        app_state::{update_app_state, AppState},
        window_state::WindowState,
    },
    router::editor::console_messages::ActiveConsole,
    routes::Route,
    util::editor_route,
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
use plottery_project::Project;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

mod components;
mod model;
mod router;
mod routes;
mod util;

/// A project macOS asked to open before the app was mounted (cold start via `open -a`).
static PENDING_OPEN: Mutex<Option<PathBuf>> = Mutex::new(None);
static MOUNTED: AtomicBool = AtomicBool::new(false);

/// The first `.plottery` file of an open-document event.
fn opened_project<T>(event: &Event<'_, T>) -> Option<PathBuf> {
    let Event::Opened { urls } = event else {
        return None;
    };
    urls.iter()
        .filter_map(|url| url.to_file_path().ok())
        .find(|path| path.extension().is_some_and(|ext| ext == "plottery"))
}

/// Shows the project in the editor and selects its folder in the browser for later.
fn open_project_path(path: PathBuf, app_state: Signal<AppState>, console: ActiveConsole) {
    let project = match Project::load_from_file(path.clone()) {
        Ok(project) => project,
        Err(err) => {
            console.error(&format!("Cannot open project '{}': {err}", path.display()));
            return;
        }
    };
    update_app_state(app_state, |state| {
        state.ensure_root_covers(&project.dir);
        if let Some(parent) = project.dir.parent() {
            state.reveal_folder(parent);
        }
    });
    let route = editor_route(&project);
    // A plain history push doesn't re-render a mounted router; before mount the router reads it.
    match root_router() {
        Some(router) => {
            router.push(route);
        }
        None => dioxus::history::history().push(route.to_string()),
    }
}

fn app() -> Element {
    let window = use_window();
    let history = dioxus::history::history();
    let app_state = use_context_provider(|| Signal::new(AppState::load_or_default()));
    let console = use_context_provider(|| ActiveConsole(Signal::new(None)));
    use_hook(|| {
        MOUNTED.store(true, Ordering::SeqCst);
        if let Some(state) = WindowState::load() {
            // The builder position is taken as the content origin; set the frame position properly.
            window.set_outer_position(state.position());
            if let Some(route) = state.route.filter(|route| route.parse::<Route>().is_ok()) {
                history.push(route);
            }
        }
        // A requested project wins over the restored route.
        if let Some(path) = PENDING_OPEN.lock().unwrap().take() {
            open_project_path(path, app_state, console);
        }
    });
    let history = dioxus::history::history();
    use_wry_event_handler(move |event, _| {
        if let Some(path) = opened_project(event) {
            window.set_focus();
            open_project_path(path, app_state, console);
            return;
        }
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
    use_context_provider(|| PlotterState(Signal::new(PlotterStatus::Connecting)));

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
        .with_close_behaviour(WindowCloseBehaviour::WindowCloses)
        // Sees events before mount; once mounted, the app's own handler takes them.
        .with_custom_event_handler(|event, _| {
            if MOUNTED.load(Ordering::SeqCst) {
                return;
            }
            if let Some(path) = opened_project(event) {
                *PENDING_OPEN.lock().unwrap() = Some(path);
            }
        });

    LaunchBuilder::new()
        .with_cfg(desktop! {
            desktop_config
        })
        .launch(app);
}
