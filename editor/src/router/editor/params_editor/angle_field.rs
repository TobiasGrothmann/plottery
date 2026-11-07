use dioxus::prelude::*;
use plottery_lib::Angle;
use plottery_project::{angle_unit::AngleUnit, project_param_value::ProjectParamValue};

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{number_input::NumberInput, segmented::Segmented},
};

#[component]
pub fn AngleField(props: LeafProps) -> Element {
    let ProjectParamValue::Angle { rad, unit } = props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let shown = unit.from_angle(Angle::from_rad(rad));
    let unit_index = AngleUnit::ALL.iter().position(|u| *u == unit);

    let value_props = props.clone();
    let unit_props = props.clone();

    rsx! {
        div { class: "AngleField", style: "display: flex; align-items: center; gap: var(--gap-0);",
            NumberInput {
                value: shown,
                oncommit: move |value: f32| value_props.edit(|param| {
                    if let ProjectParamValue::Angle { rad, unit } = param {
                        *rad = unit.to_angle(value).to_rad();
                    }
                }),
            }
            Segmented {
                options: AngleUnit::ALL.iter().map(|u| u.short_name().to_string()).collect(),
                selected: unit_index,
                onselect: move |index: usize| unit_props.edit(|param| {
                    if let ProjectParamValue::Angle { unit, .. } = param {
                        *unit = AngleUnit::ALL[index];
                    }
                }),
            }
        }
    }
}
