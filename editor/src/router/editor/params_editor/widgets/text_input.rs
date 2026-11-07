use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct TextInputProps {
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    pub oncommit: EventHandler<String>,
}

/// Text input for a string, commits on Enter or blur.
#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    rsx! {
        style { { include_str!("widgets.css") } }
        div { class: "TextInput",
            input {
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                onchange: move |event| props.oncommit.call(event.value()),
            }
        }
    }
}
