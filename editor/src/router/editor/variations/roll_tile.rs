use dioxus::prelude::*;

use crate::components::image::Image;

/// One roll in the variations grid. A click selects it, a double click or Enter opens it.
#[component]
pub fn RollTile(
    index: usize,
    png: String,
    /// Its params are the project's current ones.
    current: bool,
    /// It ran too long and has no drawing.
    skipped: bool,
    /// Changes with every new set of rolls, so a reused file name loads its new image.
    set_id: u32,
    on_select: EventHandler<usize>,
    on_open: EventHandler<usize>,
) -> Element {
    let name = match index {
        0 => "start".to_string(),
        _ => index.to_string(),
    };
    let class = match (skipped, current) {
        (true, _) => "card roll skipped",
        (false, true) => "card roll current",
        (false, false) => "card roll",
    };

    rsx! {
        div {
            id: "roll-tile-{index}",
            class,
            role: "button",
            tabindex: "0",
            onclick: move |_| {
                if !skipped {
                    on_select.call(index);
                }
            },
            ondoubleclick: move |_| {
                if !skipped {
                    on_open.call(index);
                }
            },
            onkeydown: move |event| {
                if !skipped && event.key() == Key::Enter {
                    on_open.call(index);
                }
            },
            div { class: "thumb",
                if skipped {
                    span { class: "roll_skipped", "ran too long" }
                } else {
                    Image { img_path: png, redraw_counter: set_id, class: "thumb_paper" }
                }
            }
            span { class: "roll_number", "{name}" }
        }
    }
}

/// The current drawing, when no roll is it: what the next roll starts from. A double click or
/// Enter opens it.
#[component]
pub fn CurrentTile(
    /// Url of the drawing, `None` before the first run.
    src: Option<String>,
    on_open: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "card roll current",
            role: "button",
            tabindex: "0",
            ondoubleclick: move |_| on_open.call(()),
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    on_open.call(());
                }
            },
            div { class: "thumb",
                match src {
                    Some(src) => rsx! { img { class: "thumb_paper", src: "{src}", draggable: "false" } },
                    None => rsx! { span { class: "roll_skipped", "not run yet" } },
                }
            }
            span { class: "roll_number", "current" }
        }
    }
}
