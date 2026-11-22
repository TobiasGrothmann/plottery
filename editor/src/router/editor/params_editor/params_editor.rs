use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_runtime::{
    project_param::ProjectParam, project_param_optional::ProjectParamOptional,
    project_param_struct::ProjectParamStruct, project_param_value::ProjectParamValue,
    project_param_vec::ProjectParamVec,
};

use crate::components::{
    icon::{Icon, IconName},
    ui::{Button, IconButton, Size, Switch, Variant},
};
use crate::model::app_state::{update_app_state, AppState};
use crate::router::editor::{
    editor_ctx::EditorCtx,
    params_editor::{
        angle_field::AngleField,
        bool_field::BoolField,
        color_field::ColorField,
        curve_2d_field::Curve2DField,
        easing_field::EasingField,
        enum_field::EnumField,
        file_field::FileField,
        font_field::FontField,
        number_field::NumberField,
        paper_field::PaperField,
        param_tree::{
            get_default_value, get_param_value_by_path, get_param_value_mut_by_path, leaf_value,
            leaf_value_mut, no_pick_at, reset_value,
        },
        percent_field::PercentField,
        range_field::RangeField,
        rect_field::RectField,
        seed_field::SeedField,
        slider_field::SliderField,
        string_field::StringField,
        system_font_field::SystemFontField,
        v2_field::V2Field,
    },
};

/// Props shared by all leaf widgets. `param` already has `Optional` unwrapped, `path` has not.
#[derive(Props, Clone, PartialEq)]
pub struct LeafProps {
    pub param: ProjectParam,
    pub path: Vec<String>,
    /// From `#[no_pick]` in the project: no picking on the preview.
    pub no_pick: bool,
}

impl LeafProps {
    fn new(param: ProjectParam, path: Vec<String>, ctx: EditorCtx) -> Self {
        let no_pick = no_pick_at(&ctx.project_params.read().defaults, &path);
        Self {
            param,
            path,
            no_pick,
        }
    }

    /// Hook: the handle the widget's handlers edit the value with. Call it before any early return.
    pub fn use_handle(&self) -> LeafHandle {
        let mut path = use_hook(|| CopyValue::new(self.path.clone()));
        if *path.peek() != self.path {
            path.set(self.path.clone());
        }
        LeafHandle { path }
    }
}

/// Copyable access to a leaf value, so every handler of a widget can edit it.
#[derive(Clone, Copy, PartialEq)]
pub struct LeafHandle {
    path: CopyValue<Vec<String>>,
}

impl LeafHandle {
    /// Applies `edit` to the leaf value and reruns the project.
    pub fn edit(&self, edit: impl FnOnce(&mut ProjectParamValue)) {
        edit_leaf_at(&self.path.read(), edit, true);
    }

    /// Like [`LeafHandle::edit`] without the rerun, for drags that rerun on release.
    pub fn edit_without_run(&self, edit: impl FnOnce(&mut ProjectParamValue)) {
        edit_leaf_at(&self.path.read(), edit, false);
    }

    /// Sets the leaf value back to the project's default, for a double-click on a drag widget.
    pub fn reset(&self) {
        reset_at(&self.path.read(), true);
    }
}

/// Applies `edit` to the value at `path` and reruns the project if `rerun`.
fn edit_value_at(path: &[String], edit: impl FnOnce(&mut ProjectParamValue), rerun: bool) {
    let mut ctx = consume_context::<EditorCtx>();
    let mut new_params = ctx.project_params.read().clone();
    let Some(value) = get_param_value_mut_by_path(&mut new_params.list, path) else {
        tracing::error!("Param path not found: {:?}", path);
        return;
    };
    edit(value);
    ctx.project_params.set(new_params);
    if rerun {
        ctx.trigger_run();
    }
}

/// Like [`edit_value_at`] for the leaf value, `Optional` unwrapped. Drags pass `rerun: false`
/// while moving and rerun on release.
pub fn edit_leaf_at(path: &[String], edit: impl FnOnce(&mut ProjectParamValue), rerun: bool) {
    edit_value_at(path, |value| edit(leaf_value_mut(value)), rerun);
}

