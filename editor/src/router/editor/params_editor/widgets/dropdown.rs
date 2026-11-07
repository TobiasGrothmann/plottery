use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct DropdownProps {
    pub options: Vec<String>,
    pub selected: Option<usize>,
    pub onselect: EventHandler<usize>,
}

#[component]
pub fn Dropdown(props: DropdownProps) -> Element {
    let selected = props.selected.map(|i| i.to_string()).unwrap_or_default();
    rsx! {
        style { { include_str!("widgets.css") } }
        div { class: "Dropdown",
            select {
                value: "{selected}",
                onchange: move |event| {
                    if let Ok(index) = event.value().parse::<usize>() {
                        props.onselect.call(index);
                    }
                },
                for (index, option) in props.options.iter().enumerate() {
                    option { key: "{option}", value: "{index}", "{option}" }
                }
            }
        }
    }
}
