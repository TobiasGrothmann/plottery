use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::router::editor::running_state::RunningState;
use dioxus::signals::{ReadableExt, SyncSignal, WritableExt};
use plottery_project::{
    project_params_list_wrapper::ProjectParamsListWrapper, Project, RunError, RunEvent, RunStage,
};

use super::{console_messages::ConsoleMessages, editor::LayerChangeWrapper};

#[derive(Clone)]
pub struct ProjectRunner {
    project: Project,
    cancel_tx: Option<tokio::sync::mpsc::Sender<()>>,
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

impl ProjectRunner {
    pub fn new(
        project: Project,
        layer: SyncSignal<LayerChangeWrapper>,
        params: SyncSignal<ProjectParamsListWrapper>,
    ) -> Self {
        Self {
            project,
            cancel_tx: None,
            current_run: Arc::new(AtomicU64::new(0)),
            layer,
            params,
        }
    }

    pub fn trigger_run_project(
        &mut self,
        release: bool,
        mut running_state: SyncSignal<RunningState>,
        console: SyncSignal<ConsoleMessages>,
    ) {
        let run_id = self.current_run.fetch_add(1, Ordering::SeqCst) + 1;
        let current_run = self.current_run.clone();
        console.read().clear();
        self.cancel_tx.take(); // cancels the previous run if it exists

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);
        let project = self.project.clone();

        let mut layer = self.layer;
        let params = self.params;
        let saved_params = params.read().list.clone();

        console.read().info("spawning new task to run project");
        tokio::spawn(async move {
            let is_current = || current_run.load(Ordering::SeqCst) == run_id;
            // signals are Copy, copying them keeps the closure `Fn`
            let on_event = move |event: RunEvent| {
                if !is_current() {
                    return;
                }
                let (mut running_state, mut params) = (running_state, params);
                let (info, state) = match event {
                    RunEvent::StageStarting(stage) => stage_starting(stage),
                    RunEvent::StageRunning(stage) => stage_running(stage),
                    RunEvent::Output(line) => {
                        return console.read().project_message(line.as_str());
                    }
                    RunEvent::ParamsMerged(merged) => return params.set(merged),
                };
                console.read().info(info);
                running_state.set(state);
            };
            let cancel = async move {
                cancel_rx.recv().await;
            };

            match project
                .build_and_run_async(release, &saved_params, on_event, cancel)
                .await
            {
                Err(RunError::Killed { stage }) => {
                    console
                        .read()
                        .warn(format!("previous run aborted during {stage}").as_str());
                }
                _ if !is_current() => {}
                Ok((_, new_layer)) => {
                    console.read().info("...updating editor");
                    running_state.set(RunningState::Updating {
                        msg: "updating editor".to_string(),
                    });
                    let change_counter = layer.read().change_counter;
                    layer.set(LayerChangeWrapper {
                        layer: Some(new_layer),
                        change_counter: change_counter + 1,
                    });
                    console.read().info("done");
                    running_state.set(RunningState::Idle);
                }
                Err(RunError::Failed { stage, msg }) => {
                    console.read().error(msg.as_str());
                    running_state.set(failed_state(stage, msg));
                }
            }
        });
    }
}
