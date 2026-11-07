use dioxus::prelude::*;
use plottery_lib::{Margin, Paper, PaperSize};
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{dropdown::Dropdown, format_number, number_input::NumberInput, segmented::Segmented},
};

/// Largest DIN number offered in the dropdown (A0..A6).
const MAX_DIN_NUMBER: u8 = 6;

fn din_number(size: &PaperSize) -> u8 {
    match size {
        PaperSize::DinA(number) | PaperSize::DinB(number) => (*number).min(MAX_DIN_NUMBER),
        PaperSize::Custom(_) => 4,
    }
}

fn edit_paper(props: &LeafProps, edit: impl FnOnce(&mut Paper)) {
    props.edit(|value| {
        if let ProjectParamValue::Paper(paper) = value {
            edit(paper);
        }
    });
}

#[component]
pub fn PaperField(props: LeafProps) -> Element {
    let ProjectParamValue::Paper(paper) = props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };

    let size_kind = match paper.size {
        PaperSize::DinA(_) => 0,
        PaperSize::DinB(_) => 1,
        PaperSize::Custom(_) => 2,
    };
    let din_letter = if size_kind == 1 { "B" } else { "A" };
    let din_options: Vec<String> = (0..=MAX_DIN_NUMBER)
        .map(|n| format!("{din_letter}{n}"))
        .collect();
    let (margin_kind, margin_value) = match paper.margin {
        Margin::Cm(cm) => (0, cm),
        Margin::Fraction(fraction) => (1, fraction),
    };
    let resolved = paper.size();
    let resolved_text = format!(
        "{} × {} cm",
        format_number(resolved.x),
        format_number(resolved.y)
    );

    let kind_props = props.clone();
    let number_props = props.clone();
    let width_props = props.clone();
    let height_props = props.clone();
    let landscape_props = props.clone();
    let margin_kind_props = props.clone();
    let margin_props = props.clone();

    rsx! {
        style { { include_str!("paper_field.css") } }
        div { class: "PaperField",
            span { class: "paper_label", "size" }
            Segmented {
                options: vec!["A".to_string(), "B".to_string(), "custom".to_string()],
                selected: Some(size_kind),
                onselect: move |index: usize| edit_paper(&kind_props, |paper| {
                    let number = din_number(&paper.size);
                    paper.size = match index {
                        0 => PaperSize::DinA(number),
                        1 => PaperSize::DinB(number),
                        _ => PaperSize::Custom(paper.with_landscape(false).size()),
                    };
                }),
            }

            if let PaperSize::Custom(size) = paper.size {
                span { class: "paper_label", "cm" }
                div { class: "paper_row",
                    NumberInput {
                        value: size.x,
                        placeholder: "width",
                        oncommit: move |x: f32| edit_paper(&width_props, |paper| {
                            if let PaperSize::Custom(size) = &mut paper.size { size.x = x.max(0.0); }
                        }),
                    }
                    span { "×" }
                    NumberInput {
                        value: size.y,
                        placeholder: "height",
                        oncommit: move |y: f32| edit_paper(&height_props, |paper| {
                            if let PaperSize::Custom(size) = &mut paper.size { size.y = y.max(0.0); }
                        }),
                    }
                }
            } else {
                span { class: "paper_label", "format" }
                Dropdown {
                    options: din_options,
                    selected: Some(din_number(&paper.size) as usize),
                    onselect: move |index: usize| edit_paper(&number_props, |paper| {
                        paper.size = match paper.size {
                            PaperSize::DinB(_) => PaperSize::DinB(index as u8),
                            _ => PaperSize::DinA(index as u8),
                        };
                    }),
                }
            }

            span { class: "paper_label", "orientation" }
            Segmented {
                options: vec!["portrait".to_string(), "landscape".to_string()],
                selected: Some(if paper.landscape { 1 } else { 0 }),
                onselect: move |index: usize| edit_paper(&landscape_props, |paper| paper.landscape = index == 1),
            }

            span { class: "paper_label", "margin" }
            div { class: "paper_row",
                NumberInput {
                    value: margin_value,
                    oncommit: move |value: f32| edit_paper(&margin_props, |paper| {
                        paper.margin = match paper.margin {
                            Margin::Cm(_) => Margin::Cm(value.max(0.0)),
                            Margin::Fraction(_) => Margin::Fraction(value.max(0.0)),
                        };
                    }),
                }
                Segmented {
                    options: vec!["cm".to_string(), "fraction".to_string()],
                    selected: Some(margin_kind),
                    onselect: move |index: usize| edit_paper(&margin_kind_props, |paper| {
                        let margin_cm = paper.margin_cm();
                        paper.margin = if index == 0 {
                            Margin::Cm(margin_cm)
                        } else {
                            Margin::Fraction(margin_cm / paper.size().min_axis().max(f32::EPSILON))
                        };
                    }),
                }
            }

            span { class: "paper_label", "result" }
            p { class: "paper_resolved", "{resolved_text}" }
        }
    }
}
