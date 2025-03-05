use dioxus::prelude::*;
use plottery_project::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::router::editor::{
    editor_console::EditorConsole, project_runner::ProjectRunner, running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct NumberFieldProps {
    param: ProjectParam,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<EditorConsole>,
    release: bool,
}

#[component]
pub fn NumberField(mut props: NumberFieldProps) -> Element {
    rsx! {
        style { { include_str!("number_field.css") } }
        div { class: "NumberField",
            input {
                name: "{props.param.name.clone()}",
                placeholder: "value",
                required: true,
                value: match props.param.value {
                    ProjectParamValue::Float(val) => val.to_string(),
                    ProjectParamValue::Int(val) => val.to_string(),
                    _ => panic!("Unexpected Error"),
                },
                onchange: move |event| {
                    let mut new_params = props.project_params.read().clone();
                    for param_field in new_params.list.iter_mut() {
                        if param_field.name == props.param.name.clone() {
                            let new_val = event.value().parse::<f32>();
                            match new_val {
                                Ok(val) => match param_field.value {
                                    ProjectParamValue::Float(_) => param_field.value.set_f32(val),
                                    ProjectParamValue::Int(_) => param_field.value.set_i32(val.round() as i32),
                                    _ => panic!("Unexpected Error"),
                                },
                                Err(e) => {
                                    log::error!("Error parsing value: {:?}", e);
                                    return;
                                }
                            }
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
