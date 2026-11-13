use std::rc::Rc;

use dioxus::{
    html::{geometry::PixelsVector2D, MountedData, ScrollBehavior},
    prelude::*,
};
use dioxus_logger::tracing;
use plottery_project::{Project, ScanResult};

/// Scroll offset of the project grid, kept across navigation so going back lands where you left.
static SCROLL_TOP: GlobalSignal<f64> = Signal::global(|| 0.0);

/// Project search query, kept across navigation but not across restarts.
pub static SEARCH_QUERY: GlobalSignal<String> = Signal::global(String::new);

/// Renders missing or outdated thumbnails after each scan, one project at a time. The returned
/// counter is bumped after each one, so cards re-check their files and fill in as they are done.
pub fn use_thumbnail_generation(scan: Resource<ScanResult>) -> Signal<u32> {
    let mut generation = use_signal(|| 0u32);
    let _render = use_resource(move || {
        let projects: Vec<Project> = scan
            .read()
            .as_ref()
            .map(|r| r.projects.clone())
            .unwrap_or_default();
        async move {
            let stale = projects
                .into_iter()
                .filter(Project::editor_thumbnail_is_stale);
            for project in stale {
                let rendered = tokio::task::spawn_blocking(move || {
                    project
                        .update_editor_thumbnail()
                        .inspect_err(|err| {
                            tracing::error!(
                                "Failed to render thumbnail for {}: {err}",
                                project.dir.display()
                            )
                        })
                        .is_ok()
                })
                .await
                .unwrap_or(false);
                if rendered {
                    generation += 1;
                }
            }
        }
    });
    generation
}

/// Remembers the scroll position of an element across navigation and restores it once `ready`.
#[derive(Clone, Copy)]
pub struct ScrollRestore {
    element: Signal<Option<Rc<MountedData>>>,
}

impl ScrollRestore {
    pub fn on_mounted(mut self, event: MountedEvent) {
        self.element.set(Some(event.data()));
    }

    pub fn on_scroll(self, event: ScrollEvent) {
        *SCROLL_TOP.write() = event.data().scroll_top();
    }

    pub fn scroll_to_top(self) {
        *SCROLL_TOP.write() = 0.0;
        if let Some(element) = self.element.peek().clone() {
            spawn(async move {
                let _ = element
                    .scroll(PixelsVector2D::new(0.0, 0.0), ScrollBehavior::Instant)
                    .await;
            });
        }
    }
}

pub fn use_scroll_restore(ready: Memo<bool>) -> ScrollRestore {
    let element: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let mut restored = use_signal(|| false);
    use_effect(move || {
        let Some(element) = element() else {
            return;
        };
        if !ready() || restored() {
            return;
        }
        restored.set(true);
        let top = *SCROLL_TOP.peek();
        if top > 0.0 {
            spawn(async move {
                let _ = element
                    .scroll(PixelsVector2D::new(0.0, top), ScrollBehavior::Instant)
                    .await;
            });
        }
    });
    ScrollRestore { element }
}
