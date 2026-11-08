use dioxus::prelude::*;

use crate::components::icon::{Icon, IconName};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Variant {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Dashed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Size {
    #[default]
    Md,
    Sm,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Ok,
    Busy,
    Warn,
    Err,
}

fn button_class(variant: Variant, size: Size) -> String {
    let variant = match variant {
        Variant::Primary => " btn-primary",
        Variant::Secondary => "",
        Variant::Ghost => " btn-ghost",
        Variant::Dashed => " btn-dashed",
    };
    let size = match size {
        Size::Md => "",
        Size::Sm => " btn-sm",
    };
    format!("btn{variant}{size}")
}

#[component]
pub fn Button(
    #[props(default)] variant: Variant,
    #[props(default)] size: Size,
    icon: Option<IconName>,
    title: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "{button_class(variant, size)} {class}",
            title,
            disabled,
            onclick: move |event| onclick.call(event),
            if let Some(icon) = icon {
                Icon { name: icon }
            }
            {children}
        }
    }
}

#[component]
pub fn IconButton(
    icon: IconName,
    label: String,
    #[props(default)] size: Size,
    #[props(default = Variant::Ghost)] variant: Variant,
    #[props(default)] active: bool,
    title: Option<String>,
    #[props(default)] icon_class: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    let on = if active { " on" } else { "" };
    let title = title.unwrap_or_else(|| label.clone());
    rsx! {
        button {
            r#type: "button",
            class: "{button_class(variant, size)} icon-btn{on}",
            "aria-label": "{label}",
            "aria-pressed": if active { "true" },
            title: "{title}",
            onclick: move |event| onclick.call(event),
            Icon { name: icon, class: icon_class }
        }
    }
}

#[component]
pub fn Segmented<T: Clone + PartialEq + 'static>(
    options: Vec<(T, String)>,
    value: T,
    onchange: EventHandler<T>,
) -> Element {
    rsx! {
        div { class: "seg", role: "group",
            for (option, text) in options.into_iter() {
                button {
                    key: "{text}",
                    r#type: "button",
                    class: if option == value { "on" },
                    "aria-pressed": if option == value { "true" } else { "false" },
                    onclick: move |_| onchange.call(option.clone()),
                    "{text}"
                }
            }
        }
    }
}

fn toggle_input(
    class: &'static str,
    checked: bool,
    label: Option<String>,
    aria_label: Option<String>,
    label_class: String,
    onchange: EventHandler<bool>,
) -> Element {
    let input = rsx! {
        input {
            r#type: "checkbox",
            class,
            checked,
            "aria-label": aria_label.or(label.clone()),
            onchange: move |event| onchange.call(event.value().parse().unwrap_or(false)),
        }
    };
    match label {
        Some(label) => rsx! {
            label { class: "hrow toggle-label {label_class}",
                {input}
                span { "{label}" }
            }
        },
        None => input,
    }
}

#[component]
pub fn Switch(
    checked: bool,
    onchange: EventHandler<bool>,
    label: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    toggle_input("switch", checked, label, aria_label, class, onchange)
}

#[component]
pub fn Checkbox(
    checked: bool,
    onchange: EventHandler<bool>,
    label: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    toggle_input("check", checked, label, aria_label, class, onchange)
}

#[component]
pub fn Chip(
    title: Option<String>,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "chip",
            title,
            onclick: move |event| onclick.call(event),
            {children}
        }
    }
}

#[component]
pub fn StatusPill(tone: Tone, text: String, title: Option<String>) -> Element {
    let tone = match tone {
        Tone::Ok => "ok",
        Tone::Busy => "busy",
        Tone::Warn => "warn",
        Tone::Err => "err",
    };
    rsx! {
        span { class: "pill", role: "status", title,
            span { class: "dot {tone}" }
            span { "{text}" }
        }
    }
}

#[component]
pub fn PanelHeader(
    title: String,
    #[props(default = VNode::empty())] actions: Element,
    on_collapse: Option<EventHandler<()>>,
    #[props(default = IconName::ChevronLeft)] collapse_icon: IconName,
) -> Element {
    rsx! {
        div { class: "panel-head",
            span { class: "panel-title",
                "{title}"
            }
            div { class: "hrow",
                {actions}
                if let Some(on_collapse) = on_collapse {
                    IconButton {
                        icon: collapse_icon,
                        label: "Hide {title.to_lowercase()}",
                        size: Size::Sm,
                        onclick: move |_| on_collapse.call(()),
                    }
                }
            }
        }
    }
}
