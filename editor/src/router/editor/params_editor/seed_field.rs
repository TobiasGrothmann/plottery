use dioxus::prelude::*;
use plottery_lib::rand_range_i;
use plottery_runtime::{project_param_value::ProjectParamValue, ProjectParamSeed};

use crate::{
    components::{
        icon::IconName,
        ui::{Button, Chip, IconButton, Size, Variant},
    },
    router::editor::params_editor::{
        params_editor::{LeafHandle, LeafProps},
        widgets::number_input::NumberInput,
    },
};

fn edit_seed(leaf: LeafHandle, edit: impl FnOnce(&mut ProjectParamSeed)) {
    leaf.edit(|value| {
        if let ProjectParamValue::Seed(seed) = value {
            edit(seed);
        }
    });
}

#[component]
pub fn SeedField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let ProjectParamValue::Seed(seed) = &props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let history = seed.history.clone();

    rsx! {
        div { class: "hrow",
            NumberInput {
                value: seed.current as f32,
                integer: true,
                aria_label: "Seed",
                oncommit: move |value: f32| edit_seed(leaf, |seed| seed.set(value as i32)),
            }
            IconButton {
                icon: IconName::Dice,
                label: "Random seed",
                size: Size::Sm,
                onclick: move |_| edit_seed(leaf, |seed| seed.set(rand_range_i(100, 1000))),
            }
        }
        if !history.is_empty() {
            div { class: "hrow wrap",
                span { class: "hint", "Recent" }
                for previous in history.iter().copied() {
                    {
                        rsx! {
                            Chip {
                                key: "{previous}",
                                title: "Use seed {previous} again",
                                onclick: move |_| edit_seed(leaf, |seed| seed.set(previous)),
                                "{previous}"
                            }
                        }
                    }
                }
                Button {
                    variant: Variant::Ghost,
                    size: Size::Sm,
                    title: "Clear seed history",
                    onclick: move |_| edit_seed(leaf, |seed| seed.clear_history()),
                    "Clear"
                }
            }
        }
    }
}
