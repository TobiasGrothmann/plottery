use dioxus::prelude::*;

use crate::{
    components::{
        icon::{Icon, IconName},
        ui::{Button, IconButton, Size, Variant},
    },
    model::layer_view::{LayerKey, LayerRow, LayerRows, Pen, RowKind, RowTarget},
    router::editor::params_editor::widgets::format_number,
};

/// Pens a group row shows, the tooltip has their number.
const MAX_PENS_SHOWN: usize = 3;
const SOLO_CLICK: &str = "⌥-click";

#[component]
pub fn LayerEditor(
    rows: Memo<Option<LayerRows>>,
    on_toggle_visible: EventHandler<RowTarget>,
    on_solo: EventHandler<RowTarget>,
    on_set_expanded: EventHandler<(LayerKey, bool)>,
    on_show_all: EventHandler<()>,
) -> Element {
    let Some(layer_rows) = rows() else {
        return rsx! {
            style { { include_str!("layer_editor.css") } }
            p { class: "hint layers-empty", "Run the project to see its layers." }
        };
    };

    rsx! {
        style { { include_str!("layer_editor.css") } }
        if let Some((target, label)) = layer_rows.solo.clone() {
            div { class: "layers-status",
                span { class: "layers-status-text", title: "{label}", "Only {label}" }
                Button {
                    variant: Variant::Ghost,
                    size: Size::Sm,
                    onclick: move |_| on_solo.call(target.clone()),
                    "Exit solo"
                }
            }
        } else if layer_rows.shapes_visible < layer_rows.shapes {
            div { class: "layers-status",
                span { class: "layers-status-text",
                    "{layer_rows.shapes_visible} of {layer_rows.shapes} shapes shown"
                }
                Button {
                    variant: Variant::Ghost,
                    size: Size::Sm,
                    onclick: move |_| on_show_all.call(()),
                    "Show all"
                }
            }
        }
        if layer_rows.rows.is_empty() {
            p { class: "hint layers-empty", "This drawing is empty." }
        }
        div { class: "LayerEditor", role: "tree",
            for row in layer_rows.rows {
                LayerRowView {
                    key: "{row.id()}",
                    row,
                    on_toggle_visible,
                    on_solo,
                    on_set_expanded,
                }
            }
        }
    }
}

#[component]
fn LayerRowView(
    row: LayerRow,
    on_toggle_visible: EventHandler<RowTarget>,
    on_solo: EventHandler<RowTarget>,
    on_set_expanded: EventHandler<(LayerKey, bool)>,
) -> Element {
    let indent = 6 + row.depth * 16;
    if row.kind == RowKind::More {
        return rsx! {
            div { class: "layer layer-more", style: "padding-left: {indent + 22}px;",
                span { class: "hint", "{row.label}" }
            }
        };
    }

    let expanded = match row.kind {
        RowKind::Group { expanded } => Some(expanded),
        _ => None,
    };
    let muted = row.shapes_visible == 0;
    let row_class = format!(
        "layer{}{}",
        if muted { " muted" } else { "" },
        if expanded.is_some() {
            " expandable"
        } else {
            ""
        },
    );
    let count = match row.kind {
        RowKind::Empty => "empty".to_string(),
        _ if muted || row.shapes_visible == row.shapes => row.shapes.to_string(),
        _ => format!("{}/{}", row.shapes_visible, row.shapes),
    };
    let eye_label = match (row.soloed, row.hidden) {
        (true, _) => format!("Show the other layers again, not only {}", row.label),
        (false, true) => format!("Show {}", row.label),
        (false, false) => format!("Hide {}", row.label),
    };
    let eye_title = if row.soloed {
        eye_label.clone()
    } else {
        format!("{eye_label} ({SOLO_CLICK}: show only this)")
    };

    let toggle_expanded = {
        let key = row.target.key().clone();
        move |event: MouseEvent| {
            event.stop_propagation();
            if let Some(expanded) = expanded {
                on_set_expanded.call((key.clone(), !expanded));
            }
        }
    };
    let toggle_expanded_chevron = toggle_expanded.clone();
    let eye_target = row.target.clone();

    rsx! {
        div {
            class: "{row_class}",
            role: "treeitem",
            "aria-expanded": expanded.map(|expanded| if expanded { "true" } else { "false" }),
            style: "padding-left: {indent}px;",
            title: "{row_title(&row)}",
            onclick: toggle_expanded,
            if let Some(expanded) = expanded {
                button {
                    r#type: "button",
                    class: "layer-chevron",
                    "aria-label": if expanded { "Collapse {row.label}" } else { "Expand {row.label}" },
                    onclick: toggle_expanded_chevron,
                    Icon {
                        name: IconName::ChevronDown,
                        class: if expanded { "" } else { "rot-90" },
                    }
                }
            } else {
                span { class: "layer-chevron" }
            }
            {pen_swatch(&row.pens)}
            span { class: if row.named { "layer-name" } else { "layer-name hint" }, "{row.label}" }
            span { class: "count", "{count}" }
            if row.kind == RowKind::Empty {
                span { class: "layer-eye" }
            } else {
                IconButton {
                    icon: if row.hidden { IconName::EyeOff } else { IconName::Eye },
                    label: eye_label,
                    title: eye_title,
                    size: Size::Sm,
                    active: row.soloed,
                    onclick: move |event: MouseEvent| {
                        event.stop_propagation();
                        if event.modifiers().alt() {
                            on_solo.call(eye_target.clone());
                        } else {
                            on_toggle_visible.call(eye_target.clone());
                        }
                    },
                }
            }
        }
    }
}

/// A ring on a paper disc per pen, as thick as the pen is wide.
fn pen_swatch(pens: &[Pen]) -> Element {
    rsx! {
        span { class: "pen-chips",
            if pens.is_empty() {
                span { class: "pen-chip none" }
            }
            for pen in pens.iter().take(MAX_PENS_SHOWN) {
                span { class: "pen-chip",
                    span {
                        class: "pen-ring",
                        style: "border-color: {pen.color.hex()}; border-width: {ring_width_px(pen)}px;",
                    }
                }
            }
        }
    }
}

/// 4 px per mm of pen width, up to a nearly filled ring.
fn ring_width_px(pen: &Pen) -> f32 {
    (pen.width_cm * 40.0).clamp(1.0, 5.0)
}

fn row_title(row: &LayerRow) -> String {
    let pens = match row.pens.as_slice() {
        [] => None,
        [pen] => Some(format!(
            "{} {}, {} mm pen",
            pen.color.name().name,
            pen.color.hex(),
            format_number(pen.width_cm * 10.0)
        )),
        pens => Some(format!("{} pens", pens.len())),
    };
    let shapes = match row.shapes_visible == row.shapes {
        true => format!("{} shapes", row.shapes),
        false => format!("{} of {} shapes shown", row.shapes_visible, row.shapes),
    };
    let length = (row.shapes > 0).then(|| format!("{} to draw", format_length(row.draw_length)));

    [Some(row.label.clone()), pens, Some(shapes), length]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_length(cm: f32) -> String {
    if cm >= 100.0 {
        format!("{:.1} m", cm / 100.0)
    } else {
        format!("{cm:.1} cm")
    }
}
