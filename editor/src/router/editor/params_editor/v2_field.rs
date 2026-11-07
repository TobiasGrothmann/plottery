use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

use crate::components::{
    icon::IconName,
    ui::{IconButton, Size},
};
use crate::router::editor::{
    params_editor::{params_editor::LeafProps, widgets::number_input::NumberInput},
    pick_target::PickTarget,
};

#[component]
pub fn V2Field(props: LeafProps) -> Element {
    let (x, y, integer) = match props.param.value {
        ProjectParamValue::V2(v) => (v.x, v.y, false),
        ProjectParamValue::V2i(v) => (v.x as f32, v.y as f32, true),
        _ => {
            return rsx! {
                p { "unexpected value type" }
            }
        }
    };
    let x_props = props.clone();
    let y_props = props.clone();
    let mut pick_target = use_context::<PickTarget>();
    let picking = pick_target.is_target(&props.path);
    let pick_path = props.path.clone();

    rsx! {
        div { class: "hrow",
            NumberInput {
                value: x,
                integer,
                affix: "x",
                oncommit: move |x: f32| {
                    x_props
                        .edit(|value| match value {
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
                    y_props
                        .edit(|value| match value {
                            ProjectParamValue::V2(v) => v.y = y,
                            ProjectParamValue::V2i(v) => v.y = y.round() as i32,
                            _ => {}
                        })
                },
            }
            if !integer {
                IconButton {
                    icon: IconName::Crosshair,
                    label: "Pick on preview",
                    title: if picking { "Click the preview to set this point" } else { "Pick on preview" },
                    size: Size::Sm,
                    active: picking,
                    onclick: move |_| pick_target.toggle(&pick_path),
                }
            }
        }
    }
}
