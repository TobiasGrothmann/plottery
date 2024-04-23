use std::sync::Arc;

use dioxus::prelude::*;
use plottery_project::{ProjectParamValue, ProjectParamsListWrapper};
use tokio::sync::Mutex;

use crate::router_components::editor_components::running_state::RunningState;

use super::project_runner::ProjectRunner;

#[derive(PartialEq, Props, Clone)]
pub struct ParamsEditorProps {
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    release: bool,
}

#[component]
pub fn ParamsEditor(mut props: ParamsEditorProps) -> Element {
    rsx! {
        style { { include_str!("params_editor.css") } }
        div { class: "ParamsEditor",
            for param in props.project_params.read().list.iter().cloned() {
                div { class: "param",
                    p { "{param.name.clone()}" }

                    match param.value {
                        ProjectParamValue::Float(_) | ProjectParamValue::Int(_) => {
                            // value
                            rsx! {
                                input {
                                    name: "{param.name.clone()}",
                                    placeholder: "value",
                                    required: true,
                                    value: match param.value {
                                        ProjectParamValue::Float(val) => val.to_string(),
                                        ProjectParamValue::Int(val) => val.to_string(),
                                        _ => panic!("Unexpected Error"),
                                    },
                                    onchange: move |event| {
                                        let mut new_params = props.project_params.read().clone();
                                        for param_field in new_params.list.iter_mut() {
                                            if param_field.name == param.name.clone() {
                                                let new_val = event.value().parse().unwrap();
                                                match param_field.value {
                                                    ProjectParamValue::Float(_) => param_field.value.set_f32(new_val),
                                                    ProjectParamValue::Int(_) => param_field.value.set_i32(new_val.round() as i32),
                                                    _ => panic!("Unexpected Error"),
                                                }
                                            }
                                        }
                                        props.project_params.set(new_params);
                                        match props.project_runner.read().try_lock() {
                                            Ok(mut runner) => runner.trigger_run_project(props.release, props.running_state),
                                            Err(e) => {
                                                log::error!("Error preparing to run: {:?}", e);
                                                props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                            },
                                        }
                                    }
                                }
                            }
                        }
                        ProjectParamValue::FloatRanged { .. } | ProjectParamValue::IntRanged { .. } => {
                            // slider
                            rsx! {
                                input {
                                    class: "slider",
                                    name: "{param.name.clone()}",
                                    required: true,
                                    r#type: "range",
                                    step: "0.01",
                                    min: match param.value {
                                        ProjectParamValue::FloatRanged { val: _, min, max: _ } => min.to_string(),
                                        ProjectParamValue::IntRanged { val: _, min, max: _ } => min.to_string(),
                                        _ => panic!("Unexpected Error"),
                                    },
                                    max: match param.value {
                                        ProjectParamValue::FloatRanged { val: _, min: _, max } => max.to_string(),
                                        ProjectParamValue::IntRanged { val: _, min: _, max } => max.to_string(),
                                        _ => panic!("Unexpected Error"),
                                    },
                                    value: match param.value {
                                        ProjectParamValue::FloatRanged { val, min: _, max: _ } => val.to_string(),
                                        ProjectParamValue::IntRanged { val, min: _, max: _ } => val.to_string(),
                                        _ => panic!("Unexpected Error"),
                                    },
                                    onchange: move |event| {
                                        let mut new_params = props.project_params.read().clone();
                                        for param_field in new_params.list.iter_mut() {
                                            if param_field.name == param.name.clone() {
                                                let new_val = event.value().parse().unwrap();
                                                match param_field.value {
                                                    ProjectParamValue::FloatRanged { .. } => param_field.value.set_f32(new_val),
                                                    ProjectParamValue::IntRanged { .. } => param_field.value.set_i32(new_val.round() as i32),
                                                    _ => panic!("Unexpected Error"),
                                                }
                                            }
                                        }
                                        props.project_params.set(new_params);
                                        match props.project_runner.read().try_lock() {
                                            Ok(mut runner) => runner.trigger_run_project(props.release, props.running_state),
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
                }
            }
        }
    }
}
