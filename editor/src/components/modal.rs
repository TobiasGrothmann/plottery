use dioxus::prelude::*;

use crate::components::{
    icon::IconName,
    ui::{IconButton, Size},
};

const FOCUS_FIRST_FIELD: &str = r#"
const modal = document.querySelector('.scrim .modal');
const field = modal && modal.querySelector('.modal-body input:not([type=checkbox]), .modal-body select, .modal-body textarea');
(field || document.querySelector('.scrim')).focus();
"#;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ModalWidth {
    #[default]
    Form,
    Table,
}

#[component]
pub fn Modal(
    open: Signal<bool>,
    title: String,
    subtitle: Option<String>,
    #[props(default)] width: ModalWidth,
    body: Element,
    footer: Element,
) -> Element {
    if !open() {
        return VNode::empty();
    }
    let width_class = match width {
        ModalWidth::Form => "modal-form",
        ModalWidth::Table => "modal-table",
    };
    rsx! {
        div {
            class: "scrim",
            tabindex: "-1",
            onmounted: move |_| {
                document::eval(FOCUS_FIRST_FIELD);
            },
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    event.stop_propagation();
                    open.set(false);
                }
            },
            onclick: move |_| open.set(false),
            div {
                class: "modal {width_class}",
                role: "dialog",
                "aria-modal": "true",
                "aria-label": "{title}",
                onclick: move |event| event.stop_propagation(),
                div { class: "modal-head",
                    div {
                        h2 { class: "modal-title", "{title}" }
                        if let Some(subtitle) = subtitle {
                            p { class: "modal-sub", "{subtitle}" }
                        }
                    }
                    IconButton {
                        icon: IconName::X,
                        label: "Close",
                        size: Size::Sm,
                        onclick: move |_| open.set(false),
                    }
                }
                div { class: "modal-body scroll", {body} }
                div { class: "modal-foot", {footer} }
            }
        }
    }
}
