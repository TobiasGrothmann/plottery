use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_optional::ProjectParamOptional,
    project_param_struct::ProjectParamStruct, project_param_value::ProjectParamValue,
    project_param_vec::ProjectParamVec, project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::{
        bool_field::BoolField,
        curve_2d_field::Curve2DField,
        number_field::NumberField,
        param_tree::{get_param_mut_by_path, get_param_value_mut_by_path},
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

fn add_vec_item(
    path: &[String],
    mut project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
) {
    let mut new_params = project_params.read().clone();
    if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, path) {
        match param_value {
            ProjectParamValue::Vec(vec_value) => {
                vec_value.items.push((*vec_value.item_prototype).clone());
            }
            ProjectParamValue::Optional(optional) => {
                if let ProjectParamValue::Vec(vec_value) = optional.value.as_mut() {
                    vec_value.items.push((*vec_value.item_prototype).clone());
                } else {
                    tracing::error!(
                        "Param path is Optional but not Optional<Vec<_>>: {:?}",
                        path
                    );
                    return;
                }
            }
            _ => {
                tracing::error!("Param path is not Vec: {:?}", path);
                return;
            }
        }
    } else {
        tracing::error!("Param path not found: {:?}", path);
        return;
    }

    project_params.set(new_params);
    trigger_run(project_runner, running_state, console, release);
}

fn remove_vec_item(
    path: &[String],
    index: usize,
    mut project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
) {
    let mut new_params = project_params.read().clone();
    if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, path) {
        match param_value {
            ProjectParamValue::Vec(vec_value) => {
                if index < vec_value.items.len() {
                    vec_value.items.remove(index);
                } else {
                    tracing::error!("Vec remove index out of bounds: {:?} / {}", path, index);
                    return;
                }
            }
            ProjectParamValue::Optional(optional) => {
                if let ProjectParamValue::Vec(vec_value) = optional.value.as_mut() {
                    if index < vec_value.items.len() {
                        vec_value.items.remove(index);
                    } else {
                        tracing::error!("Vec remove index out of bounds: {:?} / {}", path, index);
                        return;
                    }
                } else {
                    tracing::error!(
                        "Param path is Optional but not Optional<Vec<_>>: {:?}",
                        path
                    );
                    return;
                }
            }
            _ => {
                tracing::error!("Param path is not Vec: {:?}", path);
                return;
            }
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
        ProjectParamValue::Struct(_)
        | ProjectParamValue::Optional(_)
        | ProjectParamValue::Vec(_) => {
            rsx! {
                p { "nested..." }
            }
        }
    }
}

fn render_struct_value(
    param_struct: ProjectParamStruct,
    path: Vec<String>,
    props: &ParamsEditorProps,
) -> Element {
    rsx! {
        div { class: "ParamStruct",
            for child in param_struct.fields.into_iter() {
                {
                    let mut child_path = path.clone();
                    child_path.push(child.name.clone());
                    render_named_param(child, child_path, props, false)
                }
            }
        }
    }
}

fn render_optional_value(
    param_name: &str,
    optional: ProjectParamOptional,
    path: Vec<String>,
    props: &ParamsEditorProps,
) -> Element {
    if !optional.enabled {
        return rsx! { div {} };
    }

    match *optional.value {
        ProjectParamValue::Struct(param_struct) => render_struct_value(param_struct, path, props),
        ProjectParamValue::Vec(vec_value) => {
            render_vec_value(param_name.to_string(), vec_value, path, props)
        }
        value => {
            let inner_param = ProjectParam::new(param_name, value);
            render_leaf_param(inner_param, path, props)
        }
    }
}

fn render_value_without_label(
    param_name: String,
    value: ProjectParamValue,
    path: Vec<String>,
    props: &ParamsEditorProps,
) -> Element {
    match value {
        ProjectParamValue::Struct(param_struct) => render_struct_value(param_struct, path, props),
        ProjectParamValue::Optional(optional) => {
            render_optional_value(param_name.as_str(), optional, path, props)
        }
        ProjectParamValue::Vec(vec_value) => render_vec_value(param_name, vec_value, path, props),
        value => {
            let param = ProjectParam::new(param_name.as_str(), value);
            render_leaf_param(param, path, props)
        }
    }
}

