use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

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

#[component]
pub fn EnumField(props: LeafProps) -> Element {
    let ProjectParamValue::Enum(param_enum) = &props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let variants = param_enum.variants.clone();
    let options: Vec<String> = variants.iter().map(|v| variant_display_name(v)).collect();
    let selected = param_enum.selected_index();

    let edit_props = props.clone();
    let onselect = move |index: usize| {
        let Some(variant) = variants.get(index).cloned() else {
            return;
        };
        edit_props.edit(|value| {
            if let ProjectParamValue::Enum(param_enum) = value {
                param_enum.selected = variant;
            }
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
