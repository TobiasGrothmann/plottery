use std::sync::Arc;

use dioxus::prelude::*;
use plottery_project::{
    project_param_value::ProjectParamValue, project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::{
        bool_field::BoolField, curve_2d_norm_field::Curve2DField, number_field::NumberField,
        slider::Slider,
    },
    project_runner::ProjectRunner,
    running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct ParamsEditorProps {
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

#[component]
pub fn ParamsEditor(props: ParamsEditorProps) -> Element {
    rsx! {
        style { { include_str!("params_editor.css") } }
        div { class: "ParamsEditor",
            for param in props.project_params.read().list.iter().cloned() {
                h2 { "{param.formatted_name()}" }
                match param.value {
                    ProjectParamValue::Float(_) | ProjectParamValue::Int(_) => {
                        rsx! {
                            NumberField {
                                param: param,
                                project_params: props.project_params,
                                project_runner: props.project_runner,
                                running_state: props.running_state,
                                console: props.console,
                                release: props.release,
                            }
                        }
                    }
                    ProjectParamValue::FloatRanged { .. } | ProjectParamValue::IntRanged { .. } => {
                        rsx! {
                            Slider {
                                param: param,
                                project_params: props.project_params,
                                project_runner: props.project_runner,
                                running_state: props.running_state,
                                console: props.console,
                                release: props.release,
                            }
                        }
                    }
                    ProjectParamValue::Bool { .. } => {
                        rsx! {
                            BoolField {
                                param: param,
                                project_params: props.project_params,
                                project_runner: props.project_runner,
                                running_state: props.running_state,
                                console: props.console,
                                release: props.release,
                            }
                        }
                    }
                    ProjectParamValue::Curve2DNorm(_) => {
                        rsx! {
                            Curve2DField {
                                param: param,
                                project_params: props.project_params,
                                project_runner: props.project_runner,
                                running_state: props.running_state,
                                console: props.console,
                                release: props.release,
                            }
                        }
                    }
                }
            }
        }
    }
}
