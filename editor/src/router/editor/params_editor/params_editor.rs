use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_optional::ProjectParamOptional,
    project_param_struct::ProjectParamStruct, project_param_value::ProjectParamValue,
    project_param_vec::ProjectParamVec, project_params_list_wrapper::ProjectParamsListWrapper,
};

use crate::components::{
    icon::{Icon, IconName},
    ui::{Button, IconButton, Size, Switch, Variant},
};
use crate::model::app_state::AppState;
use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::{
        angle_field::AngleField,
        bool_field::BoolField,
        color_field::ColorField,
        curve_2d_field::Curve2DField,
        easing_field::EasingField,
        enum_field::EnumField,
        file_field::FileField,
        font_field::FontField,
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
    project_runner::{trigger_run, ProjectRunner},
    running_state::RunningState,
};

#[derive(PartialEq, Props, Clone)]
pub struct ParamsEditorProps {
    project_dir: PathBuf,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<ProjectRunner>,
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
    pub project_runner: SyncSignal<ProjectRunner>,
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

fn set_optional_enabled(
    path: &[String],
    enabled: bool,
    mut project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<ProjectRunner>,
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
    project_runner: SyncSignal<ProjectRunner>,
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
    project_runner: SyncSignal<ProjectRunner>,
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
        ProjectParamValue::Image(_) | ProjectParamValue::Svg(_) => {
            rsx! {
                FileField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Noise(_) => {
            rsx! {
                NoiseField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Easing(_) => {
            rsx! {
                EasingField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Color(_) => {
            rsx! {
                ColorField { key: "{key}", ..LeafProps::new(param, path, props) }
            }
        }
        ProjectParamValue::Font(_) => {
            rsx! {
                FontField { key: "{key}", ..LeafProps::new(param, path, props) }
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
                render_named_param(child, child_path, props, true)
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
            if enabled {
                div { class: "grow pctl",
                    {render_value_without_label(param_name.to_string(), *optional.value, path, props)}
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

fn is_struct(value: &ProjectParamValue) -> bool {
    match value {
        ProjectParamValue::Struct(_) => true,
        ProjectParamValue::Optional(optional) => is_struct(&optional.value),
        _ => false,
    }
}

fn child_path(parent: &[String], name: &str) -> Vec<String> {
    parent.iter().cloned().chain([name.to_string()]).collect()
}

fn section_state_key(props: &ParamsEditorProps, section_path: &str) -> String {
    format!("{}#{}", props.project_dir.to_string_lossy(), section_path)
}

/// Renders full-width rows, wrapping runs of params that share a `#[section]` in a `ParamSection`.
fn render_param_list(
    params: &[ProjectParam],
    parent: &[String],
    props: &ParamsEditorProps,
) -> Element {
    // Flat list so every sibling is keyed; Dioxus panics on mixed keyed/unkeyed siblings.
    let groups = params
        .chunk_by(|a, b| a.section == b.section)
        .flat_map(|group| {
            let rows = group
                .iter()
                .map(|param| render_named_param(param.clone(), child_path(parent, &param.name), props, false))
                .collect::<Vec<_>>();
            match &group[0].section {
                None => rows,
                Some(title) => {
                    let section_path = child_path(parent, &format!("§{title}")).join(".");
                    let state_key = section_state_key(props, &section_path);
                    vec![rsx! {
                        ParamSection { key: "{state_key}", state_key: state_key.clone(), title: title.clone(),
                            {rows.into_iter()}
                        }
                    }]
                }
            }
        })
        .collect::<Vec<_>>();
    rsx! { {groups.into_iter()} }
}

/// A `Struct` or `Option<Struct>` param as a collapsible section; `None` for any other value.
fn render_struct_section(
    param: &ProjectParam,
    path: &[String],
    props: &ParamsEditorProps,
) -> Option<Element> {
    let (fields, enabled) = match &param.value {
        ProjectParamValue::Struct(param_struct) => (&param_struct.fields, None),
        ProjectParamValue::Optional(optional) => match optional.value.as_ref() {
            ProjectParamValue::Struct(param_struct) => {
                (&param_struct.fields, Some(optional.enabled))
            }
            _ => return None,
        },
        _ => return None,
    };
    let state_key = section_state_key(props, &path.join("."));
    let path_for_switch = path.to_vec();
    let (project_params, project_runner, running_state, console, release) = (
        props.project_params,
        props.project_runner,
        props.running_state,
        props.console,
        props.release,
    );
    let on_enabled = enabled.map(|_| {
        EventHandler::new(move |enabled: bool| {
            set_optional_enabled(
                &path_for_switch,
                enabled,
                project_params,
                project_runner,
                running_state,
                console,
                release,
            )
        })
    });
    Some(rsx! {
        ParamSection {
            key: "{state_key}",
            state_key: state_key.clone(),
            title: param.formatted_name(),
            description: param.description.clone(),
            enabled,
            on_enabled,
            {render_param_list(fields, path, props)}
        }
    })
}

#[component]
fn ParamSection(
    state_key: String,
    title: String,
    description: Option<String>,
    /// Shows a switch in the header; the body is hidden while it is off.
    enabled: Option<bool>,
    on_enabled: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let mut collapsed = use_signal(|| {
        AppState::load_or_default()
            .collapsed_param_sections
            .contains(&state_key)
    });
    let switched_off = enabled == Some(false);
    let open = !collapsed() && !switched_off;
    let toggle_key = state_key.clone();

    rsx! {
        section { class: "psection",
            div { class: "psection-head",
                button {
                    r#type: "button",
                    class: "psection-toggle",
                    title: description,
                    disabled: switched_off,
                    "aria-expanded": if open { "true" } else { "false" },
                    onclick: move |_| {
                        collapsed.set(!collapsed());
                        let mut state = AppState::load_or_default();
                        state.toggle_param_section(toggle_key.clone());
                        state.save();
                    },
                    Icon {
                        name: IconName::ChevronDown,
                        class: if open { "sec-chevron" } else { "sec-chevron rot-90" },
                    }
                    span { class: "psection-title", "{title}" }
                }
                if let (Some(enabled), Some(on_enabled)) = (enabled, on_enabled) {
                    Switch {
                        checked: enabled,
                        aria_label: "Enable {title}",
                        onchange: move |enabled: bool| on_enabled.call(enabled),
                    }
                }
            }
            if open {
                div { class: "psection-body", {children} }
            }
        }
    }
}

/// `compact` renders a narrow label column for use inside another control (e.g. a vec item).
fn render_named_param(
    param: ProjectParam,
    path: Vec<String>,
    props: &ParamsEditorProps,
    compact: bool,
) -> Element {
    if !compact {
        if let Some(section) = render_struct_section(&param, &path, props) {
            return section;
        }
    }
    let key = format!("param_{}", path.join("."));
    let formatted_name = param.formatted_name();
    let description = param.description.clone();
    let label_class = if description.is_some() {
        "has-desc"
    } else {
        ""
    };
    let item_count = vec_item_count(&param.value).map(|count| match count {
        1 => "1 item".to_string(),
        n => format!("{n} items"),
    });
    let sub_class = if is_struct(&param.value) {
        "sub group"
    } else {
        "sub"
    };
    let control = render_value_without_label(param.name, param.value, path, props);

    if compact {
        rsx! {
            div { class: sub_class, key: "{key}",
                div { class: "sublabel {label_class}", title: description,
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
            div { class: "prow", key: "{key}",
                div { class: "plabel {label_class}", title: description,
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
    let params = props.project_params.read().list.clone();
    rsx! {
        style { {include_str!("params_editor.css")} }
        div { class: "ParamsEditor", {render_param_list(&params, &[], &props)} }
    }
}
