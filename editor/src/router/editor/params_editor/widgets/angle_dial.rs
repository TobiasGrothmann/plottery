use std::rc::Rc;

use dioxus::{
    html::{geometry::ClientPoint, input_data::MouseButton},
    prelude::*,
};

use super::slider_input::{ALT_SPEED_FACTOR, JUMP_DISTANCE_PX};

/// Must match `.AngleDial` in angle_dial.css.
const SIZE: f32 = 60.0;
const RIM_RADIUS: f32 = 29.5;
/// Distance of the handle from the center.
const HAND_LENGTH: f32 = 22.0;
const HANDLE_RADIUS: f32 = 4.0;
/// The cursor has no direction this close to the center.
const DEAD_ZONE_PX: f64 = 4.0;
const STEP_DEGREES: f64 = 1.0;
const SHIFT_STEP_DEGREES: f64 = 15.0;

#[derive(PartialEq, Props, Clone)]
pub struct AngleDialProps {
    pub rad: f32,
    /// Called with radians on every drag move, before the commit on release.
    pub oninput: EventHandler<f32>,
    pub oncommit: EventHandler<f32>,
}

#[derive(Clone, Copy, PartialEq)]
struct DragState {
    center: ClientPoint,
    /// Direction of the cursor at the last move, `None` while it had none.
    last_cursor: Option<f64>,
    /// Where the handle is, before rounding to the step.
    degrees: f64,
    /// The last reported angle, `None` until the cursor gave a direction.
    rad: Option<f32>,
}

/// Degrees in 0..360 of `cursor` seen from `center`, y pointing up. `None` close to the center.
pub fn cursor_degrees(center: ClientPoint, cursor: ClientPoint) -> Option<f64> {
    let (dx, dy) = (cursor.x - center.x, center.y - cursor.y);
    (dx.hypot(dy) >= DEAD_ZONE_PX).then(|| dy.atan2(dx).to_degrees().rem_euclid(360.0))
}

/// `degrees` after the cursor turned from `from` to `to` the short way, scaled by `speed`.
pub fn turned(degrees: f64, from: f64, to: f64, speed: f64) -> f64 {
    let delta = (to - from + 180.0).rem_euclid(360.0) - 180.0;
    (degrees + delta * speed).rem_euclid(360.0)
}

/// Radians in 0..2π of `degrees` rounded to `step_degrees`.
pub fn stepped_rad(degrees: f64, step_degrees: f64) -> f32 {
    // rounding can reach 360, which is 0 again
    let stepped = ((degrees / step_degrees).round() * step_degrees) % 360.0;
    (stepped as f32).to_radians()
}

fn step_degrees(shift: bool) -> f64 {
    if shift {
        SHIFT_STEP_DEGREES
    } else {
        STEP_DEGREES
    }
}

/// Dial in whole degrees with relative movement (alt slows it down, shift snaps to 15°),
/// commits on release. Pressing away from the handle jumps it to the mouse first.
/// 0 points right and angles grow counter-clockwise, like in a drawing.
#[component]
pub fn AngleDial(props: AngleDialProps) -> Element {
    let mut dial = use_signal(|| None::<Rc<MountedData>>);
    let mut drag = use_signal(|| None::<DragState>);

    let (oninput, oncommit) = (props.oninput, props.oncommit);
    let mut end_drag = move || {
        if let Some(rad) = drag.take().and_then(|state| state.rad) {
            oncommit.call(rad);
        }
    };

    let prop_rad = props.rad;
    let shown_rad = drag().and_then(|state| state.rad).unwrap_or(prop_rad);
    let center = SIZE / 2.0;
    let (sin, cos) = shown_rad.sin_cos();
    let handle_x = center + cos * HAND_LENGTH;
    let handle_y = center - sin * HAND_LENGTH;
    let zero_x = center + HAND_LENGTH;

    rsx! {
        style { {include_str!("angle_dial.css")} }
        div {
            class: "AngleDial",
            onmounted: move |event| dial.set(Some(event.data())),
            onmousedown: move |event| {
                if event.trigger_button() != Some(MouseButton::Primary) {
                    return;
                }
                event.prevent_default();
                let Some(dial) = dial() else { return };
                let cursor = event.client_coordinates();
                let step = step_degrees(event.modifiers().shift());
                spawn(async move {
                    if let Ok(rect) = dial.get_client_rect().await {
                        let center = ClientPoint::new(
                            rect.origin.x + rect.size.width / 2.0,
                            rect.origin.y + rect.size.height / 2.0,
                        );
                        let (sin, cos) = (prop_rad as f64).sin_cos();
                        let handle = ClientPoint::new(
                            center.x + cos * HAND_LENGTH as f64,
                            center.y - sin * HAND_LENGTH as f64,
                        );
                        let last_cursor = cursor_degrees(center, cursor);
                        let (degrees, rad) = match last_cursor {
                            Some(degrees) if cursor.distance_to(handle) > JUMP_DISTANCE_PX => {
                                (degrees, Some(stepped_rad(degrees, step)))
                            }
                            _ => ((prop_rad as f64).to_degrees().rem_euclid(360.0), None),
                        };
                        drag.set(Some(DragState { center, last_cursor, degrees, rad }));
                        if let Some(rad) = rad {
                            oninput.call(rad);
                        }
                    }
                });
            },
            svg {
                width: "{SIZE}",
                height: "{SIZE}",
                view_box: "0 0 {SIZE} {SIZE}",
                circle {
                    class: "angle_dial_rim",
                    cx: "{center}",
                    cy: "{center}",
                    r: "{RIM_RADIUS}",
                }
                line {
                    class: "angle_dial_zero",
                    x1: "{center}",
                    y1: "{center}",
                    x2: "{zero_x}",
                    y2: "{center}",
                }
                line {
                    class: "angle_dial_hand",
                    x1: "{center}",
                    y1: "{center}",
                    x2: "{handle_x:.2}",
                    y2: "{handle_y:.2}",
                }
                circle {
                    class: "angle_dial_handle",
                    cx: "{handle_x:.2}",
                    cy: "{handle_y:.2}",
                    r: "{HANDLE_RADIUS}",
                }
            }
        }
        if let Some(state) = drag() {
            div {
                class: "angle_dial_drag_overlay",
                onmousemove: move |event| {
                    if !event.held_buttons().contains(MouseButton::Primary) {
                        end_drag();
                        return;
                    }
                    let Some(cursor) = cursor_degrees(state.center, event.client_coordinates())
                    else {
                        return;
                    };
                    let speed = if event.modifiers().alt() { ALT_SPEED_FACTOR } else { 1.0 };
                    let degrees = match state.last_cursor {
                        Some(last) => turned(state.degrees, last, cursor, speed),
                        None => cursor,
                    };
                    let rad = stepped_rad(degrees, step_degrees(event.modifiers().shift()));
                    if state.rad != Some(rad) {
                        oninput.call(rad);
                    }
                    drag.set(Some(DragState {
                        last_cursor: Some(cursor),
                        degrees,
                        rad: Some(rad),
                        ..state
                    }));
                },
                onmouseup: move |_| end_drag(),
            }
        }
    }
}
