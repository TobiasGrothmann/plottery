use std::rc::Rc;

use dioxus::{html::input_data::MouseButton, prelude::*};

use super::slider_input::{ALT_SPEED_FACTOR, JUMP_DISTANCE_PX, THUMB_WIDTH_PX};

#[derive(PartialEq, Props, Clone)]
pub struct RangeSliderInputProps {
    pub low: f32,
    pub high: f32,
    pub min: f32,
    pub max: f32,
    #[props(default)]
    pub integer: bool,
    /// `(low, high)` on release.
    pub oncommit: EventHandler<(f32, f32)>,
    /// `(low, high)` on every drag move, before the commit.
    #[props(default)]
    pub oninput: Option<EventHandler<(f32, f32)>>,
}

#[derive(Clone, Copy, PartialEq)]
enum Thumb {
    Low,
    High,
}

#[derive(Clone, Copy)]
struct DragState {
    thumb: Thumb,
    last_x: f64,
    /// Pixels a thumb travels over the full value range.
    px_range: f64,
}

fn snap(value: f32, props: &RangeSliderInputProps) -> f32 {
    let value = value.clamp(props.min, props.max);
    if props.integer {
        value.round()
    } else {
        value
    }
}

fn fraction(value: f32, props: &RangeSliderInputProps) -> f32 {
    if props.max > props.min {
        ((value - props.min) / (props.max - props.min)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Moves one thumb to `value`, stopping at the other one.
fn with_thumb(values: (f32, f32), thumb: Thumb, value: f32) -> (f32, f32) {
    match thumb {
        Thumb::Low => (value.min(values.1), values.1),
        Thumb::High => (values.0, value.max(values.0)),
    }
}

/// Two-thumb slider for a min and max: the nearer thumb drags (alt slows it down), commits on
/// release. Pressing away from both thumbs jumps the nearer one to the mouse first.
#[component]
pub fn RangeSliderInput(props: RangeSliderInputProps) -> Element {
    let mut values = use_signal(|| (props.low, props.high));
    let mut drag = use_signal(|| None::<DragState>);
    let mut track = use_signal(|| None::<Rc<MountedData>>);

    // follow external changes while not dragging
    let mut last_props = use_signal(|| (props.low, props.high));
    if last_props() != (props.low, props.high) {
        last_props.set((props.low, props.high));
        if drag.peek().is_none() {
            values.set((props.low, props.high));
        }
    }

    let (low, high) = values();
    let low_fraction = fraction(snap(low, &props), &props);
    let high_fraction = fraction(snap(high, &props), &props);

    let (min, max) = (props.min, props.max);
    let props_for_commit = props.clone();
    let props_for_input = props.clone();
    let props_for_press = props.clone();
    let snapped = move |props: &RangeSliderInputProps| {
        let (low, high) = values();
        (snap(low, props), snap(high, props))
    };
    let commit = use_callback(move |_: ()| {
        props_for_commit.oncommit.call(snapped(&props_for_commit));
    });
    let mut end_drag = move || {
        if drag().is_some() {
            drag.set(None);
            commit.call(());
        }
    };

    rsx! {
        style { {include_str!("slider_input.css")} }
        style { {include_str!("range_slider_input.css")} }
        div { class: "RangeSliderInput",
            div {
                class: "slider_track",
                title: "{min} – {max}",
                onmounted: move |event| track.set(Some(event.data())),
                onmousedown: move |event| {
                    if event.trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }
                    event.prevent_default();
                    let Some(track) = track() else { return };
                    let last_x = event.client_coordinates().x;
                    let props = props_for_press.clone();
                    spawn(async move {
                        let Ok(rect) = track.get_client_rect().await else { return };
                        let px_range = (rect.size.width - THUMB_WIDTH_PX).max(1.0);
                        let track_start = rect.origin.x + THUMB_WIDTH_PX / 2.0;
                        let center = |f: f32| track_start + f as f64 * px_range;
                        let (low_center, high_center) = (center(low_fraction), center(high_fraction));
                        let (low_dist, high_dist) = ((last_x - low_center).abs(), (last_x - high_center).abs());
                        // stacked thumbs: the side of the press decides
                        let thumb = if low_dist < high_dist || (low_dist == high_dist && last_x < low_center) {
                            Thumb::Low
                        } else {
                            Thumb::High
                        };
                        if low_dist.min(high_dist) > JUMP_DISTANCE_PX {
                            let f = ((last_x - track_start) / px_range).clamp(0.0, 1.0);
                            values.set(with_thumb(values(), thumb, min + f as f32 * (max - min)));
                            if let Some(oninput) = props.oninput {
                                oninput.call(snapped(&props));
                            }
                        }
                        drag.set(Some(DragState { thumb, last_x, px_range }));
                    });
                },
                div {
                    class: "range_fill",
                    style: "left: calc({low_fraction} * (100% - {THUMB_WIDTH_PX}px) + {THUMB_WIDTH_PX / 2.0}px); width: calc({high_fraction - low_fraction} * (100% - {THUMB_WIDTH_PX}px));",
                }
                div {
                    class: "slider_thumb",
                    style: "left: calc({low_fraction} * (100% - {THUMB_WIDTH_PX}px));",
                }
                div {
                    class: "slider_thumb",
                    style: "left: calc({high_fraction} * (100% - {THUMB_WIDTH_PX}px));",
                }
            }
        }
        if let Some(state) = drag() {
            div {
                class: "slider_drag_overlay",
                onmousemove: move |event| {
                    if !event.held_buttons().contains(MouseButton::Primary) {
                        end_drag();
                        return;
                    }
                    let x = event.client_coordinates().x;
                    let speed = if event.modifiers().alt() { ALT_SPEED_FACTOR } else { 1.0 };
                    let delta = ((x - state.last_x) * speed / state.px_range * (max - min) as f64) as f32;
                    let current = values();
                    let moved = match state.thumb {
                        Thumb::Low => current.0 + delta,
                        Thumb::High => current.1 + delta,
                    };
                    values.set(with_thumb(current, state.thumb, moved.clamp(min, max)));
                    if let Some(oninput) = props_for_input.oninput {
                        oninput.call(snapped(&props_for_input));
                    }
                    drag.set(Some(DragState { last_x: x, ..state }));
                },
                onmouseup: move |_| end_drag(),
            }
        }
    }
}
