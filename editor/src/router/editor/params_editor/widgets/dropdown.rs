use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct DropdownProps {
    pub options: Vec<String>,
    pub selected: Option<usize>,
    #[props(default)]
    pub placeholder: Option<String>,
    pub onselect: EventHandler<usize>,
}

#[component]
pub fn Dropdown(props: DropdownProps) -> Element {
    // Select via the options: a select `value` is applied before its options exist.
    rsx! {
        select {
            class: "input select",
            onchange: move |event| {
                if let Ok(index) = event.value().parse::<usize>() {
                    props.onselect.call(index);
                }
            },
            if let Some(placeholder) = props.placeholder.clone() {
                option {
                    value: "",
                    disabled: true,
                    selected: props.selected.is_none(),
                    "{placeholder}"
                }
            }
            for (index, option) in props.options.iter().enumerate() {
                option {
                    key: "{option}",
                    value: "{index}",
                    selected: props.selected == Some(index),
                    "{option}"
                }
            }
        }
    }
}
