use dioxus::prelude::*;
use plottery_lib::{FontStyle, SystemFont, SystemFontFamily};
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    system_fonts::{cached_families, families, sample_style},
    widgets::{
        dropdown::Dropdown,
        search_picker::{PickerRow, SearchPicker},
    },
};

/// Style names for the dropdown; names a family uses twice get their weight appended.
fn style_labels(family: &SystemFontFamily) -> Vec<String> {
    family
        .faces
        .iter()
        .map(|face| {
            let same_name = family
                .faces
                .iter()
                .filter(|other| other.style_name == face.style_name)
                .count();
            match same_name {
                1 => face.style_name.clone(),
                _ => format!("{} {}", face.style_name, face.style.weight),
            }
        })
        .collect()
}

#[component]
pub fn SystemFontField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let ProjectParamValue::SystemFont(font) = &props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let font = font.clone();

    // The scan blocks for about a second, so it runs off the UI thread, once per process.
    let mut installed = use_signal(cached_families);
    use_hook(move || {
        if installed.peek().is_none() {
            spawn(async move {
                if let Ok(list) = tokio::task::spawn_blocking(families).await {
                    installed.set(Some(list));
                }
            });
        }
    });
    let installed = installed();
    let scanning = installed.is_none();
    let list = installed.unwrap_or_default();

    let family = list
        .iter()
        .find(|family| family.name.eq_ignore_ascii_case(&font.family))
        .cloned();
    let face_index = family.as_ref().and_then(|family| {
        let closest = family.closest_face(font.style)?;
        family.faces.iter().position(|face| face == closest)
    });
    // What `SystemFont::load` draws: the stored style, or the closest one the family has.
    let shown_style = family
        .as_ref()
        .zip(face_index)
        .map_or(font.style, |(family, index)| family.faces[index].style);
    let missing = !scanning && family.is_none();

    let rows: Vec<PickerRow> = list
        .iter()
        .map(|family| {
            let style = family
                .closest_face(FontStyle::default())
                .map_or_else(FontStyle::default, |face| face.style);
            PickerRow {
                key: family.name.clone(),
                content: rsx! {
                    span {
                        class: "font-sample",
                        style: sample_style(&family.name, style),
                        "{family.name}"
                    }
                    span { class: "font-name", "{family.name}" }
                },
            }
        })
        .collect();

    let pick_family = {
        let list = list.clone();
        let font = font.clone();
        move |name: String| {
            let Some(picked) = list.iter().find(|family| family.name == name) else {
                return;
            };
            // keeps bold bold when the new family has it
            let style = picked
                .closest_face(font.style)
                .map_or(font.style, |face| face.style);
            let new_font = SystemFont {
                family: name,
                style,
            };
            if new_font != font {
                leaf.edit(|value| *value = ProjectParamValue::SystemFont(new_font));
            }
        }
    };

    let pick_style = {
        let family = family.clone();
        move |index: usize| {
            let Some(face) = family.as_ref().and_then(|family| family.faces.get(index)) else {
                return;
            };
            let style = face.style;
            leaf.edit(|value| {
                if let ProjectParamValue::SystemFont(font) = value {
                    font.style = style;
                }
            });
        }
    };

    rsx! {
        div { class: "hrow SystemFontField",
            SearchPicker {
                rows,
                selected: family.as_ref().map(|family| family.name.clone()),
                placeholder: "Search installed fonts",
                empty_text: if scanning { "Scanning fonts…" } else { "No font matches" },
                title: "{font.family}",
                onpick: pick_family,
                if missing {
                    span { class: "hint", "'{font.family}' is not installed" }
                } else {
                    span {
                        class: "font-sample",
                        style: sample_style(&font.family, shown_style),
                        "{font.family}"
                    }
                }
            }
            if let Some(family) = family.as_ref().filter(|family| family.faces.len() > 1) {
                div { class: "font-style",
                    Dropdown {
                        options: style_labels(family),
                        selected: face_index,
                        onselect: pick_style,
                    }
                }
            }
        }
    }
}
