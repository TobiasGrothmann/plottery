use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct TextInputProps {
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    pub oncommit: EventHandler<String>,
}

/// Multi-line text input that grows with its content, commits on blur.
#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    let rows = props.value.lines().count().max(1);
    rsx! {
        style { { include_str!("widgets.css") } }
        div { class: "TextInput",
            textarea {
                rows: "{rows}",
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                onchange: move |event| props.oncommit.call(event.value()),
            }
        }
    }
}
