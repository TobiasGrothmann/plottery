use std::collections::BTreeSet;

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

/// Which view is shown, the stored variations of the open project and what to roll next.
#[derive(Clone, Copy)]
pub struct VariationsState {
    pub view: Signal<EditorView>,
    /// Read while the variations view is open, `None` when there are none.
    pub variations: SyncSignal<Option<Variations>>,
    /// Dotted paths of the params to roll.
    pub selected: Signal<BTreeSet<String>>,
    pub count: Signal<usize>,
    /// Id of the roll that is running.
    pub rolling: SyncSignal<Option<u64>>,
    /// Scroll position of the grid, kept while the edit view is shown.
    pub scroll_top: Signal<f64>,
}

pub fn use_variations_state() -> VariationsState {
    VariationsState {
        view: use_signal(|| EditorView::Edit),
        variations: use_signal_sync(|| None),
        selected: use_signal(BTreeSet::new),
        count: use_signal(|| 24),
        rolling: use_signal_sync(|| None),
        scroll_top: use_signal(|| 0.0),
    }
}

/// Reads the stored variations again, off the UI thread. Nothing selected yet selects what they rolled.
pub async fn reload_variations(
    project: Project,
    mut state: VariationsState,
    console: SyncSignal<ConsoleMessages>,
) {
    let loaded = match tokio::task::spawn_blocking(move || project.load_variations()).await {
        Ok(Ok(loaded)) => loaded,
        Ok(Err(e)) => {
            return console
                .peek()
                .error(&format!("Failed to read the variations: {e:#}"))
        }
        Err(e) => return tracing::error!("Reading the variations panicked: {e}"),
    };
    if let Some(variations) = &loaded {
        if state.selected.peek().is_empty() && state.rolling.peek().is_none() {
            state
                .selected
                .set(variations.meta.params.iter().cloned().collect());
            state
                .count
                .set(variations.rolls.len().saturating_sub(1).max(1));
        }
    }
    state.variations.set(loaded);
}
