use std::{path::Path, rc::Rc, time::Duration};

use dioxus::prelude::*;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::UnboundedSender;

use super::variations_state::reload_variations;
use crate::router::editor::editor_ctx::EditorCtx;

/// Reads the variations on mount and again whenever `variations/` changes, whoever rolls.
pub fn use_variations_watch(ctx: EditorCtx) {
    use_hook(move || {
        let project = ctx.project.peek().clone();
        let (variations, console) = (ctx.variations.variations, ctx.console);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let watcher = watch(&project.get_variations_dir(), tx);
        if let Err(e) = &watcher {
            console
                .peek()
                .error(&format!("Failed to watch the variations: {e}"));
        }
        spawn(async move {
            reload_variations(project.clone(), variations, console).await;
            while rx.recv().await.is_some() {
                // a roll writes several files at once
                tokio::time::sleep(Duration::from_millis(100)).await;
                while rx.try_recv().is_ok() {}
                reload_variations(project.clone(), variations, console).await;
            }
        });
        Rc::new(watcher.ok())
    });
}

fn watch(dir: &Path, tx: UnboundedSender<()>) -> notify::Result<RecommendedWatcher> {
    std::fs::create_dir_all(dir).map_err(notify::Error::io)?;
    let mut watcher = RecommendedWatcher::new(
        move |event: notify::Result<notify::Event>| {
            // half-written files are renamed into place, the rename is the change that counts
            let changed = event.is_ok_and(|event| {
                event
                    .paths
                    .iter()
                    .any(|path| path.extension().is_none_or(|extension| extension != "tmp"))
            });
            if changed {
                let _ = tx.send(());
            }
        },
        Config::default(),
    )?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}
