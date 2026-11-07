use dioxus::prelude::*;
use plottery_lib::rand_range_i;
use plottery_project::{project_param_value::ProjectParamValue, ProjectParamSeed};

use crate::{
    components::{
        icon::IconName,
        ui::{Button, Chip, IconButton, Size, Variant},
    },
    router::editor::params_editor::{params_editor::LeafProps, widgets::number_input::NumberInput},
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
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let history = seed.history.clone();
    let value_props = props.clone();
    let random_props = props.clone();
    let clear_props = props.clone();

    rsx! {
        div { class: "hrow",
            NumberInput {
                value: seed.current as f32,
                integer: true,
                aria_label: "Seed",
                oncommit: move |value: f32| edit_seed(&value_props, |seed| seed.set(value as i32)),
            }
            IconButton {
                icon: IconName::Dice,
                label: "Random seed",
                size: Size::Sm,
                onclick: move |_| edit_seed(&random_props, |seed| seed.set(rand_range_i(0, 101))),
            }
        }
        if !history.is_empty() {
            div { class: "hrow wrap",
                span { class: "hint", "Recent" }
                for previous in history.iter().copied() {
                    {
                        let chip_props = props.clone();
                        rsx! {
                            Chip {
                                key: "{previous}",
                                title: "Use seed {previous} again",
                                onclick: move |_| edit_seed(&chip_props, |seed| seed.set(previous)),
                                "{previous}"
                            }
                        }
                    }
                }
                Button {
                    variant: Variant::Ghost,
                    size: Size::Sm,
                    title: "Clear seed history",
                    onclick: move |_| edit_seed(&clear_props, |seed| seed.clear_history()),
                    "Clear"
                }
            }
        }
    }
}