fn render_vec_value(
    param_name: String,
    vec_value: ProjectParamVec,
    path: Vec<String>,
    props: &ParamsEditorProps,
) -> Element {
    let vec_path_for_add = path.clone();
    let project_params_for_add = props.project_params;
    let project_runner_for_add = props.project_runner;
    let running_state_for_add = props.running_state;
    let console_for_add = props.console;
    let release_for_add = props.release;

    rsx! {
        div { class: "ParamVec",
            for (index, item) in vec_value.items.into_iter().enumerate() {
                {
                    let mut item_path = path.clone();
                    item_path.push(format!("[{index}]"));

                    let vec_path_for_remove = path.clone();
                    let project_params_for_remove = props.project_params;
                    let project_runner_for_remove = props.project_runner;
                    let running_state_for_remove = props.running_state;
                    let console_for_remove = props.console;
                    let release_for_remove = props.release;
                    let item_key = item_path.join("_");

                    rsx! {
                        div { class: "ParamVecItem", key: "{item_key}",
                            div { class: "ParamVecItemContent",
                                {render_value_without_label(param_name.clone(), item, item_path, props)}
                            }
                            button {
                                class: "ParamVecDeleteButton",
                                r#type: "button",
                                title: "Delete item",
                                onclick: move |_| {
                                    remove_vec_item(
                                        &vec_path_for_remove,
                                        index,
                                        project_params_for_remove,
                                        project_runner_for_remove,
                                        running_state_for_remove,
                                        console_for_remove,
                                        release_for_remove,
                                    );
                                },
                                "X"
                            }
                        }
                    }
                }
            }

            div { class: "ParamVecAddRow",
                button {
                    class: "ParamVecAddButton",
                    r#type: "button",
                    onclick: move |_| {
                        add_vec_item(
                            &vec_path_for_add,
                            project_params_for_add,
                            project_runner_for_add,
                            running_state_for_add,
                            console_for_add,
                            release_for_add,
                        );
                    },
                    "+"
                }
            }
        }
    }
}

fn render_named_param(
    param: ProjectParam,
    path: Vec<String>,
    props: &ParamsEditorProps,
    top_level: bool,
) -> Element {
    let heading_key = format!("name_{}", path.join("."));
    let formatted_name = param.formatted_name();
    let param_name = param.name;

    match param.value {
        ProjectParamValue::Struct(param_struct) => {
            if top_level {
                rsx! {
                    h2 { key: "{heading_key}", "{formatted_name}" }
                    {render_struct_value(param_struct.clone(), path.clone(), props)}
                }
            } else {
                rsx! {
                    h3 { key: "{heading_key}", "{formatted_name}" }
                    {render_struct_value(param_struct, path, props)}
                }
            }
        }
        ProjectParamValue::Optional(optional) => {
            let path_for_handler = path.clone();
            let project_params = props.project_params;
            let project_runner = props.project_runner;
            let running_state = props.running_state;
            let console = props.console;
            let release = props.release;

            let optional_for_top = optional.clone();
            let optional_for_top_enabled = optional_for_top.enabled;
            let optional_enabled = optional.enabled;

            if top_level {
                rsx! {
                    h2 { key: "{heading_key}", class: "ParamOptionLabel",
                        "{formatted_name}"
                        input {
                            r#type: "checkbox",
                            checked: optional_for_top_enabled,
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
                    {render_optional_value(param_name.as_str(), optional_for_top, path.clone(), props)}
                }
            } else {
                rsx! {
                    h3 { key: "{heading_key}", class: "ParamOptionLabel",
                        "{formatted_name}"
                        input {
                            r#type: "checkbox",
                            checked: optional_enabled,
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
                    {render_optional_value(param_name.as_str(), optional, path, props)}
                }
            }
        }
        ProjectParamValue::Vec(vec_value) => {
            if top_level {
                rsx! {
                    h2 { key: "{heading_key}", "{formatted_name}" }
                    {render_vec_value(param_name.clone(), vec_value.clone(), path.clone(), props)}
                }
            } else {
                rsx! {
                    h3 { key: "{heading_key}", "{formatted_name}" }
                    {render_vec_value(param_name, vec_value, path, props)}
                }
            }
        }
        value => {
            let leaf_param = ProjectParam::new(param_name.as_str(), value);
            if top_level {
                rsx! {
                    h2 { key: "{heading_key}", "{formatted_name}" }
                    {render_leaf_param(leaf_param.clone(), path.clone(), props)}
                }
            } else {
                rsx! {
                    h3 { key: "{heading_key}", "{formatted_name}" }
                    {render_leaf_param(leaf_param, path, props)}
                }
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
                {
                    let path = vec![param.name.clone()];
                    render_named_param(param, path, &props, true)
                }
            }
        }
    }
}
