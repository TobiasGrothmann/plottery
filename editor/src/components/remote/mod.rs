pub mod remote_modal;

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use dioxus::prelude::*;
use plottery_server_lib::{
    server_state::{request_server_state, ServerState, StateRequestError},
    Activity,
};
use tokio::sync::Notify;

use crate::{
    model::plot_watch::PlotWatch,
    util::notify::{notify, plot_notification},
};
use remote_modal::RemoteModal;

const RESTORE_FOCUS: &str = r#"
const el = window.__remoteReturnFocus;
if (el && el.isConnected && !document.querySelector('.scrim')) el.focus();
"#;

const POLL_FAST: Duration = Duration::from_secs(1);
const POLL_SLOW: Duration = Duration::from_secs(15);
const POLL_TIMEOUT: Duration = Duration::from_secs(1);
/// How long a poke keeps the fast cadence, enough for a sent plot to show up as active.
const POKE_BOOST: Duration = Duration::from_secs(20);

/// Whether the plotter remote modal is open; provided at the app root.
#[derive(Clone, Copy)]
pub struct RemoteOpen(pub Signal<bool>);

/// What the last poll of the plotter server returned; provided at the app root.
#[derive(Clone, Copy)]
pub struct PlotterState(pub Signal<PlotterStatus>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlotterStatus {
    Connecting,
    Offline,
    /// The server answered, but with a state this editor cannot read.
    DifferentVersion,
    Online(ServerState),
}

impl PlotterStatus {
    fn is_busy(&self) -> bool {
        matches!(self, Self::Online(state) if state.activity != Activity::Idle)
    }
}

static POLL_NOW: Notify = Notify::const_new();
static POKED_AT: Mutex<Option<Instant>> = Mutex::new(None);

/// Polls the plotter now and keeps polling fast for a while. Call after sending a task.
pub fn poke_plotter() {
    *POKED_AT.lock().unwrap() = Some(Instant::now());
    POLL_NOW.notify_one();
}

fn recently_poked() -> bool {
    POKED_AT
        .lock()
        .unwrap()
        .is_some_and(|at| at.elapsed() < POKE_BOOST)
}

/// Hosts the plotter remote modal, its global R shortcut and the one poll loop for the plotter's state.
#[component]
pub fn PlotterRemote() -> Element {
    let RemoteOpen(mut open) = use_context::<RemoteOpen>();
    let PlotterState(mut status) = use_context::<PlotterState>();

    use_future(move || async move {
        let mut eval = document::eval(include_str!("remote_shortcut.js"));
        while eval.recv::<String>().await.is_ok() {
            open.toggle();
        }
    });

    use_future(move || async move {
        let mut watch = PlotWatch::default();
        loop {
            let result = request_server_state(Some(POLL_TIMEOUT)).await;

            let (next_watch, event) = watch.observe(result.as_ref().ok());
            watch = next_watch;
            if let Some((title, body)) = event.and_then(plot_notification) {
                notify(title, &body);
            }

            let new_status = match result {
                Ok(state) => PlotterStatus::Online(state),
                Err(StateRequestError::DifferentVersion) => PlotterStatus::DifferentVersion,
                Err(StateRequestError::Unreachable(_)) => PlotterStatus::Offline,
            };
            if *status.peek() != new_status {
                status.set(new_status);
            }

            let fast =
                *open.peek() || new_status.is_busy() || watch.is_active() || recently_poked();
            let pause = if fast { POLL_FAST } else { POLL_SLOW };
            tokio::time::timeout(pause, POLL_NOW.notified()).await.ok();
        }
    });

    use_effect(move || {
        if open() {
            POLL_NOW.notify_one();
        } else {
            document::eval(RESTORE_FOCUS);
        }
    });

    rsx! {
        if open() {
            RemoteModal { open }
        }
    }
}
