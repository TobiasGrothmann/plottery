use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{dropdown::Dropdown, segmented::Segmented},
};

/// Variants up to this count render as buttons, more as a dropdown.
const MAX_SEGMENTED_VARIANTS: usize = 4;

/// `SomeVariantName` → `some variant name`
pub fn variant_display_name(variant: &str) -> String {
    let mut out = String::with_capacity(variant.len() + 4);
    for (i, c) in variant.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push(' ');
        }
        out.extend(c.to_lowercase());
    }
    out.replace('_', " ")
}

/// The variant selector of an enum, with or without fields.
#[component]
pub fn EnumField(props: LeafProps) -> Element {
    let (variants, selected) = match &props.param.value {
        ProjectParamValue::Enum(param_enum) => {
            (param_enum.variants.clone(), param_enum.selected_index())
        }
        ProjectParamValue::DataEnum(data_enum) => {
            (data_enum.variant_names(), data_enum.selected_index())
        }
        _ => return rsx! { p { "unexpected value type" } },
    };
    let options: Vec<String> = variants.iter().map(|v| variant_display_name(v)).collect();

    let edit_props = props.clone();
    let onselect = move |index: usize| {
        let Some(variant) = variants.get(index).cloned() else {
            return;
        };
        edit_props.edit(|value| match value {
            ProjectParamValue::Enum(param_enum) => param_enum.selected = variant,
            ProjectParamValue::DataEnum(data_enum) => data_enum.selected = variant,
            _ => {}
        });
    };

    if options.len() <= MAX_SEGMENTED_VARIANTS {
        rsx! {
            Segmented { options, selected, onselect }
        }
    } else {
        rsx! {
            Dropdown { options, selected, onselect }
        }
    }
}
