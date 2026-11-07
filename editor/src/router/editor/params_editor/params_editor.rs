use std::{path::PathBuf, sync::Arc};

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_optional::ProjectParamOptional,
    project_param_struct::ProjectParamStruct, project_param_value::ProjectParamValue,
    project_param_vec::ProjectParamVec, project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::components::{
    icon::IconName,
    ui::{Button, IconButton, Size, Switch, Variant},
};
use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::{
        angle_field::AngleField,
        bool_field::BoolField,
        color_field::ColorField,
        curve_2d_field::Curve2DField,
        enum_field::EnumField,
        image_field::ImageField,
        noise_field::NoiseField,
        number_field::NumberField,
        paper_field::PaperField,
        param_tree::{get_param_mut_by_path, get_param_value_mut_by_path, leaf_value_mut},
        range_field::RangeField,
        seed_field::SeedField,
        slider::Slider,
        string_field::StringField,
        v2_field::V2Field,
    },
    project_runner::ProjectRunner,
    running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct ParamsEditorProps {
    project_dir: PathBuf,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

/// Props shared by all leaf widgets. `param` already has `Optional` unwrapped, `path` has not.
#[derive(Props, Clone)]
pub struct LeafProps {
    pub param: ProjectParam,
    pub path: Vec<String>,
    pub project_dir: PathBuf,
    pub project_params: SyncSignal<ProjectParamsListWrapper>,
    pub project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    pub running_state: SyncSignal<RunningState>,
    pub console: SyncSignal<ConsoleMessages>,
    pub release: bool,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for LeafProps {
    fn eq(&self, other: &Self) -> bool {
        self.param.name == other.param.name
            && self.param.value == other.param.value
            && self.path == other.path
            && self.project_dir == other.project_dir
            && self.project_params == other.project_params
            && self.project_runner == other.project_runner
            && self.running_state == other.running_state
            && self.console == other.console
            && self.release == other.release
    }
}

impl LeafProps {
    fn new(param: ProjectParam, path: Vec<String>, props: &ParamsEditorProps) -> Self {
        Self {
            param,
            path,
            project_dir: props.project_dir.clone(),
            project_params: props.project_params,
            project_runner: props.project_runner,
            running_state: props.running_state,
            console: props.console,
            release: props.release,
        }
    }

    /// Applies `edit` to the leaf value at `path` and reruns the project.
    pub fn edit(&self, edit: impl FnOnce(&mut ProjectParamValue)) {
        let mut project_params = self.project_params;
        let mut new_params = project_params.read().clone();
        match get_param_value_mut_by_path(&mut new_params.list, &self.path) {
            Some(value) => edit(leaf_value_mut(value)),
            None => {
                tracing::error!("Param path not found: {:?}", self.path);
                return;
            }
        }
        project_params.set(new_params);
        trigger_run(
            self.project_runner,
            self.running_state,
            self.console,
            self.release,
        );
    }
}

pub(crate) fn trigger_run(
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
                    param,
                    path,
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
                Slider { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Bool(_) => {
            rsx! {
                BoolField {
                    key: "{key}",
                    param,
                    path,
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
                    param,
                    path,
                    project_params: props.project_params,
                    project_runner: props.project_runner,
                    running_state: props.running_state,
                    console: props.console,
                    release: props.release,
                }
            }
        }
        ProjectParamValue::Enum(_) => {
            rsx! {
                EnumField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::String(_) => {
            rsx! {
                StringField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Paper(_) => {
            rsx! {
                PaperField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::RangeF32 { .. } | ProjectParamValue::RangeI32 { .. } => {
            rsx! {
                RangeField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Angle { .. } => {
            rsx! {
                AngleField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::V2(_) | ProjectParamValue::V2i(_) => {
            rsx! {
                V2Field { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Seed(_) => {
            rsx! {
                SeedField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Image(_) => {
            rsx! {
                ImageField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Noise(_) => {
            rsx! {
                NoiseField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Color(_) => {
            rsx! {
                ColorField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Struct(_)
        | ProjectParamValue::Optional(_)
        | ProjectParamValue::Vec(_) => {
            rsx! {
                span { class: "hint", "Unsupported nested value" }
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
        for child in param_struct.fields.into_iter() {
            {
                let mut child_path = path.clone();
                child_path.push(child.name.clone());
                render_named_param(child, child_path, props, false)
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
    let enabled = optional.enabled;
    let path_for_switch = path.clone();
    let project_params = props.project_params;
    let project_runner = props.project_runner;
    let running_state = props.running_state;
    let console = props.console;
    let release = props.release;

    rsx! {
        div { class: "hrow opt-row",
            Switch {
                checked: enabled,
                aria_label: "Set {param_name}",
                onchange: move |enabled: bool| {
                    set_optional_enabled(
                        &path_for_switch,
                        enabled,
                        project_params,
                        project_runner,
                        running_state,
                        console,
                        release,
                    );
                },
            }
            div { class: "grow pctl",
                if enabled {
                    {render_value_without_label(param_name.to_string(), *optional.value, path, props)}
                } else {
                    span { class: "hint opt-unset", "Not set" }
                }
            }
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
        div { class: "pvec",
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
                        div { class: "vec-item", key: "{item_key}",
                            span { class: "mono vec-index", "{index}" }
                            div { class: "grow pctl",
                                {render_value_without_label(param_name.clone(), item, item_path, props)}
                            }
                            IconButton {
                                icon: IconName::X,
                                label: "Remove item",
                                size: Size::Sm,
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
                            }
                        }
                    }
                }
            }
            div {
                Button {
                    variant: Variant::Dashed,
                    size: Size::Sm,
                    icon: IconName::Plus,
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
                    "Add item"
                }
            }
        }
    }
}

fn vec_item_count(value: &ProjectParamValue) -> Option<usize> {
    match value {
        ProjectParamValue::Vec(vec_value) => Some(vec_value.items.len()),
        ProjectParamValue::Optional(optional) => vec_item_count(&optional.value),
        _ => None,
    }
}

fn render_named_param(
    param: ProjectParam,
    path: Vec<String>,
    props: &ParamsEditorProps,
    top_level: bool,
) -> Element {
    let key = format!("param_{}", path.join("."));
    let formatted_name = param.formatted_name();
    let item_count = vec_item_count(&param.value).map(|count| match count {
        1 => "1 item".to_string(),
        n => format!("{n} items"),
    });
    let control = render_value_without_label(param.name, param.value, path, props);

    if top_level {
        rsx! {
            div { class: "prow", key: "{key}",
                div { class: "plabel",
                    "{formatted_name}"
                    if let Some(item_count) = item_count {
                        span { class: "count", "{item_count}" }
                    }
                }
                div { class: "pctl", {control} }
            }
        }
    } else {
        rsx! {
            div { class: "sub", key: "{key}",
                div { class: "sublabel",
                    "{formatted_name}"
                    if let Some(item_count) = item_count {
                        span { class: "count", "{item_count}" }
                    }
                }
                div { class: "pctl", {control} }
            }
        }
    }
}

#[component]
pub fn ParamsEditor(props: ParamsEditorProps) -> Element {
    rsx! {
        style { {include_str!("params_editor.css")} }
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
