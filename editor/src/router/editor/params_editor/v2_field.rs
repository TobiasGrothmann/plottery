use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::components::{
    icon::IconName,
    ui::{IconButton, Size},
};
use crate::router::editor::{
    params_editor::{params_editor::LeafProps, widgets::number_input::NumberInput},
    pick_target::{PickPart, PickTarget},
};

#[component]
pub fn V2Field(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let (x, y, integer) = match props.param.value {
        ProjectParamValue::V2(v) => (v.x, v.y, false),
        ProjectParamValue::V2i(v) => (v.x as f32, v.y as f32, true),
        _ => {
            return rsx! {
                p { "unexpected value type" }
            }
        }
    };
    let mut pick_target = use_context::<PickTarget>();
    let picking = pick_target.is_target(&props.path, PickPart::Point);
    let pick_path = props.path.clone();
    let hover_path = props.path.clone();
    let drop_path = props.path.clone();
    use_drop(move || pick_target.unhover_path(&drop_path));
    let pickable = !integer && !props.no_pick;

    rsx! {
        div {
            class: "hrow",
            onmouseenter: move |_| {
                if pickable {
                    pick_target.hover(&hover_path, PickPart::Point);
                }
            },
            onmouseleave: move |_| {
                if pickable {
                    pick_target.unhover();
                }
            },
            NumberInput {
                value: x,
                integer,
                affix: "x",
                oncommit: move |x: f32| {
                    leaf.edit(|value| match value {
                            ProjectParamValue::V2(v) => v.x = x,
                            ProjectParamValue::V2i(v) => v.x = x.round() as i32,
                            _ => {}
                        })
                },
            }
            NumberInput {
                value: y,
                integer,
                affix: "y",
                oncommit: move |y: f32| {
                    leaf.edit(|value| match value {
                            ProjectParamValue::V2(v) => v.y = y,
                            ProjectParamValue::V2i(v) => v.y = y.round() as i32,
                            _ => {}
                        })
                },
            }
            if pickable {
                IconButton {
                    icon: IconName::Crosshair,
                    label: "Pick on preview",
                    title: if picking { "Click the preview to set this point" } else { "Pick on preview" },
                    size: Size::Sm,
                    active: picking,
                    onclick: move |_| pick_target.toggle(&pick_path, PickPart::Point),
                }
            }
        }
    }
}
