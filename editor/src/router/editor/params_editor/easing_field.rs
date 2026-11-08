use dioxus::prelude::*;
use plottery_lib::{Easing, EasingKind};
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{segmented::Segmented, slider_input::SliderInput},
};

const PREVIEW_SIZE: f32 = 48.0;
const PREVIEW_INSET: f32 = 3.0;
const PREVIEW_SAMPLES: usize = 32;

fn edit_easing(props: &LeafProps, edit: impl FnOnce(&mut Easing)) {
    props.edit(|value| {
        if let ProjectParamValue::Easing(easing) = value {
            edit(easing);
        }
    });
}

/// SVG `points` of the curve, y pointing up.
fn preview_points(easing: &Easing) -> String {
    let span = PREVIEW_SIZE - 2.0 * PREVIEW_INSET;
    (0..=PREVIEW_SAMPLES)
        .map(|i| i as f32 / PREVIEW_SAMPLES as f32)
        .map(|t| {
            let x = PREVIEW_INSET + t * span;
            let y = PREVIEW_SIZE - PREVIEW_INSET - easing.apply(t) * span;
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn EasingField(props: LeafProps) -> Element {
    let ProjectParamValue::Easing(easing) = props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let kind_index = EasingKind::ALL.iter().position(|k| *k == easing.kind);
    // amount while the slider is dragged, so the preview follows before the commit
    let mut live_amount = use_signal(|| None::<f32>);
    let preview = Easing {
        amount: live_amount().unwrap_or(easing.amount),
        ..easing
    };
    let points = preview_points(&preview);
    let (lo, hi) = (PREVIEW_INSET, PREVIEW_SIZE - PREVIEW_INSET);

    let kind_props = props.clone();
    let amount_props = props.clone();

    rsx! {
        div { class: "EasingField",
            div { class: "easing-controls",
                Segmented {
                    options: vec!["power".to_string(), "bias".to_string(), "s-curve".to_string()],
                    selected: kind_index,
                    onselect: move |index: usize| edit_easing(
                        &kind_props,
                        |easing| {
                            easing.kind = EasingKind::ALL[index];
                        },
                    ),
                }
                SliderInput {
                    value: easing.amount,
                    min: -1.0,
                    max: 1.0,
                    oninput: move |value: f32| live_amount.set(Some(value)),
                    oncommit: move |value: f32| {
                        live_amount.set(None);
                        edit_easing(&amount_props, |easing| easing.amount = value);
                    },
                }
            }
            svg {
                class: "easing-preview",
                width: "{PREVIEW_SIZE}",
                height: "{PREVIEW_SIZE}",
                view_box: "0 0 {PREVIEW_SIZE} {PREVIEW_SIZE}",
                line {
                    class: "easing-diagonal",
                    x1: "{lo}",
                    y1: "{hi}",
                    x2: "{hi}",
                    y2: "{lo}",
                }
                polyline { class: "easing-line", points: "{points}" }
            }
        }
    }
}
