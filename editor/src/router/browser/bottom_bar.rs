use dioxus::prelude::*;

use crate::model::app_state::{TILE_SIZE_MAX, TILE_SIZE_MIN};

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
            p { class: "bottom_bar_status", "{props.status}" }
            if let Some(detail) = props.detail.as_ref() {
                p { class: "bottom_bar_detail", title: "{detail}", "{detail}" }
            }
            div { class: "tile_size_control", title: "Tile size",
                span { class: "tile_size_glyph tile_size_glyph_small" }
                input {
                    r#type: "range",
                    class: "tile_size_slider",
                    min: "{TILE_SIZE_MIN}",
                    max: "{TILE_SIZE_MAX}",
                    step: "10",
                    value: "{props.tile_size}",
                    oninput: move |event| {
                        if let Ok(size) = event.value().parse::<u32>() {
                            props.on_tile_size.call(size);
                        }
                    },
                }
                span { class: "tile_size_glyph tile_size_glyph_large" }
            }
        }
    }
}
