use std::sync::Arc;

use dioxus::prelude::*;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    editor_console::EditorConsole, project_runner::ProjectRunner, running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct EditorSliderProps {
    param: ProjectParam,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<EditorConsole>,
    release: bool,
}

#[component]
pub fn Slider(mut props: EditorSliderProps) -> Element {
    let slider_step = match props.param.value {
        ProjectParamValue::FloatRanged { val: _, min, max } => {
            ((max - min) / 100_000_f32).to_string()
        }
        ProjectParamValue::IntRanged {
            val: _,
            min: _,
            max: _,
        } => "1".to_string(),
        _ => panic!("Unexpected Error"),
    };

    let mut slider_value = use_signal(|| match props.param.value {
        ProjectParamValue::FloatRanged {
            val,
            min: _,
            max: _,
        } => val,
        ProjectParamValue::IntRanged {
            val,
            min: _,
            max: _,
        } => val as f32,
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
                value: slider_value.to_string(),
                oninput: move |event| {
                    let new_value = event.value().parse::<f32>().unwrap();
                    slider_value.set(new_value);
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
                        Ok(mut runner) => runner.trigger_run_project(props.release, props.running_state, props.console),
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
