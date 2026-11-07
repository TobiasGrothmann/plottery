use dioxus::prelude::*;
use plottery_lib::rand_range_i;
use plottery_project::{project_param_value::ProjectParamValue, ProjectParamSeed};

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::number_input::NumberInput,
};

fn edit_seed(props: &LeafProps, edit: impl FnOnce(&mut ProjectParamSeed)) {
    props.edit(|value| {
        if let ProjectParamValue::Seed(seed) = value {
            edit(seed);
        }
    });
}

#[component]
pub fn SeedField(props: LeafProps) -> Element {
    let ProjectParamValue::Seed(seed) = &props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let history = seed.history.clone();
    let value_props = props.clone();
    let random_props = props.clone();
    let clear_props = props.clone();

    rsx! {
        style { { include_str!("seed_field.css") } }
        div { class: "SeedField",
            div { class: "seed_row",
                NumberInput {
                    value: seed.current as f32,
                    integer: true,
                    oncommit: move |value: f32| edit_seed(&value_props, |seed| seed.set(value as i32)),
                }
                button {
                    r#type: "button",
                    class: "seed_button",
                    onclick: move |_| edit_seed(&random_props, |seed| seed.set(rand_range_i(0, i32::MAX))),
                    "random"
                }
            }
            if !history.is_empty() {
                div { class: "seed_history",
                    for previous in history.iter().copied() {
                        {
                            let chip_props = props.clone();
                            rsx! {
                                button {
                                    key: "{previous}",
                                    r#type: "button",
                                    class: "seed_chip",
                                    title: "use this seed again",
                                    onclick: move |_| edit_seed(&chip_props, |seed| seed.set(previous)),
                                    "{previous}"
                                }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "seed_chip seed_clear",
                        title: "clear history",
                        onclick: move |_| edit_seed(&clear_props, |seed| seed.clear_history()),
                        "clear"
                    }
                }
            }
        }
    }
}
