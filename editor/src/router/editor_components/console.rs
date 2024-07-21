use dioxus::prelude::*;

use super::editor_console::EditorConsole;

#[derive(PartialEq, Props, Clone)]
pub struct ConsoleProps {
    pub console: SyncSignal<EditorConsole>,
}

#[component]
pub fn Console(props: ConsoleProps) -> Element {
    let messages = props.console.read().get_messages();

    rsx! {
        style { { include_str!("console.css") } }
        div { class: "Console",
            for message in messages.iter() {
                p { class: "info",
                    "{message.msg}",
                }
            }
        }
    }
}
