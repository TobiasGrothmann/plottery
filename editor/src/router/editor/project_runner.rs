use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use crate::router::editor::running_state::RunningState;
use dioxus::signals::{ReadableExt, SyncSignal, WritableExt};
use dioxus_logger::tracing;
use plottery_project::{
    project_params_list_wrapper::ProjectParamsListWrapper, Project, RunError, RunEvent, RunStage,
};

use super::{console_messages::ConsoleMessages, editor::LayerChangeWrapper};

/// Clones share the run state, so hot reload and the UI cancel each other's runs.
#[derive(Clone)]
pub struct ProjectRunner {
    project: Project,
    cancel_tx: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
    /// Id of the latest run, older runs leave the editor alone.
    current_run: Arc<AtomicU64>,
    layer: SyncSignal<LayerChangeWrapper>,
    params: SyncSignal<ProjectParamsListWrapper>,
}

fn stage_starting(stage: RunStage) -> (&'static str, RunningState) {
    match stage {
        RunStage::Build => (
            "...starting build",
            RunningState::StartingBuild {
                msg: "starting build".to_string(),
            },
        ),
        RunStage::GetParams => (
            "...starting getting params",
            RunningState::StartingGetParams {
                msg: "starting getting params".to_string(),
            },
        ),
        RunStage::Run => (
            "...starting run",
            RunningState::StartingRun {
                msg: "starting run".to_string(),
            },
        ),
    }
}

fn stage_running(stage: RunStage) -> (&'static str, RunningState) {
    match stage {
        RunStage::Build => (
            "...building",
            RunningState::Building {
                msg: "building".to_string(),
            },
        ),
        RunStage::GetParams => (
            "...getting params",
            RunningState::GetParams {
                msg: "getting params".to_string(),
            },
        ),
        RunStage::Run => (
            "...running",
            RunningState::Running {
                msg: "running".to_string(),
            },
        ),
    }
}

fn failed_state(stage: RunStage, msg: String) -> RunningState {
    match stage {
        RunStage::Build => RunningState::BuildFailed { msg },
        RunStage::GetParams => RunningState::GetParamsFailed { msg },
        RunStage::Run => RunningState::RunFailed { msg },
    }
}

/// Signals can be dropped while a run is in flight, e.g. after leaving the editor.
fn log(console: SyncSignal<ConsoleMessages>, write: impl FnOnce(&ConsoleMessages)) {
    if let Ok(console) = console.try_read() {
        write(&console);
    }
}

fn set_state(mut running_state: SyncSignal<RunningState>, state: RunningState) {
    if let Ok(mut current) = running_state.try_write() {
        *current = state;
    }
}

/// Starts a run of the editor's project, cancelling the previous one.
pub fn trigger_run(
    project_runner: SyncSignal<ProjectRunner>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
) {
    match project_runner.try_read() {
        Ok(runner) => runner.trigger_run_project(release, running_state, console),
        Err(e) => tracing::error!("Project runner unavailable: {:?}", e),
    }
}

impl ProjectRunner {
    pub fn new(
        project: Project,
        layer: SyncSignal<LayerChangeWrapper>,
        params: SyncSignal<ProjectParamsListWrapper>,
    ) -> Self {
        Self {
            project,
            cancel_tx: Arc::new(Mutex::new(None)),
            current_run: Arc::new(AtomicU64::new(0)),
            layer,
            params,
        }
    }

    pub fn trigger_run_project(
        &self,
        release: bool,
        running_state: SyncSignal<RunningState>,
        console: SyncSignal<ConsoleMessages>,
    ) {
        set_state(
            running_state,
            RunningState::Preparing {
                msg: "preparing".to_string(),
            },
        );
        let run_id = self.current_run.fetch_add(1, Ordering::SeqCst) + 1;
        let current_run = self.current_run.clone();
        log(console, |c| c.clear());

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        // replacing the sender drops the previous one, which cancels its run
        *self
            .cancel_tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(cancel_tx);
        let project = self.project.clone();

        let layer = self.layer;
        let params = self.params;
        let Ok(saved_params) = params.try_read().map(|p| p.list.clone()) else {
            return;
        };

        log(console, |c| c.info("spawning new task to run project"));
        tokio::spawn(async move {
            let is_current = || current_run.load(Ordering::SeqCst) == run_id;
            let on_event = move |event: RunEvent| {
                if !is_current() {
                    return;
                }
                let (info, state) = match event {
                    RunEvent::StageStarting(stage) => stage_starting(stage),
                    RunEvent::StageRunning(stage) => stage_running(stage),
                    RunEvent::Output(line) => {
                        return log(console, |c| c.project_message(line.as_str()));
                    }
                    RunEvent::ParamsMerged(merged) => {
                        let mut params = params;
                        if let Ok(mut current) = params.try_write() {
                            *current = merged;
                        }
                        return;
                    }
                };
                log(console, |c| c.info(info));
                set_state(running_state, state);
            };
            let cancel = async move {
                cancel_rx.recv().await;
            };

            match project
                .build_and_run_async(release, &saved_params, on_event, cancel)
                .await
            {
                Err(RunError::Killed { stage }) => {
                    log(console, |c| {
                        c.warn(format!("previous run aborted during {stage}").as_str())
                    });
                }
                _ if !is_current() => {}
                Ok((_, new_layer)) => {
                    log(console, |c| c.info("...updating editor"));
                    set_state(
                        running_state,
                        RunningState::Updating {
                            msg: "updating editor".to_string(),
                        },
                    );
                    let mut layer = layer;
                    if let Ok(mut current) = layer.try_write() {
                        current.layer = Some(new_layer);
                        current.change_counter += 1;
                    }
                    log(console, |c| c.info("done"));
                    set_state(running_state, RunningState::Idle);
                }
                Err(RunError::Failed { stage, msg }) => {
                    log(console, |c| c.error(msg.as_str()));
                    set_state(running_state, failed_state(stage, msg));
                }
            }
        });
    }
}
