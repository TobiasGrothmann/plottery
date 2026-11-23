use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{can_roll, holds_rollable, is_rollable, own_roll_range, takes_roll_range};
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
            dot_path, get_default_value, get_param_value_by_path, get_param_value_mut_by_path,
            leaf_value, leaf_value_mut, no_pick_at, reset_value,
        },
        percent_field::PercentField,
        range_field::RangeField,
        rect_field::RectField,
        roll_range_field::RollRangeField,
        seed_field::SeedField,
        slider_field::SliderField,
        string_field::StringField,
        system_font_field::SystemFontField,
        v2_field::V2Field,
    },
    variations::variations_state::EditorView,
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
        ProjectParamValue::Angle { .. } | ProjectParamValue::AngleRanged { .. } => {
            leaf!(AngleField)
        }
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
                disabled: selecting(ctx),
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
            // in the variations view the fields below can be ticked, the selector stays read-only
            rsx! {
                div { class: "contents", inert: if selecting(ctx) { "true" },
                    {render_leaf_param(selector, path, ctx)}
                }
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
    let read_only = selecting(ctx);

    rsx! {
        div { class: "pvec",
            for (index, item) in vec_value.items.into_iter().enumerate() {
                {
                    let mut item_path = path.clone();
                    item_path.push(format!("[{index}]"));

                    let vec_path_for_remove = path.clone();
                    let item_key = item_path.join("_");
                    let tick = roll_tick(ctx, &item, &item_path);
                    let item_class = roll_class("vec-item", &tick, &item, ctx);
                    let range_cell = roll_range_cell(&item, &tick);
                    let inert = read_only && !holds_rows(&item);

                    rsx! {
                        div {
                            class: item_class,
                            key: "{item_key}",
                            role: if tick.is_some() { "button" },
                            "aria-pressed": tick.as_ref().map(|(_, ticked)| ticked.to_string()),
                            onclick: roll_click(ctx, tick.clone()),
                            span { class: "mono vec-index", "#{index}" }
                            div { class: "grow pctl", inert: if inert { "true" },
                                {render_value_without_label(param_name.clone(), item, item_path, ctx)}
                            }
                            {range_cell}
                            if !read_only {
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
            }
            if !read_only {
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

/// In the variations view the params are read-only and rollable ones get a tick to roll them.
fn selecting(ctx: EditorCtx) -> bool {
    (ctx.variations.view)() == EditorView::Variations
}

/// While a roll runs, what it rolls can't change.
fn rolling(ctx: EditorCtx) -> bool {
    (ctx.variations.rolling)().is_some()
}

/// The dotted path to roll the param at `path` by, and whether it is ticked. `None` outside the
/// variations view and for params that can't be rolled.
fn roll_tick(ctx: EditorCtx, value: &ProjectParamValue, path: &[String]) -> Option<(String, bool)> {
    if !selecting(ctx) || !can_roll(value) {
        return None;
    }
    let dot = dot_path(&ctx.project_params.peek().list, path)?;
    let ticked = ctx.variations.selected.read().contains(&dot);
    Some((dot, ticked))
}

fn set_ticked(ctx: EditorCtx, dot: String, ticked: bool) {
    if ctx.variations.rolling.peek().is_some() {
        return;
    }
    let mut selected = ctx.variations.selected;
    match ticked {
        true => selected.write().insert(dot),
        false => selected.write().remove(&dot),
    };
}

/// The range inputs of a number with a tick, the right column of the variations view.
fn roll_range_cell(value: &ProjectParamValue, tick: &Option<(String, bool)>) -> Option<Element> {
    let (dot, ticked) = tick.as_ref()?;
    if !takes_roll_range(value) {
        return None;
    }
    let integer = matches!(
        leaf_value(value),
        ProjectParamValue::Int(_) | ProjectParamValue::IntRanged { .. }
    );
    Some(rsx! {
        RollRangeField {
            dot: dot.clone(),
            integer,
            own: own_roll_range(value),
            required: *ticked && !is_rollable(value),
        }
    })
}

/// Values drawn as rows of their own (fields, items, a variant's fields), which can have ticks.
fn holds_rows(value: &ProjectParamValue) -> bool {
    match value {
        ProjectParamValue::Struct(_) | ProjectParamValue::Vec(_) => true,
        ProjectParamValue::DataEnum(data_enum) => !data_enum.selected_fields().is_empty(),
        ProjectParamValue::Optional(optional) => holds_rows(&optional.value),
        _ => false,
    }
}

/// `base` plus the variations view's classes: a toggle with a tick, dimmed with nothing to roll.
fn roll_class(
    base: &str,
    tick: &Option<(String, bool)>,
    value: &ProjectParamValue,
    ctx: EditorCtx,
) -> String {
    if !selecting(ctx) {
        return base.to_string();
    }
    // a number's row has a third column for its range
    let ranged = match takes_roll_range(value) {
        true => " ranged",
        false => "",
    };
    let class = match tick {
        Some((_, true)) => format!("{base} roll-row{ranged} on"),
        Some((_, false)) => format!("{base} roll-row{ranged}"),
        None if holds_rollable(value) => base.to_string(),
        None => format!("{base} roll-locked"),
    };
    match rolling(ctx) {
        true => format!("{class} frozen"),
        false => class,
    }
}

/// In the variations view a click on a row toggles its tick and doesn't reach the rows around it.
fn roll_click(ctx: EditorCtx, tick: Option<(String, bool)>) -> impl FnMut(MouseEvent) {
    let read_only = selecting(ctx);
    move |event: MouseEvent| {
        if !read_only {
            return;
        }
        event.stop_propagation();
        if let Some((dot, ticked)) = &tick {
            set_ticked(ctx, dot.clone(), !ticked);
        }
    }
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
    let tick = roll_tick(ctx, &param.value, path);
    let on_tick = tick.clone().map(|(dot, _)| {
        EventHandler::new(move |ticked: bool| set_ticked(ctx, dot.clone(), ticked))
    });
    Some(rsx! {
        ParamSection {
            key: "{state_key}",
            state_key: state_key.clone(),
            title: param.formatted_name(),
            description: param.description.clone(),
            enabled,
            on_enabled,
            locked: selecting(ctx),
            ticked: tick.map(|(_, ticked)| ticked),
            on_tick,
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
    /// The switch can't be changed.
    #[props(default)]
    locked: bool,
    /// Shows a tick in the header to roll the section's param.
    ticked: Option<bool>,
    on_tick: Option<EventHandler<bool>>,
    children: Element,
) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let ticks_locked = rolling(use_context::<EditorCtx>());
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
                if let (Some(ticked), Some(on_tick)) = (ticked, on_tick) {
                    button {
                        r#type: "button",
                        class: if ticked { "roll-chip on" } else { "roll-chip" },
                        "aria-pressed": if ticked { "true" } else { "false" },
                        disabled: ticks_locked,
                        title: "Roll whether {title} is on",
                        onclick: move |_| on_tick.call(!ticked),
                        "roll"
                    }
                }
                if let (Some(enabled), Some(on_enabled)) = (enabled, on_enabled) {
                    Switch {
                        checked: enabled,
                        disabled: locked,
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
    let read_only = selecting(ctx);
    let onreset = move |_| {
        if !read_only {
            reset_at(&reset_path, false)
        }
    };
    // in the variations view a rollable row is a toggle, rows with nothing to roll are dimmed
    let tick = roll_tick(ctx, &param.value, &path);
    let row_class = roll_class(
        if compact { "sub" } else { "prow" },
        &tick,
        &param.value,
        ctx,
    );
    let range_cell = roll_range_cell(&param.value, &tick);
    let item_count = vec_item_count(&param.value).map(|count| match count {
        1 => "1 item".to_string(),
        n => format!("{n} items"),
    });
    let sub_class = match is_struct(&param.value) {
        true => format!("{row_class} group"),
        false => row_class.clone(),
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
    // rows inside the control can have ticks, so only a single widget is made inert
    let inert = read_only && (variant_rows.is_some() || !holds_rows(&param.value));
    let control = match variant_rows {
        Some(_) => render_leaf_param(ProjectParam::new(&param.name, param.value), path, ctx),
        None => render_value_without_label(param.name, param.value, path, ctx),
    };

    if compact {
        rsx! {
            div {
                class: sub_class,
                key: "{key}",
                role: if tick.is_some() { "button" },
                "aria-pressed": tick.as_ref().map(|(_, ticked)| ticked.to_string()),
                onclick: roll_click(ctx, tick.clone()),
                div {
                    class: "sublabel {label_class}",
                    title: label_title,
                    ondoubleclick: onreset,
                    "{formatted_name}"
                    if let Some(item_count) = item_count {
                        span { class: "count", "{item_count}" }
                    }
                }
                div { class: "pctl", inert: if inert { "true" }, {control} }
                {range_cell}
            }
        }
    } else {
        rsx! {
            div {
                class: row_class,
                key: "{key}",
                role: if tick.is_some() { "button" },
                "aria-pressed": tick.as_ref().map(|(_, ticked)| ticked.to_string()),
                onclick: roll_click(ctx, tick.clone()),
                div {
                    class: "plabel {label_class}",
                    title: label_title,
                    ondoubleclick: onreset,
                    "{formatted_name}"
                    if let Some(item_count) = item_count {
                        span { class: "count", "{item_count}" }
                    }
                }
                div { class: "pctl", inert: if inert { "true" }, {control} }
                {range_cell}
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
