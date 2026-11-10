use std::path::{Path, PathBuf};

use dioxus::signals::{ReadableExt, SyncSignal, WritableExt};
use dioxus_logger::tracing;
use notify::{Config, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use plottery_project::project_params_list_wrapper::ProjectParamsListWrapper;
use tokio::task::JoinHandle;

use super::{
    console_messages::ConsoleMessages, params_file::ParamsFile, project_runner::ProjectRunner,
    running_state::RunningState,
};

/// Reruns on source changes, and on params changed by someone else, e.g. `plottery set`.
pub fn start_hot_reload(
    path_to_watch: PathBuf,
    release: bool,
    project_runner: ProjectRunner,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    params_file: ParamsFile,
    mut project_params: SyncSignal<ProjectParamsListWrapper>,
) -> (JoinHandle<()>, FsEventWatcher) {
    // async channel: a blocking `recv` would hold a runtime thread and stall the runs spawned below
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = RecommendedWatcher::new(
        move |event: notify::Result<notify::Event>| {
            let _ = tx.send(event);
        },
        Config::default(),
    )
    .unwrap();
    watcher
        .watch(&path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    // the editor writes its layer and previews here too, only the params file counts
    let params_path = params_file.path();
    let resource_dir = params_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if let Err(e) = std::fs::create_dir_all(&resource_dir)
        .map_err(notify::Error::io)
        .and_then(|_| watcher.watch(&resource_dir, RecursiveMode::NonRecursive))
    {
        tracing::error!("Failed to watch {:?}: {:?}", resource_dir, e);
    }
    // events carry resolved paths, e.g. /private/var for /var
    let resource_dirs = [resource_dir.canonicalize().ok(), Some(resource_dir)];

    let hot_reload_handle = tokio::spawn(async move {
        let in_resource_dir = |path: &Path| {
            path.parent()
                .is_some_and(|parent| resource_dirs.iter().flatten().any(|dir| dir == parent))
        };
        let is_params_file =
            |path: &Path| in_resource_dir(path) && path.file_name() == params_path.file_name();

        while let Some(event) = rx.recv().await {
            match event {
                Ok(event) => {
                    if event.paths.iter().any(|p| is_params_file(p)) {
                        match params_file.read_external() {
                            Ok(Some(params)) => {
                                // the runner reads params from the signal when it starts
                                if let Ok(mut current) = project_params.try_write() {
                                    *current = params;
                                }
                                tracing::info!("Params changed on disk");
                                project_runner.trigger_run_project(release, running_state, console);
                                if let Ok(console) = console.try_read() {
                                    console.info("params changed on disk");
                                }
                            }
                            Ok(None) => {}
                            Err(e) => tracing::error!("Failed to read changed params: {:?}", e),
                        }
                        continue;
                    }
                    if event.paths.iter().all(|p| in_resource_dir(p)) {
                        continue;
                    }

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

                    tracing::info!("Hot reload triggered");
                    project_runner.trigger_run_project(release, running_state, console);
                }
                Err(e) => tracing::error!("Hot reload error: {:?}", e),
            }
        }
        tracing::info!("Hot reload thread finished");
    });

    (hot_reload_handle, watcher)
}
