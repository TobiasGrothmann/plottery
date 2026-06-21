use std::sync::Arc;

use dioxus::prelude::*;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::{
        bool_field::BoolField, curve_2d_field::Curve2DField, number_field::NumberField,
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

fn render_leaf_param(param: ProjectParam, path: Vec<String>, props: &ParamsEditorProps) -> Element {
    let key = path.join(".");
    match &param.value {
        ProjectParamValue::Float(_) | ProjectParamValue::Int(_) => {
            rsx! {
                NumberField {
                    key: "{key}",
                    param: param,
                    path: path,
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
                    key: "{key}",
                    param: param,
                    path: path,
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
                    key: "{key}",
                    param: param,
                    path: path,
                    project_params: props.project_params,
                    project_runner: props.project_runner,
                    running_state: props.running_state,
                    console: props.console,
                    release: props.release,
                }
            }
        }
        ProjectParamValue::Curve2DNorm(_) | ProjectParamValue::Curve2D(_) => {
            rsx! {
                Curve2DField {
                    key: "{key}",
                    param: param,
                    path: path,
                    project_params: props.project_params,
                    project_runner: props.project_runner,
                    running_state: props.running_state,
                    console: props.console,
                    release: props.release,
                }
            }
        }
        ProjectParamValue::Struct(_) => {
            rsx! {
                p { "nested..." }
            }
        }
    }
}

#[component]
pub fn ParamsEditor(props: ParamsEditorProps) -> Element {
    rsx! {
        style { { include_str!("params_editor.css") } }
        div { class: "ParamsEditor",
            for param in props.project_params.read().list.iter().cloned() {
                match param.value.clone() {
                    ProjectParamValue::Struct(param_struct) => {
                        rsx! {
                            h2 { key: "name_{param.name}", "{param.formatted_name()}" }
                            div { class: "ParamStruct",
                                for child in param_struct.fields {
                                    h3 { key: "name_{param.name}_{child.name}", "{child.formatted_name()}" }
                                    {
                                        match child.value.clone() {
                                            ProjectParamValue::Struct(_) => {
                                                rsx! { p { "nested..." } }
                                            }
                                            _ => {
                                                render_leaf_param(
                                                    child.clone(),
                                                    vec![param.name.clone(), child.name.clone()],
                                                    &props,
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        rsx! {
                            h2 { key: "name_{param.name}", "{param.formatted_name()}" }
                            {render_leaf_param(param.clone(), vec![param.name.clone()], &props)}
                        }
                    }
                }
            }
        }
    }
}
