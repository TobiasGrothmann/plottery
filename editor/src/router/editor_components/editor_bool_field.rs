use dioxus::prelude::*;
use plottery_project::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    editor_console::EditorConsole, project_runner::ProjectRunner, running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct EditorBoolFieldProps {
    param: ProjectParam,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<EditorConsole>,
    release: bool,
}

#[component]
pub fn EditorBoolField(mut props: EditorBoolFieldProps) -> Element {
    rsx! {
        style { { include_str!("editor_bool_field.css") } }
        div { class: "EditorBoolField",
            input {
                name: "{props.param.name.clone()}",
                type: "checkbox",
                required: true,
                value: match props.param.value {
                    ProjectParamValue::Bool(val) => val.to_string(),
                    _ => panic!("Unexpected Error"),
                },
                onchange: move |event| {
                    let mut new_params = props.project_params.read().clone();
                    for param_field in new_params.list.iter_mut() {
                        if param_field.name == props.param.name.clone() {
                            param_field.value.set_bool(event.value() == "true");
                        }
                    }
                    props.project_params.set(new_params);
                    match props.project_runner.read().try_lock() {
                        Ok(mut runner) => {
                            runner.trigger_run_project(props.release, props.running_state, props.console);
                        },
                        Err(e) => {
                            log::error!("Error preparing to run: {:?}", e);
                            props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                        },
                    }
                }
            }
        }
    }
}
