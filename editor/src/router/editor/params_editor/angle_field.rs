use dioxus::prelude::*;
use plottery_lib::Angle;
use plottery_runtime::{angle_unit::AngleUnit, project_param_value::ProjectParamValue};

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{angle_dial::AngleDial, number_input::NumberInput, segmented::Segmented},
};

#[component]
pub fn AngleField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let (rad, unit, bounds) = match props.param.value {
        ProjectParamValue::Angle { rad, unit } => (rad, unit, None),
        ProjectParamValue::AngleRanged {
            rad,
            unit,
            min,
            max,
        } => (rad, unit, Some((min, max))),
        _ => return rsx! { p { "unexpected value type" } },
    };
    // a typed number is clamped as it is, the dial fits its angles into the range itself
    let clamp = move |value: f32| bounds.map_or(value, |(min, max)| value.clamp(min, max));
    // angle while the dial is dragged, so the number follows before the commit
    let mut live_rad = use_signal(|| None::<f32>);
    let shown = unit.from_angle(Angle::from_rad(live_rad().unwrap_or(rad)));
    let unit_index = AngleUnit::ALL.iter().position(|u| *u == unit);

    rsx! {
        div { class: "AngleField",
            div { class: "reset-target", ondoubleclick: move |_| leaf.reset(),
                AngleDial {
                    rad,
                    range: bounds,
                    oninput: move |value: f32| live_rad.set(Some(value)),
                    oncommit: move |value: f32| {
                        live_rad.set(None);
                        leaf.edit(|param| set_rad(param, value));
                    },
                }
            }
            div { class: "angle-controls",
                NumberInput {
                    value: shown,
                    oncommit: move |value: f32| {
                        leaf.edit(|param| set_rad(param, clamp(unit.to_angle(value).to_rad())))
                    },
                }
                Segmented {
                    options: AngleUnit::ALL.iter().map(|u| u.short_name().to_string()).collect(),
                    selected: unit_index,
                    onselect: move |index: usize| leaf.edit(|param| {
                        if let ProjectParamValue::Angle { unit, .. }
                        | ProjectParamValue::AngleRanged { unit, .. } = param
                        {
                            *unit = AngleUnit::ALL[index];
                        }
                    }),
                }
            }
        }
    }
}

fn set_rad(param: &mut ProjectParamValue, value: f32) {
    if let ProjectParamValue::Angle { rad, .. } | ProjectParamValue::AngleRanged { rad, .. } = param
    {
        *rad = value;
    }
}
