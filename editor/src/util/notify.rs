use std::time::Duration;

use dioxus_logger::tracing;
use notify_rust::Notification;

use crate::{model::plot_watch::PlotEvent, util::format_plot_duration};

/// Shows a system notification without blocking the caller.
pub fn notify(title: &str, body: &str) {
    let (title, body) = (title.to_string(), body.to_string());
    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        set_sender();
        if let Err(err) = Notification::new().summary(&title).body(&body).show() {
            tracing::warn!("Cannot show notification '{title}': {err}");
        }
    });
}

/// Sends as the installed editor app; without one macOS shows Finder as the sender.
#[cfg(target_os = "macos")]
fn set_sender() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let bundle_id = notify_rust::get_bundle_identifier_or_default("Plottery Editor");
        notify_rust::set_application(&bundle_id).ok();
    });
}

/// Title and text of the notification for `event`; aborts are the user's own doing and stay silent.
pub fn plot_notification(event: PlotEvent) -> Option<(&'static str, String)> {
    match event {
        PlotEvent::Finished(duration) => Some(("Plot finished", took(duration))),
        PlotEvent::Lost => Some((
            "Plot interrupted",
            "The plotter server restarted before the plot finished.".to_string(),
        )),
        PlotEvent::Aborted => None,
    }
}

fn took(duration: Duration) -> String {
    format!("Took {}", format_plot_duration(duration))
}
