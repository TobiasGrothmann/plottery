use dioxus::prelude::*;

use crate::components::tile_size_slider::TileSizeSlider;

#[derive(PartialEq, Props, Clone)]
pub struct BottomBarProps {
    pub status: String,
    pub detail: Option<String>,
    pub tile_size: u32,
    pub on_tile_size: EventHandler<u32>,
}

#[component]
pub fn BottomBar(props: BottomBarProps) -> Element {
    rsx! {
        style { { include_str!("./bottom_bar.css") } }
        div { class: "BottomBar",
            span { class: "bottom_bar_status", "{props.status}" }
            if let Some(detail) = props.detail.as_ref() {
                span { class: "mono bottom_bar_detail selectable", title: "{detail}", "{detail}" }
            }
            TileSizeSlider { value: props.tile_size, on_change: props.on_tile_size }
        }
    }
}
