use dioxus::prelude::*;

use crate::router::editor::console_messages::ConsoleMessageType;

use super::console_messages::ConsoleMessages;

#[derive(PartialEq, Props, Clone)]
pub struct ConsoleProps {
    pub console: SyncSignal<ConsoleMessages>,
}

#[component]
pub fn Console(props: ConsoleProps) -> Element {
    let messages = props.console.read().get_messages();
    let _change_counter = props.console.read().get_change_counter(); // to update this component when the console changes

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
        div { class: "Console",
            { console_items },
        },
    }
}
