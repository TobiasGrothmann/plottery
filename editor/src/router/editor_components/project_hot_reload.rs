use std::{path::PathBuf, sync::Arc};

use dioxus::signals::SyncSignal;
use notify::{Config, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{sync::Mutex, task::JoinHandle};

use super::{
    editor_console::EditorConsole, project_runner::ProjectRunner, running_state::RunningState,
};

pub fn start_hot_reload(
    path_to_watch: PathBuf,
    release: bool,
    project_runner: Arc<Mutex<ProjectRunner>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<EditorConsole>,
) -> (JoinHandle<()>, FsEventWatcher) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher
        .watch(&path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    let hot_reload_handle = tokio::spawn(async move {
        for event in rx {
            match event {
                Ok(event) => {
                    let ignore_list = [".DS_Store"];
                    let changed_paths = event.paths.iter().filter(|p| {
                        p.file_name()
                            .map(|s| !ignore_list.contains(&s.to_str().unwrap()))
                            .unwrap_or(false)
                    });
                    if changed_paths.count() == 0 {
                        continue;
                    }

                    let hot_reload_required = match event.kind {
                        notify::EventKind::Any => false,
                        notify::EventKind::Access(_) => false,
                        notify::EventKind::Create(_) => true,
                        notify::EventKind::Modify(modifyKind) => match modifyKind {
                            notify::event::ModifyKind::Any => false,
                            notify::event::ModifyKind::Data(_) => true,
                            notify::event::ModifyKind::Metadata(_) => false,
                            notify::event::ModifyKind::Name(_) => true,
                            notify::event::ModifyKind::Other => true,
                        },
                        notify::EventKind::Remove(_) => true,
                        notify::EventKind::Other => false,
                    };
                    if !hot_reload_required {
                        continue;
                    }

                    log::info!("Hot reload triggered");
                    project_runner.lock().await.trigger_run_project(
                        release,
                        running_state,
                        console,
                    );
                }
                Err(e) => log::error!("Hot reload error: {:?}", e),
            }
        }
        log::info!("Hot reload thread finished");
    });

    (hot_reload_handle, watcher)
}
