use std::rc::Rc;

use dioxus::prelude::*;

const POPOVER_MAX_HEIGHT: f64 = 360.0;
const POPOVER_MIN_WIDTH: f64 = 260.0;
const POPOVER_GAP: f64 = 4.0;
/// Opens upwards when there is less room than this below the field.
const MIN_ROOM_BELOW: f64 = 220.0;

const SCROLL_ACTIVE_INTO_VIEW: &str = r#"
requestAnimationFrame(() => document
    .querySelector('.font-popover .font-row.active')
    ?.scrollIntoView({ block: 'nearest' }));
"#;

/// Fixed position of the open popover, from the trigger's client rect.
#[derive(Clone, Copy, PartialEq)]
struct Placement {
    left: f64,
    width: f64,
    /// `top` when opening downwards, else the CSS `bottom` distance
    edge: f64,
    downwards: bool,
    max_height: f64,
}

impl Placement {
    fn style(&self) -> String {
        let Placement {
            left,
            width,
            edge,
            downwards,
            max_height,
        } = *self;
        let vertical = if downwards { "top" } else { "bottom" };
        format!(
            "left: {left}px; width: {width}px; {vertical}: {edge}px; max-height: {max_height}px;"
        )
    }
}

/// One option of a [`SearchPicker`]: the search and `onpick` use `key`, the list shows `content`.
#[derive(Clone, PartialEq)]
pub struct PickerRow {
    pub key: String,
    pub content: Element,
}

/// Matches ignoring case and spaces, so `hershey sans` finds `HersheySansMedium`.
fn matches_query(key: &str, query: &str) -> bool {
    let compact = |text: &str| text.split_whitespace().collect::<String>().to_lowercase();
    compact(key).contains(&compact(query))
}

#[derive(PartialEq, Props, Clone)]
pub struct SearchPickerProps {
    pub rows: Vec<PickerRow>,
    /// Key of the current value, if it is one of `rows`.
    pub selected: Option<String>,
    pub placeholder: String,
    /// Shown instead of the list when no row matches.
    pub empty_text: String,
    /// Tooltip of the trigger.
    pub title: String,
    pub onpick: EventHandler<String>,
    /// Content of the button that opens the popover.
    pub children: Element,
}

/// A button that opens a searchable list below it, with keyboard navigation.
#[component]
pub fn SearchPicker(props: SearchPickerProps) -> Element {
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut placement = use_signal(|| None::<Placement>);
    let mut query = use_signal(String::new);
    let mut highlight = use_signal(|| None::<String>);

    let filtered: Vec<&PickerRow> = props
        .rows
        .iter()
        .filter(|row| matches_query(&row.key, &query.read()))
        .collect();
    let keys: Vec<String> = filtered.iter().map(|row| row.key.clone()).collect();
    let active = highlight()
        .filter(|key| keys.contains(key))
        .or_else(|| keys.first().cloned());

    let open_popover = {
        let selected = props.selected.clone();
        move |_| {
            let selected = selected.clone();
            async move {
                let Some(trigger) = trigger() else { return };
                let Ok(rect) = trigger.get_client_rect().await else {
                    return;
                };
                let viewport_height = document::eval("return window.innerHeight;")
                    .await
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(800.0);
                let room_below = viewport_height - rect.max_y() - 2.0 * POPOVER_GAP;
                let room_above = rect.min_y() - 2.0 * POPOVER_GAP;
                let downwards = room_below >= MIN_ROOM_BELOW || room_below >= room_above;
                query.set(String::new());
                highlight.set(selected);
                placement.set(Some(Placement {
                    left: rect.min_x(),
                    width: rect.width().max(POPOVER_MIN_WIDTH),
                    edge: if downwards {
                        rect.max_y() + POPOVER_GAP
                    } else {
                        viewport_height - rect.min_y() + POPOVER_GAP
                    },
                    downwards,
                    max_height: POPOVER_MAX_HEIGHT.min(if downwards {
                        room_below
                    } else {
                        room_above
                    }),
                }));
            }
        }
    };

    let onpick = props.onpick;
    let pick = use_callback(move |key: String| {
        placement.set(None);
        onpick.call(key);
    });

    let on_search_key = {
        let active = active.clone();
        move |event: KeyboardEvent| {
            let position = active
                .as_ref()
                .and_then(|active| keys.iter().position(|key| key == active));
            let step = match event.key() {
                Key::ArrowDown => 1,
                Key::ArrowUp => -1,
                Key::Enter => {
                    event.prevent_default();
                    if let Some(key) = active.clone() {
                        pick.call(key);
                    }
                    return;
                }
                Key::Escape => {
                    event.prevent_default();
                    event.stop_propagation();
                    placement.set(None);
                    return;
                }
                _ => return,
            };
            event.prevent_default();
            if keys.is_empty() {
                return;
            }
            let last = keys.len() as i32 - 1;
            let next = position.map_or(0, |i| (i as i32 + step).clamp(0, last));
            highlight.set(Some(keys[next as usize].clone()));
            document::eval(SCROLL_ACTIVE_INTO_VIEW);
        }
    };

    rsx! {
        div { class: "FontField",
            button {
                class: "input select font-trigger",
                title: "{props.title}",
                onmounted: move |event| trigger.set(Some(event.data())),
                onclick: open_popover,
                {props.children}
            }
            if let Some(open_at) = placement() {
                div {
                    class: "font-backdrop",
                    onclick: move |_| placement.set(None),
                }
                div { class: "font-popover", style: open_at.style(),
                    input {
                        class: "input font-search",
                        placeholder: "{props.placeholder}",
                        value: "{query}",
                        onmounted: move |event| async move {
                            let _ = event.data().set_focus(true).await;
                            document::eval(SCROLL_ACTIVE_INTO_VIEW);
                        },
                        oninput: move |event| {
                            query.set(event.value());
                            highlight.set(None);
                        },
                        onkeydown: on_search_key,
                    }
                    div { class: "font-list scroll",
                        for row in filtered.iter() {
                            button {
                                key: "{row.key}",
                                class: "font-row",
                                class: if active.as_ref() == Some(&row.key) { "active" },
                                class: if props.selected.as_ref() == Some(&row.key) { "selected" },
                                tabindex: "-1",
                                // mousemove, not mouseenter: rows scrolling under a still cursor must not steal the highlight
                                onmousemove: {
                                    let key = row.key.clone();
                                    move |_| {
                                        if highlight.peek().as_ref() != Some(&key) {
                                            highlight.set(Some(key.clone()));
                                        }
                                    }
                                },
                                onclick: {
                                    let key = row.key.clone();
                                    move |_| pick.call(key.clone())
                                },
                                {row.content.clone()}
                            }
                        }
                        if filtered.is_empty() {
                            div { class: "font-empty hint", "{props.empty_text}" }
                        }
                    }
                }
            }
        }
    }
}
