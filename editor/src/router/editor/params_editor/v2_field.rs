use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::{
    params_editor::{params_editor::LeafProps, widgets::number_input::NumberInput},
    pick_target::PickTarget,
};

#[component]
pub fn V2Field(props: LeafProps) -> Element {
    let (x, y, integer) = match props.param.value {
        ProjectParamValue::V2(v) => (v.x, v.y, false),
        ProjectParamValue::V2i(v) => (v.x as f32, v.y as f32, true),
        _ => return rsx! { p { "unexpected value type" } },
    };
    let x_props = props.clone();
    let y_props = props.clone();
    let mut pick_target = use_context::<PickTarget>();
    let picking = pick_target.is_target(&props.path);
    let pick_path = props.path.clone();

    rsx! {
        div { class: "V2Field", style: "display: flex; align-items: center; gap: var(--gap-0);",
            span { "x" }
            NumberInput {
                value: x,
                integer,
                oncommit: move |x: f32| x_props.edit(|value| match value {
                    ProjectParamValue::V2(v) => v.x = x,
                    ProjectParamValue::V2i(v) => v.x = x.round() as i32,
                    _ => {}
                }),
            }
            span { "y" }
            NumberInput {
                value: y,
                integer,
                oncommit: move |y: f32| y_props.edit(|value| match value {
                    ProjectParamValue::V2(v) => v.y = y,
                    ProjectParamValue::V2i(v) => v.y = y.round() as i32,
                    _ => {}
                }),
            }
            button {
                r#type: "button",
                class: if picking { "v2_pick_button active_button" } else { "v2_pick_button" },
                style: "min-height: 0; padding: var(--gap-small) var(--gap-0); font-size: 14px; white-space: nowrap;",
                title: "click on the preview to set this point",
                onclick: move |_| pick_target.toggle(&pick_path),
                if picking { "click preview…" } else { "pick" }
            }
        }
    }
}
