use dioxus::prelude::*;
use plottery_project::Project;
use plottery_runtime::project_params_list_wrapper::ProjectParamsListWrapper;

use super::{
    console_messages::ConsoleMessages,
    project_runner::{trigger_run, ProjectRunner},
    running_state::RunningState,
};

/// What the panels and param widgets of the open project share, provided as context by the editor.
#[derive(Clone, Copy)]
pub struct EditorCtx {
    pub project: Signal<Project>,
    pub project_params: SyncSignal<ProjectParamsListWrapper>,
    pub project_runner: SyncSignal<ProjectRunner>,
    pub running_state: SyncSignal<RunningState>,
    pub console: SyncSignal<ConsoleMessages>,
    pub release: bool,
}

impl EditorCtx {
    pub fn trigger_run(&self) {
        trigger_run(
            self.project_runner,
            self.running_state,
            self.console,
            self.release,
        );
    }
}
