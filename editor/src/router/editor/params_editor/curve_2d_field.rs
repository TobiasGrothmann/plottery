use dioxus::{html::input_data::MouseButton, prelude::*};
use plottery_lib::V2;
use plottery_runtime::{project_param_value::ProjectParamValue, Curve2D};

use crate::router::editor::{
    editor_ctx::EditorCtx,
    params_editor::{
        params_editor::{edit_leaf_at, LeafProps},
        widgets::{number_input::NumberInput, slider_input::ALT_SPEED_FACTOR},
    },
};

// Margin is constant, dimensions are detected from CSS
const MARGIN: f64 = 8.0;
/// Pixels the mouse has to move before a press on a handle becomes a drag.
const DRAG_THRESHOLD_PX: f64 = 2.0;
/// Grid the handles snap to while shift is held.
const SNAP_STEPS: f32 = 10.0;

const CANVAS_WIDTH: f64 = 128.0;
const CANVAS_HEIGHT: f64 = 128.0;
const GRID_STEP: f64 = 32.0;
const GRAPH_WIDTH: f64 = CANVAS_WIDTH - 2.0 * MARGIN;
const GRAPH_HEIGHT: f64 = CANVAS_HEIGHT - 2.0 * MARGIN;
const GRAPH_BOTTOM: f64 = CANVAS_HEIGHT - MARGIN;

#[derive(Clone, Copy)]
struct CurveDrag {
    index: usize,
    last: (f64, f64),
    /// Unsnapped normalized position, so snapping doesn't accumulate.
    pos: V2,
    active: bool,
}

/// Moves point `index` of the curve in `value`; the endpoints only move vertically.
fn move_curve_point(value: &mut ProjectParamValue, index: usize, point: V2) {
    match value {
        ProjectParamValue::Curve2DNorm(g) => {
            if index == 0 {
                g.update_endpoint(true, point.y);
            } else if index == g.len() - 1 {
                g.update_endpoint(false, point.y);
            } else {
                let _ = g.update_point_norm(index - 1, point);
            }
        }
        ProjectParamValue::Curve2D(c) => {
            if index == 0 {
                c.update_endpoint_norm(true, point.y);
            } else if index == c.len() - 1 {
                c.update_endpoint_norm(false, point.y);
            } else {
                let _ = c.update_point_norm(index - 1, point);
            }
        }
        _ => {}
    }
}

fn add_curve_point(value: &mut ProjectParamValue, point: V2) {
    match value {
        ProjectParamValue::Curve2DNorm(c) => {
            let _ = c.add_point(point);
        }
        ProjectParamValue::Curve2D(c) => {
            let _ = c.add_point_norm(point);
        }
        _ => {}
    }
}

fn remove_curve_point(value: &mut ProjectParamValue, index: usize) {
    match value {
        ProjectParamValue::Curve2DNorm(c) => {
            let _ = c.remove_point_at(index);
        }
        ProjectParamValue::Curve2D(c) => {
            let _ = c.remove_point_at(index);
        }
        _ => {}
    }
}

/// One of the four numbers a `Curve2D` maps its 0..1 curve to.
#[derive(Clone, Copy, PartialEq)]
enum Domain {
    XStart,
    XEnd,
    YStart,
    YEnd,
}

impl Domain {
    fn get(self, curve: &Curve2D) -> f32 {
        let mapped = &curve.mapped_to;
        match self {
            Domain::XStart => mapped.x_start,
            Domain::XEnd => mapped.x_end,
            Domain::YStart => mapped.y_start,
            Domain::YEnd => mapped.y_end,
        }
    }

    fn set(self, value: &mut ProjectParamValue, new: f32) {
        let ProjectParamValue::Curve2D(curve) = value else {
            return;
        };
        let mapped = &mut curve.mapped_to;
        match self {
            Domain::XStart => mapped.x_start = new,
            Domain::XEnd => mapped.x_end = new,
            Domain::YStart => mapped.y_start = new,
            Domain::YEnd => mapped.y_end = new,
        }
    }
}

#[component]
fn DomainInput(leaf: LeafProps, value: f32, domain: Domain, label: String) -> Element {
    rsx! {
        NumberInput {
            value,
            aria_label: label,
            oncommit: move |new: f32| leaf.edit(|value| domain.set(value, new)),
        }
    }
}

