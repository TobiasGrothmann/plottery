use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct SegmentedProps {
    pub options: Vec<String>,
    pub selected: Option<usize>,
    pub onselect: EventHandler<usize>,
}

/// A row of buttons of which one is active.
#[component]
pub fn Segmented(props: SegmentedProps) -> Element {
    rsx! {
        style { { include_str!("widgets.css") } }
        div { class: "Segmented",
            for (index, option) in props.options.iter().enumerate() {
                button {
                    key: "{option}",
                    r#type: "button",
                    class: if props.selected == Some(index) { "active_button" } else { "" },
                    onclick: move |_| props.onselect.call(index),
                    "{option}"
                }
            }
        }
    }
}
