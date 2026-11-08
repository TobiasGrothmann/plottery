use std::rc::Rc;

use dioxus::{html::input_data::MouseButton, prelude::*};

use super::format_number;

/// Must match `.slider_thumb` width in slider_input.css.
const THUMB_WIDTH_PX: f64 = 12.0;
pub const ALT_SPEED_FACTOR: f64 = 0.25;
/// Presses farther than this from the thumb centre jump the thumb to the mouse.
const JUMP_DISTANCE_PX: f64 = 20.0;

#[derive(PartialEq, Props, Clone)]
pub struct SliderInputProps {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    #[props(default)]
    pub integer: bool,
    /// Shown instead of the number, e.g. a percentage.
    #[props(default)]
    pub label: Option<String>,
    pub oncommit: EventHandler<f32>,
    /// Called on every drag move, before the commit on release.
    #[props(default)]
    pub oninput: Option<EventHandler<f32>>,
}

#[derive(Clone, Copy)]
struct DragState {
    last_x: f64,
    /// Pixels the thumb travels over the full value range.
    px_range: f64,
}

fn snap(value: f32, props: &SliderInputProps) -> f32 {
    let value = value.clamp(props.min, props.max);
    if props.integer {
        value.round()
    } else {
        value
    }
}

/// Drag slider with relative movement (alt slows it down), commits on release.
/// Pressing away from the thumb jumps it to the mouse first.
#[component]
pub fn SliderInput(props: SliderInputProps) -> Element {
    let mut slider_value = use_signal(|| props.value);
    let mut drag = use_signal(|| None::<DragState>);
    let mut track = use_signal(|| None::<Rc<MountedData>>);

    // follow external changes while not dragging
    let mut last_prop_value = use_signal(|| props.value);
    if last_prop_value() != props.value {
        last_prop_value.set(props.value);
        if drag.peek().is_none() {
            slider_value.set(props.value);
        }
    }

    let shown_value = snap(slider_value(), &props);
    let shown_text = props
        .label
        .clone()
        .unwrap_or_else(|| format_number(shown_value));
    let thumb_fraction = if props.max > props.min {
        ((shown_value - props.min) / (props.max - props.min)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let (min, max) = (props.min, props.max);
    let oninput = props.oninput;
    let props_for_commit = props.clone();
    let props_for_input = props.clone();
    let props_for_jump = props.clone();
    let commit = use_callback(move |_: ()| {
        props_for_commit
            .oncommit
            .call(snap(slider_value(), &props_for_commit));
    });
    let mut end_drag = move || {
        if drag().is_some() {
            drag.set(None);
            commit.call(());
        }
    };

    rsx! {
        style { {include_str!("slider_input.css")} }
        div { class: "SliderInput",
            div {
                class: "slider_track",
                onmounted: move |event| track.set(Some(event.data())),
                onmousedown: move |event| {
                    if event.trigger_button() != Some(MouseButton::Primary) {
                        return;
                    }
                    event.prevent_default();
                    let Some(track) = track() else { return };
                    let last_x = event.client_coordinates().x;
                    let props = props_for_jump.clone();
                    spawn(async move {
                        if let Ok(rect) = track.get_client_rect().await {
                            let px_range = (rect.size.width - THUMB_WIDTH_PX).max(1.0);
                            let track_start = rect.origin.x + THUMB_WIDTH_PX / 2.0;
                            let thumb_center = track_start + thumb_fraction as f64 * px_range;
                            if (last_x - thumb_center).abs() > JUMP_DISTANCE_PX {
                                let fraction = ((last_x - track_start) / px_range).clamp(0.0, 1.0);
                                let value = min + fraction as f32 * (max - min);
                                slider_value.set(value);
                                if let Some(oninput) = props.oninput {
                                    oninput.call(snap(value, &props));
                                }
                            }
                            drag.set(Some(DragState { last_x, px_range }));
                        }
                    });
                },
                div {
                    class: "slider_thumb",
                    style: "left: calc({thumb_fraction} * (100% - {THUMB_WIDTH_PX}px));",
                }
            }
            span { class: "mono slider_value", "{shown_text}" }
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
                    let delta = (x - state.last_x) * speed / state.px_range * (max - min) as f64;
                    let value = (slider_value() + delta as f32).clamp(min, max);
                    slider_value.set(value);
                    if let Some(oninput) = oninput {
                        oninput.call(snap(value, &props_for_input));
                    }
                    drag.set(Some(DragState { last_x: x, ..state }));
                },
                onmouseup: move |_| end_drag(),
            }
        }
    }
}
