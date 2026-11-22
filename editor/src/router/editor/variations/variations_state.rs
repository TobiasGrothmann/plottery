use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{Project, Variations};

use crate::router::editor::console_messages::ConsoleMessages;

/// What the open project shows: its drawing to edit, or its variations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorView {
    Edit,
    Variations,
}

/// Which view is shown and the stored variations of the open project.
#[derive(Clone, Copy)]
pub struct VariationsState {
    pub view: Signal<EditorView>,
    /// Read while the variations view is open, `None` when there are none.
    pub variations: SyncSignal<Option<Variations>>,
}

pub fn use_variations_state() -> VariationsState {
    VariationsState {
        view: use_signal(|| EditorView::Edit),
        variations: use_signal_sync(|| None),
    }
}

/// Reads the stored variations again, off the UI thread.
pub async fn reload_variations(
    project: Project,
    mut variations: SyncSignal<Option<Variations>>,
    console: SyncSignal<ConsoleMessages>,
) {
    match tokio::task::spawn_blocking(move || project.load_variations()).await {
        Ok(Ok(loaded)) => variations.set(loaded),
        Ok(Err(e)) => console
            .peek()
            .error(&format!("Failed to read the variations: {e:#}")),
        Err(e) => tracing::error!("Reading the variations panicked: {e}"),
    }
}
