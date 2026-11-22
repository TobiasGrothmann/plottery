#![allow(non_snake_case)]
use crate::{
    components::remote::{PlotterRemote, PlotterState, PlotterStatus, RemoteOpen},
    model::{
        app_state::{update_app_state, AppState},
        open_request::{OpenRequest, RequestedView},
        window_state::WindowState,
    },
    menu::{use_menu_action, MenuAction},
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
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

mod components;
mod menu;
mod model;
mod router;
mod routes;
mod util;

/// A project macOS asked to open before the app was mounted (cold start via `open -a`).
static PENDING_OPEN: Mutex<Option<OpenRequest>> = Mutex::new(None);
static MOUNTED: AtomicBool = AtomicBool::new(false);

/// The first project file of an open-document event, see [`OpenRequest`].
fn opened_project<T>(event: &Event<'_, T>) -> Option<OpenRequest> {
    let Event::Opened { urls } = event else {
        return None;
    };
    urls.iter()
        .filter_map(|url| url.to_file_path().ok())
        .find_map(|path| OpenRequest::from_file(&path))
}

/// Shows the project in the requested view and selects its folder in the browser for later.
fn open_project_path(
    request: OpenRequest,
    app_state: Signal<AppState>,
    console: ActiveConsole,
    RequestedView(mut requested_view): RequestedView,
) {
    let path = request.config_path;
    let project = match Project::load_from_file(path.clone()) {
        Ok(project) => project,
        Err(err) => {
            console.error(&format!("Cannot open project '{}': {err}", path.display()));
            return;
        }
    };
    requested_view.set(Some((project.dir.clone(), request.view)));
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
    let requested_view = use_context_provider(|| RequestedView(Signal::new(None)));
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
        if let Some(request) = PENDING_OPEN.lock().unwrap().take() {
            open_project_path(request, app_state, console, requested_view);
        }
    });
    let history = dioxus::history::history();
    use_wry_event_handler(move |event, _| {
        if let Some(request) = opened_project(event) {
            window.set_focus();
            open_project_path(request, app_state, console, requested_view);
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
    let RemoteOpen(mut remote_open) = use_context_provider(|| RemoteOpen(Signal::new(false)));
    use_menu_action(move |action| {
        if action == MenuAction::PlotterRemote {
            remote_open.toggle();
        }
    });
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
        .with_menu(menu::menu_bar())
        .with_close_behaviour(WindowCloseBehaviour::WindowCloses)
        // Sees events before mount; once mounted, the app's own handler takes them.
        .with_custom_event_handler(|event, _| {
            if MOUNTED.load(Ordering::SeqCst) {
                return;
            }
            if let Some(request) = opened_project(event) {
                *PENDING_OPEN.lock().unwrap() = Some(request);
            }
        });

    LaunchBuilder::new()
        .with_cfg(desktop! {
            desktop_config
        })
        .launch(app);
}