/// Sets the param at `path` back to the project's default and reruns, unless it is there
/// already. With `leaf_only` an `Option` keeps its switch. Does nothing until the first run has
/// delivered the defaults.
pub fn reset_at(path: &[String], leaf_only: bool) {
    let ctx = consume_context::<EditorCtx>();
    let (current, default) = {
        let params = ctx.project_params.read();
        (
            get_param_value_by_path(&params.list, path),
            get_default_value(&params.defaults, path),
        )
    };
    let (Some(current), Some(default)) = (current, default) else {
        return;
    };
    let mut reset = current.clone();
    match leaf_only {
        true => reset_value(leaf_value_mut(&mut reset), leaf_value(&default)),
        false => reset_value(&mut reset, &default),
    }
    if reset != current {
        edit_value_at(path, |value| *value = reset, true);
    }
}

fn set_optional_enabled(path: &[String], enabled: bool) {
    let edit = |value: &mut ProjectParamValue| match value {
        ProjectParamValue::Optional(optional) => optional.enabled = enabled,
        other => tracing::error!("{path:?} is a {}, not an Optional", other.type_name()),
    };
    edit_value_at(path, edit, true);
}

/// Applies `edit` to the `Vec` or `Option<Vec>` at `path`.
fn edit_vec_at(path: &[String], edit: impl FnOnce(&mut ProjectParamVec)) {
    let edit = |value: &mut ProjectParamValue| match value {
        ProjectParamValue::Vec(vec_value) => edit(vec_value),
        other => tracing::error!("{path:?} is a {}, not a Vec", other.type_name()),
    };
    edit_leaf_at(path, edit, true);
}

fn add_vec_item(path: &[String]) {
    edit_vec_at(path, |vec_value| {
        vec_value.items.push((*vec_value.item_prototype).clone())
    });
}

fn remove_vec_item(path: &[String], index: usize) {
    edit_vec_at(path, |vec_value| {
        if index < vec_value.items.len() {
            vec_value.items.remove(index);
        }
    });
}

fn render_leaf_param(param: ProjectParam, path: Vec<String>, ctx: EditorCtx) -> Element {
    let key = path.join(".");
    macro_rules! leaf {
        ($Field:ident) => {
            rsx! {
                $Field { key: "{key}", ..LeafProps::new(param, path, ctx) }
            }
        };
    }
    match &param.value {
        ProjectParamValue::Float(_) | ProjectParamValue::Int(_) => leaf!(NumberField),
        ProjectParamValue::FloatRanged { .. } | ProjectParamValue::IntRanged { .. } => {
            leaf!(SliderField)
        }
        ProjectParamValue::Bool(_) => leaf!(BoolField),
        ProjectParamValue::Curve2DNorm(_) | ProjectParamValue::Curve2D(_) => leaf!(Curve2DField),
        ProjectParamValue::Enum(_) | ProjectParamValue::DataEnum(_) => leaf!(EnumField),
        ProjectParamValue::String(_) => leaf!(StringField),
        ProjectParamValue::Paper(_) => leaf!(PaperField),
        ProjectParamValue::RangeF32 { .. } | ProjectParamValue::RangeI32 { .. } => {
            leaf!(RangeField)
        }
        ProjectParamValue::Angle { .. } => leaf!(AngleField),
        ProjectParamValue::Percent { .. } => leaf!(PercentField),
        ProjectParamValue::V2(_) | ProjectParamValue::V2i(_) => leaf!(V2Field),
        ProjectParamValue::Rect(_) => leaf!(RectField),
        ProjectParamValue::Seed(_) => leaf!(SeedField),
        ProjectParamValue::Image(_) | ProjectParamValue::Svg(_) => leaf!(FileField),
        ProjectParamValue::Easing(_) => leaf!(EasingField),
        ProjectParamValue::Color(_) => leaf!(ColorField),
        ProjectParamValue::Font(_) => leaf!(FontField),
        ProjectParamValue::SystemFont(_) => leaf!(SystemFontField),
        ProjectParamValue::Struct(_)
        | ProjectParamValue::Optional(_)
        | ProjectParamValue::Vec(_) => {
            rsx! {
                span { class: "hint", "Unsupported nested value" }
            }
        }
        // Only in saved files, merging with the project's params turns it into a struct.
        ProjectParamValue::Noise(_) => {
            rsx! {
                span { class: "hint", "Noise from an older Plottery, run the project to update it" }
            }
        }
    }
}

