use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::number_input::NumberInput,
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

#[component]
pub fn RangeField(props: LeafProps) -> Element {
    let Some(view) = RangeView::from_value(&props.param.value) else {
        return rsx! { p { "unexpected value type" } };
    };
    let min_props = props.clone();
    let max_props = props.clone();
    let bounds_text = view
        .bounds
        .map(|(lo, hi)| format!("({lo} – {hi})"))
        .unwrap_or_default();

    rsx! {
        div { class: "hrow",
            NumberInput {
                value: view.min,
                integer: view.is_int,
                placeholder: "min",
                aria_label: "Minimum",
                oncommit: move |new_min: f32| min_props.edit(|value| set_range(value, new_min, view.max)),
            }
            span { class: "hint", "–" }
            NumberInput {
                value: view.max,
                integer: view.is_int,
                placeholder: "max",
                aria_label: "Maximum",
                oncommit: move |new_max: f32| max_props.edit(|value| set_range(value, view.min, new_max)),
            }
            if !bounds_text.is_empty() {
                span { class: "hint nowrap", "{bounds_text}" }
            }
        }
    }
}
