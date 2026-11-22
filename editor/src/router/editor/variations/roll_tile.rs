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
        .map(|(path, value)| format!("{}: {}", path.replace('_', " "), neat_value(value)))
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

/// A value's text for reading: decimals cut to two places, degrees as °.
pub(super) fn neat_value(text: &str) -> String {
    let mut neat = String::new();
    let mut number = String::new();
    let flush = |number: &mut String, neat: &mut String| {
        match number
            .contains('.')
            .then(|| number.parse::<f32>().ok())
            .flatten()
        {
            Some(value) => {
                let short = format!("{value:.2}");
                neat.push_str(short.trim_end_matches('0').trim_end_matches('.'));
            }
            None => neat.push_str(number),
        }
        number.clear();
    };
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        // a dot belongs to the number when a digit follows, `..` of a range doesn't
        let in_number = c.is_ascii_digit()
            || (c == '.'
                && chars.get(i + 1).is_some_and(char::is_ascii_digit)
                && number.chars().any(|c| c.is_ascii_digit())
                && !number.contains('.'))
            || (c == '-'
                && number.is_empty()
                && chars.get(i + 1).is_some_and(char::is_ascii_digit));
        if in_number {
            number.push(c);
        } else {
            flush(&mut number, &mut neat);
            neat.push(c);
        }
    }
    flush(&mut number, &mut neat);
    neat.replace("deg", "°")
}