/// Curve editor for `Curve2DNorm`, plus the x and y range for `Curve2D`.
#[component]
pub fn Curve2DField(props: LeafProps) -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut drag = use_signal(|| None::<CurveDrag>);
    // The field is keyed by its path, so the path never changes while it is mounted.
    let param_path = use_hook(|| CopyValue::new(props.path.clone()));

    let svg_width = format!("{}", CANVAS_WIDTH as i32);
    let svg_height = format!("{}", CANVAS_HEIGHT as i32);
    let view_box = format!("0 0 {} {}", CANVAS_WIDTH as i32, CANVAS_HEIGHT as i32);

    // Relative drag like the sliders: alt slows it down, shift snaps to the grid. The project
    // reruns on release.
    let mut drag_to = move |state: CurveDrag, client: (f64, f64), slow: bool, snap: bool| {
        let speed = if slow { ALT_SPEED_FACTOR } else { 1.0 };
        let delta = V2::new(
            ((client.0 - state.last.0) * speed / GRAPH_WIDTH) as f32,
            (-(client.1 - state.last.1) * speed / GRAPH_HEIGHT) as f32,
        );
        let pos = (state.pos + delta).max(V2::zero()).min(V2::new(1.0, 1.0));
        let point = if snap {
            V2::new(
                (pos.x * SNAP_STEPS).round() / SNAP_STEPS,
                (pos.y * SNAP_STEPS).round() / SNAP_STEPS,
            )
        } else {
            pos
        };
        edit_leaf_at(
            &param_path.read(),
            |value| move_curve_point(value, state.index, point),
            false,
        );
        drag.set(Some(CurveDrag {
            last: client,
            pos,
            active: true,
            ..state
        }));
    };
    let mut end_drag = move || {
        let was_active = drag().is_some_and(|state| state.active);
        drag.set(None);
        if was_active {
            ctx.trigger_run();
        }
    };

    let (graph, curve) = match &props.param.value {
        ProjectParamValue::Curve2DNorm(graph) => (graph.clone(), None),
        ProjectParamValue::Curve2D(curve) => (curve.get_curve_norm().clone(), Some(curve.clone())),
        _ => return rsx! { p { "unexpected value type" } },
    };
    let graph_len = graph.len();
    let line_points = graph
        .iter_points()
        .map(|p| {
            let x = p.x as f64 * GRAPH_WIDTH + MARGIN;
            let y = GRAPH_BOTTOM - (p.y as f64 * GRAPH_HEIGHT);
            format!("{x},{y}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    rsx! {
        style { {include_str!("curve_2d_field.css")} }
        div { class: "Curve2D",
            if let Some(curve) = curve {
                div { class: "domain-fields",
                    span { class: "hint", "x" }
                    DomainInput {
                        leaf: props.clone(),
                        value: Domain::XStart.get(&curve),
                        domain: Domain::XStart,
                        label: "x from",
                    }
                    span { class: "hint", "–" }
                    DomainInput {
                        leaf: props.clone(),
                        value: Domain::XEnd.get(&curve),
                        domain: Domain::XEnd,
                        label: "x to",
                    }
                    span { class: "hint", "y" }
                    DomainInput {
                        leaf: props.clone(),
                        value: Domain::YStart.get(&curve),
                        domain: Domain::YStart,
                        label: "y from",
                    }
                    span { class: "hint", "–" }
                    DomainInput {
                        leaf: props.clone(),
                        value: Domain::YEnd.get(&curve),
                        domain: Domain::YEnd,
                        label: "y to",
                    }
                }
            }
            svg {
                width: "{svg_width}",
                height: "{svg_height}",
                view_box: "{view_box}",
                preserve_aspect_ratio: "none",
                ondoubleclick: move |event| {
                    let rect = event.data.element_coordinates();
                    let x = ((rect.x - MARGIN) / GRAPH_WIDTH).clamp(0.0, 1.0) as f32;
                    let y = ((GRAPH_BOTTOM - rect.y) / GRAPH_HEIGHT).clamp(0.0, 1.0) as f32;
                    edit_leaf_at(&param_path.read(), |value| add_curve_point(value, V2::new(x, y)), true);
                },
                onmousemove: move |event| {
                    let Some(state) = drag() else { return };
                    if state.active {
                        return;
                    }
                    if !event.held_buttons().contains(MouseButton::Primary) {
                        drag.set(None);
                        return;
                    }
                    let client = event.client_coordinates();
                    let moved = (client.x - state.last.0).hypot(client.y - state.last.1);
                    if moved >= DRAG_THRESHOLD_PX {
                        let modifiers = event.modifiers();
                        drag_to(state, (client.x, client.y), modifiers.alt(), modifiers.shift());
                    }
                },
                onmouseup: move |_| end_drag(),

                for step in 1..(CANVAS_WIDTH / GRID_STEP) as i32 {
                    {
                        let at = step as f64 * GRID_STEP;
                        rsx! {
                            line { class: "curve-grid", x1: "{at}", y1: "0", x2: "{at}", y2: "{CANVAS_HEIGHT}" }
                            line { class: "curve-grid", x1: "0", y1: "{at}", x2: "{CANVAS_WIDTH}", y2: "{at}" }
                        }
                    }
                }
                polyline { points: line_points, class: "curve-line" }

                for (i, point) in graph.iter_points().enumerate() {
                    {
                        let cx = point.x as f64 * GRAPH_WIDTH + MARGIN;
                        let cy = GRAPH_BOTTOM - (point.y as f64 * GRAPH_HEIGHT);
                        rsx! {
                            circle {
                                cx: "{cx}",
                                cy: "{cy}",
                                r: "4",
                                class: "curve-handle",
                                ondoubleclick: move |event| {
                                    event.stop_propagation();
                                    if i > 0 && i < graph_len - 1 {
                                        edit_leaf_at(&param_path.read(), |value| remove_curve_point(value, i), true);
                                    }
                                },
                                onmousedown: move |event| {
                                    event.stop_propagation();
                                    event.prevent_default();
                                    let client = event.client_coordinates();
                                    drag.set(Some(CurveDrag {
                                        index: i,
                                        last: (client.x, client.y),
                                        pos: point,
                                        active: false,
                                    }));
                                },
                            }
                        }
                    }
                }
            }
            if drag().is_some_and(|state| state.active) {
                div {
                    class: "curve_drag_overlay",
                    onmousemove: move |event| {
                        if !event.held_buttons().contains(MouseButton::Primary) {
                            end_drag();
                            return;
                        }
                        let Some(state) = drag() else { return };
                        let client = event.client_coordinates();
                        let modifiers = event.modifiers();
                        drag_to(state, (client.x, client.y), modifiers.alt(), modifiers.shift());
                    },
                    onmouseup: move |_| end_drag(),
                }
            }
        }
    }
}
