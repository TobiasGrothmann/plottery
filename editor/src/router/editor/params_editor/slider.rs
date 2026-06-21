use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages, params_editor::param_tree::get_param_value_mut_by_path,
    project_runner::ProjectRunner, running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct EditorSliderProps {
    param: ProjectParam,
    path: Vec<String>,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

#[component]
pub fn Slider(mut props: EditorSliderProps) -> Element {
    let slider_step = match &props.param.value {
        ProjectParamValue::FloatRanged { val: _, min, max } => {
            ((max - min) / 100_000_f32).to_string()
        }
        ProjectParamValue::IntRanged {
            val: _,
            min: _,
            max: _,
        } => "1".to_string(),
        ProjectParamValue::Optional(optional) => match optional.value.as_ref() {
            ProjectParamValue::FloatRanged { val: _, min, max } => {
                ((max - min) / 100_000_f32).to_string()
            }
            ProjectParamValue::IntRanged {
                val: _,
                min: _,
                max: _,
            } => "1".to_string(),
            _ => panic!("Unexpected Error"),
        },
        _ => panic!("Unexpected Error"),
    };

    let mut slider_value = use_signal(|| match &props.param.value {
        ProjectParamValue::FloatRanged {
            val,
            min: _,
            max: _,
        } => *val,
        ProjectParamValue::IntRanged {
            val,
            min: _,
            max: _,
        } => *val as f32,
        ProjectParamValue::Optional(optional) => match optional.value.as_ref() {
            ProjectParamValue::FloatRanged {
                val,
                min: _,
                max: _,
            } => *val,
            ProjectParamValue::IntRanged {
                val,
                min: _,
                max: _,
            } => *val as f32,
            _ => panic!("Unexpected Error"),
        },
        _ => panic!("Unexpected Error"),
    });
    let slider_value_string = use_memo(move || {
        format!("{:.5}", slider_value.read())
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    });

    rsx! {
        style { { include_str!("slider.css") } }
        div { class: "Slider",
            p { class: "slider_value",
                "{slider_value_string}"
            }
            input {
                class: "slider",
                name: "{props.param.name.clone()}",
                required: true,
                r#type: "range",
                step: slider_step,
                min: match &props.param.value {
                    ProjectParamValue::FloatRanged { val: _, min, max: _ } => min.to_string(),
                    ProjectParamValue::IntRanged { val: _, min, max: _ } => min.to_string(),
                    ProjectParamValue::Optional(optional) => match optional.value.as_ref() {
                        ProjectParamValue::FloatRanged { val: _, min, max: _ } => min.to_string(),
                        ProjectParamValue::IntRanged { val: _, min, max: _ } => min.to_string(),
                        _ => panic!("Unexpected Error"),
                    },
                    _ => panic!("Unexpected Error"),
                },
                max: match &props.param.value {
                    ProjectParamValue::FloatRanged { val: _, min: _, max } => max.to_string(),
                    ProjectParamValue::IntRanged { val: _, min: _, max } => max.to_string(),
                    ProjectParamValue::Optional(optional) => match optional.value.as_ref() {
                        ProjectParamValue::FloatRanged { val: _, min: _, max } => max.to_string(),
                        ProjectParamValue::IntRanged { val: _, min: _, max } => max.to_string(),
                        _ => panic!("Unexpected Error"),
                    },
                    _ => panic!("Unexpected Error"),
                },
                value: slider_value.to_string(),
                oninput: move |event| {
                    let new_value = event.value().parse::<f32>().unwrap();
                    slider_value.set(new_value);
                },
                onchange: move |event| {
                    let mut new_params = props.project_params.read().clone();
                    if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &props.path) {
                        let new_val = event
                            .value()
                            .parse::<f32>()
                            .expect("Failed to parse slider value");
                        match param_value {
                            ProjectParamValue::FloatRanged { val, min, max } => {
                                *val = new_val.clamp(*min, *max)
                            }
                            ProjectParamValue::IntRanged { val, min, max } => {
                                *val = (new_val.round() as i32).clamp(*min, *max)
                            }
                            ProjectParamValue::Optional(optional) => {
                                match optional.value.as_mut() {
                                    ProjectParamValue::FloatRanged { val, min, max } => {
                                        *val = new_val.clamp(*min, *max)
                                    }
                                    ProjectParamValue::IntRanged { val, min, max } => {
                                        *val = (new_val.round() as i32).clamp(*min, *max)
                                    }
                                    _ => panic!("Unexpected parameter value type in slider"),
                                }
                            }
                            _ => panic!("Unexpected parameter value type in slider"),
                        }
                    } else {
                        tracing::error!("Param path not found: {:?}", props.path);
                        return;
                    }
                    props.project_params.set(new_params);
                    match props.project_runner.read().try_lock() {
                        Ok(mut runner) => runner.trigger_run_project(props.release, props.running_state, props.console),
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
