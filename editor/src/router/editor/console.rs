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
            div { class: "console-body Console",
                { console_items },
            }
        }
    }
}