fn render_struct_value(
    param_struct: ProjectParamStruct,
    path: Vec<String>,
    ctx: EditorCtx,
) -> Element {
    rsx! {
        for child in param_struct.fields.into_iter() {
            {
                let mut child_path = path.clone();
                child_path.push(child.name.clone());
                render_named_param(child, child_path, ctx, true)
            }
        }
    }
}

fn render_optional_value(
    param_name: &str,
    optional: ProjectParamOptional,
    path: Vec<String>,
    ctx: EditorCtx,
) -> Element {
    let enabled = optional.enabled;
    let path_for_switch = path.clone();

    rsx! {
        div { class: "hrow opt-row",
            Switch {
                checked: enabled,
                aria_label: "Set {param_name}",
                onchange: move |enabled: bool| set_optional_enabled(&path_for_switch, enabled),
            }
            if enabled {
                div { class: "grow pctl",
                    {render_value_without_label(param_name.to_string(), *optional.value, path, ctx)}
                }
            }
        }
    }
}

fn render_value_without_label(
    param_name: String,
    value: ProjectParamValue,
    path: Vec<String>,
    ctx: EditorCtx,
) -> Element {
    match value {
        ProjectParamValue::Struct(param_struct) => render_struct_value(param_struct, path, ctx),
        ProjectParamValue::Optional(optional) => {
            render_optional_value(param_name.as_str(), optional, path, ctx)
        }
        ProjectParamValue::Vec(vec_value) => render_vec_value(param_name, vec_value, path, ctx),
        ProjectParamValue::DataEnum(data_enum) => {
            let fields = data_enum.selected_fields().to_vec();
            let variant_path = child_path(&path, &data_enum.selected);
            let selector =
                ProjectParam::new(param_name.as_str(), ProjectParamValue::DataEnum(data_enum));
            rsx! {
                {render_leaf_param(selector, path, ctx)}
                if !fields.is_empty() {
                    div { class: "penum-fields",
                        for field in fields.into_iter() {
                            {
                                let field_path = child_path(&variant_path, &field.name);
                                render_named_param(field, field_path, ctx, true)
                            }
                        }
                    }
                }
            }
        }
        value => {
            let param = ProjectParam::new(param_name.as_str(), value);
            render_leaf_param(param, path, ctx)
        }
    }
}

