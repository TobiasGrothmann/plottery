use dioxus::prelude::*;
use plottery_lib::{NoiseScale, NoiseSettings, NoiseType};
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{number_input::NumberInput, segmented::Segmented},
};

fn edit_noise(props: &LeafProps, edit: impl FnOnce(&mut NoiseSettings)) {
    props.edit(|value| {
        if let ProjectParamValue::Noise(noise) = value {
            edit(noise);
        }
    });
}

#[component]
pub fn NoiseField(props: LeafProps) -> Element {
    let ProjectParamValue::Noise(noise) = props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let type_index = NoiseType::ALL.iter().position(|t| *t == noise.noise_type);
    let scale_index = match noise.scale {
        NoiseScale::PerCm => 0,
        NoiseScale::PerPaper => 1,
    };

    let type_props = props.clone();
    let frequency_props = props.clone();
    let octaves_props = props.clone();
    let scale_props = props.clone();

    rsx! {
        div {
            Segmented {
                options: vec!["perlin".to_string(), "simplex".to_string(), "worley".to_string()],
                selected: type_index,
                onselect: move |index: usize| edit_noise(
                    &type_props,
                    |noise| {
                        noise.noise_type = NoiseType::ALL[index];
                    },
                ),
            }
        }
        div { class: "sub",
            span { class: "sublabel", "frequency" }
            div { class: "hrow",
                NumberInput {
                    aria_label: "Frequency",
                    value: noise.frequency,
                    oncommit: move |value: f32| edit_noise(
                        &frequency_props,
                        |noise| {
                            noise.frequency = value.max(0.0);
                        },
                    ),
                }
                Segmented {
                    options: vec!["per cm".to_string(), "per paper".to_string()],
                    selected: Some(scale_index),
                    onselect: move |index: usize| edit_noise(
                        &scale_props,
                        |noise| {
                            noise.scale = if index == 0 {
                                NoiseScale::PerCm
                            } else {
                                NoiseScale::PerPaper
                            };
                        },
                    ),
                }
            }
        }
        div { class: "sub",
            span { class: "sublabel", "octaves" }
            NumberInput {
                aria_label: "Octaves",
                value: noise.octaves as f32,
                integer: true,
                oncommit: move |value: f32| edit_noise(
                    &octaves_props,
                    |noise| {
                        noise.octaves = (value.max(1.0)) as u32;
                    },
                ),
            }
        }
    }
}
