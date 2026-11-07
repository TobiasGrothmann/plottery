use dioxus::prelude::*;

use crate::components::ui;

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
        ui::Segmented {
            options: props.options.iter().cloned().enumerate().collect::<Vec<_>>(),
            value: props.selected.unwrap_or(usize::MAX),
            onchange: props.onselect,
        }
    }
}