fn render_vec_value(
    param_name: String,
    vec_value: ProjectParamVec,
    path: Vec<String>,
    ctx: EditorCtx,
) -> Element {
    let vec_path_for_add = path.clone();

    rsx! {
        div { class: "pvec",
            for (index, item) in vec_value.items.into_iter().enumerate() {
                {
                    let mut item_path = path.clone();
                    item_path.push(format!("[{index}]"));

                    let vec_path_for_remove = path.clone();
                    let item_key = item_path.join("_");

                    rsx! {
                        div { class: "vec-item", key: "{item_key}",
                            span { class: "mono vec-index", "#{index}" }
                            div { class: "grow pctl",
                                {render_value_without_label(param_name.clone(), item, item_path, ctx)}
                            }
                            IconButton {
                                icon: IconName::X,
                                label: "Remove item",
                                size: Size::Sm,
                                onclick: move |_| remove_vec_item(&vec_path_for_remove, index),
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
                    onclick: move |_| add_vec_item(&vec_path_for_add),
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

fn section_state_key(ctx: EditorCtx, section_path: &str) -> String {
    format!(
        "{}#{}",
        ctx.project.read().dir.to_string_lossy(),
        section_path
    )
}

/// Renders full-width rows, wrapping runs of params that share a `#[section]` in a `ParamSection`.
fn render_param_list(params: &[ProjectParam], parent: &[String], ctx: EditorCtx) -> Element {
    // Flat list so every sibling is keyed; Dioxus panics on mixed keyed/unkeyed siblings.
    let groups = params
        .chunk_by(|a, b| a.section == b.section)
        .flat_map(|group| {
            let rows = group
                .iter()
                .map(|param| render_named_param(param.clone(), child_path(parent, &param.name), ctx, false))
                .collect::<Vec<_>>();
            match &group[0].section {
                None => rows,
                Some(title) => {
                    let section_path = child_path(parent, &format!("§{title}")).join(".");
                    let state_key = section_state_key(ctx, &section_path);
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
fn render_struct_section(param: &ProjectParam, path: &[String], ctx: EditorCtx) -> Option<Element> {
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
    let state_key = section_state_key(ctx, &path.join("."));
    let path_for_switch = path.to_vec();
    let on_enabled = enabled.map(|_| {
        EventHandler::new(move |enabled: bool| set_optional_enabled(&path_for_switch, enabled))
    });
    Some(rsx! {
        ParamSection {
            key: "{state_key}",
            state_key: state_key.clone(),
            title: param.formatted_name(),
            description: param.description.clone(),
            enabled,
            on_enabled,
            {render_param_list(fields, path, ctx)}
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
    let app_state = use_context::<Signal<AppState>>();
    let mut collapsed = use_signal(|| {
        app_state
            .peek()
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
                        update_app_state(app_state, |state| state.toggle_param_section(toggle_key.clone()));
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
    ctx: EditorCtx,
    compact: bool,
) -> Element {
    if !compact {
        if let Some(section) = render_struct_section(&param, &path, ctx) {
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
    let label_title = match &description {
        Some(description) => format!("{description}\nDouble-click to reset"),
        None => "Double-click to reset".to_string(),
    };
    let reset_path = path.clone();
    let onreset = move |_| reset_at(&reset_path, false);
    let item_count = vec_item_count(&param.value).map(|count| match count {
        1 => "1 item".to_string(),
        n => format!("{n} items"),
    });
    let sub_class = if is_struct(&param.value) {
        "sub group"
    } else {
        "sub"
    };
    // In a full-width row the selected variant's fields get full-width rows below the selector.
    let variant_rows = match &param.value {
        ProjectParamValue::DataEnum(data_enum)
            if !compact && !data_enum.selected_fields().is_empty() =>
        {
            let variant_path = child_path(&path, &data_enum.selected);
            Some(render_param_list(
                data_enum.selected_fields(),
                &variant_path,
                ctx,
            ))
        }
        _ => None,
    };
    let control = match variant_rows {
        Some(_) => render_leaf_param(ProjectParam::new(&param.name, param.value), path, ctx),
        None => render_value_without_label(param.name, param.value, path, ctx),
    };

    if compact {
        rsx! {
            div { class: sub_class, key: "{key}",
                div {
                    class: "sublabel {label_class}",
                    title: label_title,
                    ondoubleclick: onreset,
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
                div {
                    class: "plabel {label_class}",
                    title: label_title,
                    ondoubleclick: onreset,
                    "{formatted_name}"
                    if let Some(item_count) = item_count {
                        span { class: "count", "{item_count}" }
                    }
                }
                div { class: "pctl", {control} }
                if let Some(variant_rows) = variant_rows {
                    div { class: "psection-body penum-body", {variant_rows} }
                }
            }
        }
    }
}

#[component]
pub fn ParamsEditor() -> Element {
    let ctx = use_context::<EditorCtx>();
    let params = ctx.project_params.read().list.clone();
    rsx! {
        style { {include_str!("params_editor.css")} }
        div { class: "ParamsEditor", {render_param_list(&params, &[], ctx)} }
    }
}
