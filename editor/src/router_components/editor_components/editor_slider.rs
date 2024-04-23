use std::sync::Arc;

use dioxus::prelude::*;
use plottery_project::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};
use tokio::sync::Mutex;

use super::{project_runner::ProjectRunner, running_state::RunningState};

#[derive(PartialEq, Props, Clone)]
pub struct EditorSliderProps {
    param: ProjectParam,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    release: bool,
}

#[component]
pub fn EditorSlider(mut props: EditorSliderProps) -> Element {
    rsx! {
        div { class: "EditorSlider",
            input {
                class: "slider",
                name: "{props.param.name.clone()}",
                required: true,
                r#type: "range",
                step: "0.01",
                min: match props.param.value {
                    ProjectParamValue::FloatRanged { val: _, min, max: _ } => min.to_string(),
                    ProjectParamValue::IntRanged { val: _, min, max: _ } => min.to_string(),
                    _ => panic!("Unexpected Error"),
                },
                max: match props.param.value {
                    ProjectParamValue::FloatRanged { val: _, min: _, max } => max.to_string(),
                    ProjectParamValue::IntRanged { val: _, min: _, max } => max.to_string(),
                    _ => panic!("Unexpected Error"),
                },
                value: match props.param.value {
                    ProjectParamValue::FloatRanged { val, min: _, max: _ } => val.to_string(),
                    ProjectParamValue::IntRanged { val, min: _, max: _ } => val.to_string(),
                    _ => panic!("Unexpected Error"),
                },
                onchange: move |event| {
                    let mut new_params = props.project_params.read().clone();
                    for param_field in new_params.list.iter_mut() {
                        if param_field.name == props.param.name.clone() {
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
