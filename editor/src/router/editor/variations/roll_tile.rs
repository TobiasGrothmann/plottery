use dioxus::prelude::*;

use crate::components::image::Image;

/// One roll in the variations grid. A click makes its params the project's.
#[component]
pub fn RollTile(
    index: usize,
    /// Its PNG once the roll is done.
    png: Option<String>,
    values: Vec<(String, String)>,
    /// Changes with every new set of rolls, so a reused file name loads its new image.
    set_id: u32,
    on_pick: EventHandler<usize>,
) -> Element {
    let name = match index {
        0 => "start".to_string(),
        _ => index.to_string(),
    };
    let title = values
        .iter()
        .map(|(path, value)| format!("{path} = {value}"))
        .chain(std::iter::once("Click to edit these params".to_string()))
        .collect::<Vec<_>>()
        .join("\n");

    let class = match png {
        Some(_) => "card roll",
        None => "card roll pending",
    };

    rsx! {
        div {
            class,
            role: "button",
            tabindex: "0",
            title: "{title}",
            onclick: move |_| on_pick.call(index),
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    on_pick.call(index);
                }
            },
            div { class: "thumb",
                if let Some(img_path) = png {
                    Image { img_path, redraw_counter: set_id, class: "thumb_paper" }
                } else {
                    span { class: "thumb_empty", "rolling…" }
                }
            }
            span { class: "roll_number", "{name}" }
        }
    }
}
