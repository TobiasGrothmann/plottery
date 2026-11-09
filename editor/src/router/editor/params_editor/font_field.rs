use std::{rc::Rc, sync::Arc};

use dioxus::prelude::*;
use plottery_lib::BundledFont;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    font_preview::{cached_preview, font_label, preview, FontPreview},
    params_editor::LeafProps,
};

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

/// Matches labels ignoring case and spaces, so `hershey sans` finds `HersheySansMedium`.
fn matches_query(font: BundledFont, query: &str) -> bool {
    let query: String = query.split_whitespace().collect::<String>().to_lowercase();
    font.name().to_lowercase().contains(&query)
}

#[component]
fn FontPreviewSvg(preview: Arc<FontPreview>, height: f32) -> Element {
    let width = height * preview.width / preview.height;
    rsx! {
        svg {
            class: "font-preview",
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {preview.width} {preview.height}",
            preserve_aspect_ratio: "xMinYMid meet",
            path { d: "{preview.d}" }
        }
    }
}

#[component]
pub fn FontField(props: LeafProps) -> Element {
    let ProjectParamValue::Font(name) = &props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let selected = BundledFont::from_name(name);
    let selected_name = name.clone();

    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut placement = use_signal(|| None::<Placement>);
    let mut query = use_signal(String::new);
    let mut highlight = use_signal(|| None::<BundledFont>);

    // Rendering all previews takes ~0.1 s in release and ~0.7 s in debug, so warm the cache off
    // the UI thread; an open popover re-renders as they arrive.
    let mut warmed = use_signal(|| 0usize);
    use_hook(move || {
        spawn(async move {
            for font in BundledFont::ALL.iter().copied() {
                if cached_preview(font).is_none() {
                    let _ = tokio::task::spawn_blocking(move || preview(font)).await;
                    if placement.peek().is_some() {
                        *warmed.write() += 1;
                    }
                }
            }
        });
    });
    let _ = warmed();

    let filtered: Vec<BundledFont> = BundledFont::ALL
        .iter()
        .copied()
        .filter(|font| matches_query(*font, &query.read()))
        .collect();
    let active = highlight()
        .filter(|font| filtered.contains(font))
        .or_else(|| filtered.first().copied());

    let open_popover = move |_| async move {
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
            max_height: POPOVER_MAX_HEIGHT.min(if downwards { room_below } else { room_above }),
        }));
    };

    let pick = {
        let props = props.clone();
        use_callback(move |font: BundledFont| {
            placement.set(None);
            if Some(font) != selected {
                props.edit(|value| *value = ProjectParamValue::Font(font.name().to_string()));
            }
        })
    };

    let on_search_key = {
        let filtered = filtered.clone();
        move |event: KeyboardEvent| {
            let position = active.and_then(|font| filtered.iter().position(|f| *f == font));
            let step = match event.key() {
                Key::ArrowDown => 1,
                Key::ArrowUp => -1,
                Key::Enter => {
                    event.prevent_default();
                    if let Some(font) = active {
                        pick.call(font);
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
            if filtered.is_empty() {
                return;
            }
            let last = filtered.len() as i32 - 1;
            let next = position.map_or(0, |i| (i as i32 + step).clamp(0, last));
            highlight.set(Some(filtered[next as usize]));
            document::eval(SCROLL_ACTIVE_INTO_VIEW);
        }
    };

    rsx! {
        div { class: "FontField",
            button {
                class: "input select font-trigger",
                title: "{selected_name}",
                onmounted: move |event| trigger.set(Some(event.data())),
                onclick: open_popover,
                if let Some(font) = selected {
                    FontPreviewSvg { preview: preview(font), height: 16.0 }
                } else {
                    span { class: "hint", "unknown font '{selected_name}'" }
                }
            }
            if let Some(open_at) = placement() {
                div {
                    class: "font-backdrop",
                    onclick: move |_| placement.set(None),
                }
                div { class: "font-popover", style: open_at.style(),
                    input {
                        class: "input font-search",
                        placeholder: "Search fonts",
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
                        for font in filtered.iter().copied() {
                            button {
                                key: "{font.name()}",
                                class: "font-row",
                                class: if Some(font) == active { "active" },
                                class: if Some(font) == selected { "selected" },
                                tabindex: "-1",
                                // mousemove, not mouseenter: rows scrolling under a still cursor must not steal the highlight
                                onmousemove: move |_| {
                                    if highlight() != Some(font) {
                                        highlight.set(Some(font));
                                    }
                                },
                                onclick: move |_| pick.call(font),
                                if let Some(preview) = cached_preview(font) {
                                    FontPreviewSvg { preview, height: 22.0 }
                                } else {
                                    div { class: "font-pending" }
                                }
                                span { class: "font-name", "{font_label(font)}" }
                            }
                        }
                        if filtered.is_empty() {
                            div { class: "font-empty hint", "No font matches" }
                        }
                    }
                }
            }
        }
    }
}
