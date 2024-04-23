use std::sync::Arc;

use dioxus::prelude::*;
use plottery_project::{ProjectParamValue, ProjectParamsListWrapper};
use tokio::sync::Mutex;

use crate::router_components::editor_components::{
    editor_number_field::EditorNumberField, editor_slider::EditorSlider,
    running_state::RunningState,
};

use super::project_runner::ProjectRunner;

#[derive(PartialEq, Props, Clone)]
pub struct ParamsEditorProps {
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    release: bool,
}

#[component]
pub fn ParamsEditor(props: ParamsEditorProps) -> Element {
    rsx! {
        style { { include_str!("params_editor.css") } }
        div { class: "ParamsEditor",
            for param in props.project_params.read().list.iter().cloned() {
                div { class: "param",
                    p { "{param.name.clone()}" }

                    match param.value {
                        ProjectParamValue::Float(_) | ProjectParamValue::Int(_) => {
                            rsx! {
                                EditorNumberField {
                                    param: param,
                                    project_params: props.project_params,
                                    project_runner: props.project_runner,
                                    running_state: props.running_state,
                                    release: props.release,
                                }
                            }
                        }
                        ProjectParamValue::FloatRanged { .. } | ProjectParamValue::IntRanged { .. } => {
                            rsx! {
                                EditorSlider {
                                    param: param,
                                    project_params: props.project_params,
                                    project_runner: props.project_runner,
                                    running_state: props.running_state,
                                    release: props.release,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
