use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{number_input::NumberInput, range_slider_input::RangeSliderInput},
};

#[derive(Clone, Copy)]
struct RangeView {
    min: f32,
    max: f32,
    bounds: Option<(f32, f32)>,
    is_int: bool,
}

impl RangeView {
    fn from_value(value: &ProjectParamValue) -> Option<Self> {
        match value {
            ProjectParamValue::RangeF32 { min, max, bounds } => Some(Self {
                min: *min,
                max: *max,
                bounds: *bounds,
                is_int: false,
            }),
            ProjectParamValue::RangeI32 { min, max, bounds } => Some(Self {
                min: *min as f32,
                max: *max as f32,
                bounds: bounds.map(|(lo, hi)| (lo as f32, hi as f32)),
                is_int: true,
            }),
            _ => None,
        }
    }
}

/// Writes `new_min..new_max` ordered and clamped to the bounds.
fn set_range(value: &mut ProjectParamValue, new_min: f32, new_max: f32) {
    let (lo, hi) = if new_min <= new_max {
        (new_min, new_max)
    } else {
        (new_max, new_min)
    };
    match value {
        ProjectParamValue::RangeF32 { min, max, bounds } => {
            let clamp = |v: f32| bounds.map_or(v, |(b_lo, b_hi)| v.clamp(b_lo, b_hi));
            *min = clamp(lo);
            *max = clamp(hi);
        }
        ProjectParamValue::RangeI32 { min, max, bounds } => {
            let clamp = |v: f32| {
                let v = v.round() as i32;
                bounds.map_or(v, |(b_lo, b_hi)| v.clamp(b_lo, b_hi))
            };
            *min = clamp(lo);
            *max = clamp(hi);
        }
        _ => {}
    }
}

/// Min and max boxes; with `#[range]` bounds a two-thumb slider between them.
#[component]
pub fn RangeField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    // values while the slider is dragged, so the boxes follow before the commit
    let mut live = use_signal(|| None::<(f32, f32)>);
    let Some(view) = RangeView::from_value(&props.param.value) else {
        return rsx! { p { "unexpected value type" } };
    };
    let (shown_min, shown_max) = live().unwrap_or((view.min, view.max));
    let min_box = rsx! {
        NumberInput {
            value: shown_min,
            integer: view.is_int,
            placeholder: "min",
            aria_label: "Minimum",
            oncommit: move |new_min: f32| leaf.edit(|value| set_range(value, new_min, view.max)),
        }
    };
    let max_box = rsx! {
        NumberInput {
            value: shown_max,
            integer: view.is_int,
            placeholder: "max",
            aria_label: "Maximum",
            oncommit: move |new_max: f32| leaf.edit(|value| set_range(value, view.min, new_max)),
        }
    };

    match view.bounds {
        Some((lo, hi)) => rsx! {
            div { class: "hrow",
                div { class: "range-box", {min_box} }
                RangeSliderInput {
                    low: view.min,
                    high: view.max,
                    min: lo,
                    max: hi,
                    integer: view.is_int,
                    oninput: move |values: (f32, f32)| live.set(Some(values)),
                    oncommit: move |(low, high): (f32, f32)| {
                        // a press that moved nothing keeps the value as it is
                        if live.take().is_none() {
                            return;
                        }
                        leaf.edit(|value| set_range(value, low, high));
                    },
                }
                div { class: "range-box", {max_box} }
            }
        },
        None => rsx! {
            div { class: "hrow",
                {min_box}
                span { class: "hint", "–" }
                {max_box}
            }
        },
    }
}
