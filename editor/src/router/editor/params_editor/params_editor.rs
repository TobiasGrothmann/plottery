use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_optional::ProjectParamOptional,
    project_param_value::ProjectParamValue, project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::{
        bool_field::BoolField, curve_2d_field::Curve2DField, number_field::NumberField,
        param_tree::get_param_mut_by_path, slider::Slider,
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

fn trigger_run(
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    mut running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
) {
    match project_runner.read().try_lock() {
        Ok(mut runner) => {
            runner.trigger_run_project(release, running_state, console);
        }
        Err(e) => {
            tracing::error!("Error preparing to run: {:?}", e);
            running_state.set(RunningState::RunFailed {
                msg: format!("Error preparing to run: {}", e),
            });
        }
    }
}

fn set_optional_enabled(
    path: &[String],
    enabled: bool,
    mut project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
) {
    let mut new_params = project_params.read().clone();
    if let Some(param_field) = get_param_mut_by_path(&mut new_params.list, path) {
        if let ProjectParamValue::Optional(optional) = &mut param_field.value {
            optional.enabled = enabled;
        } else {
            tracing::error!("Param path is not Optional: {:?}", path);
            return;
        }
    } else {
        tracing::error!("Param path not found: {:?}", path);
        return;
    }

    project_params.set(new_params);
    trigger_run(project_runner, running_state, console, release);
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
        ProjectParamValue::Bool(_) => {
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
        ProjectParamValue::Struct(_) | ProjectParamValue::Optional(_) => {
            rsx! {
                p { "nested..." }
            }
        }
    }
}

fn render_optional_value(
    param_name: &str,
    optional: &ProjectParamOptional,
    path: Vec<String>,
    props: &ParamsEditorProps,
) -> Element {
    if !optional.enabled {
        return rsx! { div {} };
    }

    match optional.value.as_ref() {
        ProjectParamValue::Struct(param_struct) => {
            rsx! {
                div { class: "ParamStruct",
                    for child in param_struct.fields.iter().cloned() {
                        {render_nested_child_row(param_name.to_string(), child, props)}
                    }
                }
            }
        }
        _ => {
            let inner_param = ProjectParam::new(param_name, (*optional.value).clone());
            render_leaf_param(inner_param, path, props)
        }
    }
}

fn render_nested_child_row(
    parent_name: String,
    child: ProjectParam,
    props: &ParamsEditorProps,
) -> Element {
    let path = vec![parent_name.clone(), child.name.clone()];

    match child.value.clone() {
        ProjectParamValue::Struct(_) => {
            rsx! {
                h3 { key: "name_{parent_name}_{child.name}", "{child.formatted_name()}" }
                p { "nested..." }
            }
        }
        ProjectParamValue::Optional(optional) => {
            let label_path = path.clone();
            let label_path_for_handler = label_path.clone();
            let project_params = props.project_params;
            let project_runner = props.project_runner;
            let running_state = props.running_state;
            let console = props.console;
            let release = props.release;
            rsx! {
                h3 { key: "name_{parent_name}_{child.name}", class: "ParamOptionLabel",
                    "{child.formatted_name()}"
                    input {
                        r#type: "checkbox",
                        checked: optional.enabled,
                        onchange: move |event| {
                            let enabled = event.value().parse().unwrap_or(false);
                            set_optional_enabled(
                                &label_path_for_handler,
                                enabled,
                                project_params,
                                project_runner,
                                running_state,
                                console,
                                release,
                            );
                        }
                    }
                }
                {
                    if optional.enabled {
                        match optional.value.as_ref() {
                            ProjectParamValue::Struct(_) => rsx! { p { "nested..." } },
                            _ => {
                                let inner_param = ProjectParam::new(child.name.as_str(), (*optional.value).clone());
                                render_leaf_param(inner_param, label_path, props)
                            }
                        }
                    } else {
                        rsx! { div {} }
                    }
                }
            }
        }
        _ => {
            rsx! {
                h3 { key: "name_{parent_name}_{child.name}", "{child.formatted_name()}" }
                {render_leaf_param(child, path, props)}
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
                                    {render_nested_child_row(param.name.clone(), child, &props)}
                                }
                            }
                        }
                    }
                    ProjectParamValue::Optional(optional) => {
                        let path = vec![param.name.clone()];
                        let path_for_handler = path.clone();
                        let project_params = props.project_params;
                        let project_runner = props.project_runner;
                        let running_state = props.running_state;
                        let console = props.console;
                        let release = props.release;
                        rsx! {
                            h2 { key: "name_{param.name}", class: "ParamOptionLabel",
                                "{param.formatted_name()}"
                                input {
                                    r#type: "checkbox",
                                    checked: optional.enabled,
                                    onchange: move |event| {
                                        let enabled = event.value().parse().unwrap_or(false);
                                        set_optional_enabled(
                                            &path_for_handler,
                                            enabled,
                                            project_params,
                                            project_runner,
                                            running_state,
                                            console,
                                            release,
                                        );
                                    }
                                }
                            }
                            {
                                render_optional_value(
                                    param.name.as_str(),
                                    &optional,
                                    path,
                                    &props,
                                )
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
