use dioxus::prelude::*;

use crate::{
    components::{
        icon::IconName,
        ui::{IconButton, Size, StatusPill, Tone},
    },
    router::editor::console_messages::ConsoleMessageType,
};

use super::console_messages::ConsoleMessages;

#[derive(PartialEq, Props, Clone)]
pub struct ConsoleProps {
    pub console: SyncSignal<ConsoleMessages>,
    pub open: bool,
    pub height: f64,
    pub on_toggle: EventHandler<()>,
}

#[component]
pub fn Console(props: ConsoleProps) -> Element {
    let messages = props.console.read().get_messages();
    let _change_counter = props.console.read().get_change_counter(); // to update this component when the console changes
    let error_count = messages
        .iter()
        .filter(|message| matches!(message.msg_type, ConsoleMessageType::Error))
        .count();
    let error_text = match error_count {
        1 => "1 error".to_string(),
        n => format!("{n} errors"),
    };

    let console_items = messages.iter().rev().map(|message| {
        let type_class_name = match message.msg_type {
            ConsoleMessageType::Error => "error",
            ConsoleMessageType::Info => "info",
            ConsoleMessageType::ProjectLog => "project_log",
        };
        rsx! {
            p { class: "{type_class_name}",
                key: "{message.id}",
                "{message.msg}",
            }
        }
    });

    rsx! {
        style { { include_str!("console.css") } }
        section { class: "console",
            div { class: "console-head",
                IconButton {
                    icon: IconName::ChevronDown,
                    label: if props.open { "Hide console" } else { "Show console" },
                    size: Size::Sm,
                    icon_class: if props.open { "" } else { "rot-90" },
                    onclick: move |_| props.on_toggle.call(()),
                }
                span { class: "panel-title",
                    "Console"
                    span { class: "count", "{messages.len()}" }
                }
                if error_count > 0 {
                    StatusPill { tone: Tone::Err, text: error_text }
                }
                div { class: "console-actions",
                    IconButton {
                        icon: IconName::Trash,
                        label: "Clear console",
                        size: Size::Sm,
                        onclick: move |_| props.console.read().clear(),
                    }
                }
            }
            if props.open {
                div { class: "console-body Console scroll", style: "height: {props.height}px;",
                    { console_items },
                }
            }
        }
    }
}
