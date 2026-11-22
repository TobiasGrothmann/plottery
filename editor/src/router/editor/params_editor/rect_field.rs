use dioxus::prelude::*;
use plottery_lib::{Rect, V2};
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::components::{
    icon::IconName,
    ui::{IconButton, Size},
};
use crate::router::editor::{
    params_editor::{params_editor::LeafProps, widgets::number_input::NumberInput},
    pick_target::{PickPart, PickTarget},
};

/// Moves the corner `part` of `rect` to `point`; corners that cross are swapped by `Rect::new`.
fn with_corner(rect: Rect, part: PickPart, point: V2) -> Rect {
    match part {
        PickPart::BottomLeft => Rect::new(point, rect.tr()),
        _ => Rect::new(rect.bl(), point),
    }
}

/// x and y of one corner, and the button that picks it on the preview.
#[component]
fn CornerRow(leaf: LeafProps, part: PickPart, label: String, rect: Rect) -> Element {
    let corner = match part {
        PickPart::BottomLeft => rect.bl(),
        _ => rect.tr(),
    };
    let handle = leaf.use_handle();
    let edit_corner = move |point: V2| {
        handle.edit(|value| {
            if let ProjectParamValue::Rect(rect) = value {
                *rect = with_corner(*rect, part, point);
            }
        });
    };
    let mut pick_target = use_context::<PickTarget>();
    let picking = pick_target.is_target(&leaf.path, part);
    let pick_path = leaf.path.clone();
    let hover_path = leaf.path.clone();
    let drop_path = leaf.path.clone();
    use_drop(move || pick_target.unhover_path(&drop_path));
    let pickable = !leaf.no_pick;

    rsx! {
        div {
            class: "sub",
            onmouseenter: move |_| {
                if pickable {
                    pick_target.hover(&hover_path, part);
                }
            },
            onmouseleave: move |_| {
                if pickable {
                    pick_target.unhover();
                }
            },
            span { class: "sublabel", "{label}" }
            div { class: "hrow",
                NumberInput {
                    value: corner.x,
                    affix: "x",
                    oncommit: move |x: f32| edit_corner(V2::new(x, corner.y)),
                }
                NumberInput {
                    value: corner.y,
                    affix: "y",
                    oncommit: move |y: f32| edit_corner(V2::new(corner.x, y)),
                }
                if pickable {
                    IconButton {
                        icon: IconName::Crosshair,
                        label: "Pick {label} on preview",
                        title: if picking { "Click the preview to set this corner" } else { "Pick on preview" },
                        size: Size::Sm,
                        active: picking,
                        onclick: move |_| pick_target.toggle(&pick_path, part),
                    }
                }
            }
        }
    }
}

/// Bottom left and top right corner, each settable by clicking the preview.
#[component]
pub fn RectField(props: LeafProps) -> Element {
    let ProjectParamValue::Rect(rect) = props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    rsx! {
        CornerRow {
            leaf: props.clone(),
            part: PickPart::BottomLeft,
            label: "bottom left",
            rect,
        }
        CornerRow {
            leaf: props,
            part: PickPart::TopRight,
            label: "top right",
            rect,
        }
    }
}
