use dioxus::prelude::*;

use crate::model::app_state::{TILE_SIZE_MAX, TILE_SIZE_MIN};

/// Slider for the size of grid tiles, between a small and a large square.
#[component]
pub fn TileSizeSlider(
    value: u32,
    #[props(default = TILE_SIZE_MAX)] max: u32,
    on_change: EventHandler<u32>,
) -> Element {
    rsx! {
        style { { include_str!("./tile_size_slider.css") } }
        div { class: "tile_size_control",
            span { class: "hint", "Tile size" }
            span { class: "tile_size_glyph tile_size_glyph_small" }
            input {
                r#type: "range",
                class: "range",
                "aria-label": "Tile size",
                min: "{TILE_SIZE_MIN}",
                max: "{max}",
                step: "10",
                value: "{value}",
                oninput: move |event| {
                    if let Ok(size) = event.value().parse::<u32>() {
                        on_change.call(size);
                    }
                },
            }
            span { class: "tile_size_glyph tile_size_glyph_large" }
        }
    }
}
