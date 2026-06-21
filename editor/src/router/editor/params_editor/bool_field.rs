use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages, params_editor::param_tree::get_param_value_mut_by_path,
    project_runner::ProjectRunner, running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct BoolFieldProps {
    param: ProjectParam,
    path: Vec<String>,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

#[component]
pub fn BoolField(mut props: BoolFieldProps) -> Element {
    rsx! {
        style { { include_str!("bool_field.css") } }
        div { class: "BoolField",
            input {
                name: "{props.param.name.clone()}",
                type: "checkbox",
                required: true,
                value: match &props.param.value {
                    ProjectParamValue::Bool(val) => val.to_string(),
                    ProjectParamValue::Optional(optional) => match optional.value.as_ref() {
                        ProjectParamValue::Bool(val) => val.to_string(),
                        _ => panic!("Unexpected Error"),
                    },
                    _ => panic!("Unexpected Error"),
                },
                onchange: move |event| {
                    let mut new_params = props.project_params.read().clone();
                    if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &props.path) {
                        let new_val = event.value().parse().expect("Failed to parse boolean value");
                        match param_value {
                            ProjectParamValue::Bool(val) => *val = new_val,
                            ProjectParamValue::Optional(optional) => match optional.value.as_mut() {
                                ProjectParamValue::Bool(val) => *val = new_val,
                                _ => panic!("Unexpected Error"),
                            },
                            _ => panic!("Unexpected Error"),
                        }
                    } else {
                        tracing::error!("Param path not found: {:?}", props.path);
                        return;
                    }
                    props.project_params.set(new_params);
                    match props.project_runner.read().try_lock() {
                        Ok(mut runner) => {
                            runner.trigger_run_project(props.release, props.running_state, props.console);
                        },
                        Err(e) => {
                            tracing::error!("Error preparing to run: {:?}", e);
                            props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                        },
                    }
                }
            }
        }
    }
}
