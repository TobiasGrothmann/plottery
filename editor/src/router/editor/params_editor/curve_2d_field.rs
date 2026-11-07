use dioxus::{html::input_data::MouseButton, prelude::*};
use dioxus_logger::tracing;
use plottery_lib::V2;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::param_tree::{get_param_value_by_path, get_param_value_mut_by_path},
    params_editor::params_editor::trigger_run,
    params_editor::widgets::slider_input::ALT_SPEED_FACTOR,
    project_runner::ProjectRunner,
    running_state::RunningState,
};

// Margin is constant, dimensions are detected from CSS
const MARGIN: f64 = 8.0;
/// Pixels the mouse has to move before a press on a handle becomes a drag.
const DRAG_THRESHOLD_PX: f64 = 2.0;
/// Grid the handles snap to while shift is held.
const SNAP_STEPS: f32 = 10.0;

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
        ProjectParamValue::Optional(optional) => {
            move_curve_point(optional.value.as_mut(), index, point)
        }
        _ => {}
    }
}

#[derive(Props, Clone)]
pub struct Curve2DNormProps {
    param: ProjectParam,
    path: Vec<String>,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for Curve2DNormProps {
    fn eq(&self, other: &Self) -> bool {
        self.param.name == other.param.name
            && self.param.value == other.param.value
            && self.path == other.path
            && self.project_params == other.project_params
            && self.project_runner == other.project_runner
            && self.running_state == other.running_state
            && self.console == other.console
            && self.release == other.release
    }
}

#[component]
pub fn Curve2DField(mut props: Curve2DNormProps) -> Element {
    let mut drag = use_signal(|| None::<CurveDrag>);
    let param_path = use_memo(move || props.path.clone());

    const CANVAS_WIDTH: f64 = 128.0;
    const CANVAS_HEIGHT: f64 = 128.0;
    const GRID_STEP: f64 = 32.0;
    const GRAPH_WIDTH: f64 = CANVAS_WIDTH - 2.0 * MARGIN;
    const GRAPH_HEIGHT: f64 = CANVAS_HEIGHT - 2.0 * MARGIN;
    const GRAPH_BOTTOM: f64 = CANVAS_HEIGHT - MARGIN;

    // Generate strings from constants
    let svg_width = format!("{}", CANVAS_WIDTH as i32);
    let svg_height = format!("{}", CANVAS_HEIGHT as i32);
    let view_box = format!("0 0 {} {}", CANVAS_WIDTH as i32, CANVAS_HEIGHT as i32);

    // Relative drag like the sliders: alt slows it down, shift snaps to the grid.
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
        let mut new_params = props.project_params.read().clone();
        if let Some(value) = get_param_value_mut_by_path(&mut new_params.list, &param_path.read()) {
            move_curve_point(value, state.index, point);
        }
        props.project_params.set(new_params);
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
            trigger_run(
                props.project_runner,
                props.running_state,
                props.console,
                props.release,
            );
        }
    };

    rsx! {
        style { { include_str!("curve_2d_field.css") } }
        div { class: "Curve2D",
            {
                let path = param_path.read().clone();
                let raw_param_value = get_param_value_by_path(&props.project_params.read().list, &path)
                .unwrap_or_else(|| {
                    tracing::warn!("Curve2D parameter path '{:?}' not found, using default", path);
                    ProjectParamValue::Curve2DNorm(Default::default())
                });

                let param_value = match raw_param_value {
                    ProjectParamValue::Optional(optional) => {
                        if optional.enabled {
                            (*optional.value).clone()
                        } else {
                            ProjectParamValue::Curve2DNorm(Default::default())
                        }
                    }
                    value => value,
                };

                let is_curve2d = matches!(param_value, ProjectParamValue::Curve2D(_));

                let graph = match &param_value {
                    ProjectParamValue::Curve2DNorm(g) => g.clone(),
                    ProjectParamValue::Curve2D(c) => c.get_curve_norm().clone(),
                    _ => {
                        tracing::warn!("Unexpected parameter type for path '{:?}', using default", path);
                        Default::default()
                    }
                };

                let graph_len = graph.len();

                rsx! {
                    // Domain input fields (only for Curve2D)
                    if is_curve2d {
                        if let ProjectParamValue::Curve2D(curve) = &param_value {
                            div { class: "domain-fields",
                                    span { class: "hint", "x" }
                                    input {
                                        class: "input",
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "x_start",
                                        "aria-label": "x from",
                                        value: "{curve.mapped_to.x_start}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                let path = param_path.read().clone();
                                                if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &path) {
                                                    match param_value {
                                                        ProjectParamValue::Curve2D(c) => c.mapped_to.x_start = val,
                                                        ProjectParamValue::Optional(optional) => {
                                                            if let ProjectParamValue::Curve2D(c) = optional.value.as_mut() {
                                                                c.mapped_to.x_start = val;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                trigger_run(props.project_runner, props.running_state, props.console, props.release);
                                            }
                                        }
                                    }
                                    span { class: "hint", "–" }
                                    input {
                                        class: "input",
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "x_end",
                                        "aria-label": "x to",
                                        value: "{curve.mapped_to.x_end}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                let path = param_path.read().clone();
                                                if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &path) {
                                                    match param_value {
                                                        ProjectParamValue::Curve2D(c) => c.mapped_to.x_end = val,
                                                        ProjectParamValue::Optional(optional) => {
                                                            if let ProjectParamValue::Curve2D(c) = optional.value.as_mut() {
                                                                c.mapped_to.x_end = val;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                trigger_run(props.project_runner, props.running_state, props.console, props.release);
                                            }
                                        }
                                    }
                                    span { class: "hint", "y" }
                                    input {
                                        class: "input",
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "y_start",
                                        "aria-label": "y from",
                                        value: "{curve.mapped_to.y_start}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                let path = param_path.read().clone();
                                                if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &path) {
                                                    match param_value {
                                                        ProjectParamValue::Curve2D(c) => c.mapped_to.y_start = val,
                                                        ProjectParamValue::Optional(optional) => {
                                                            if let ProjectParamValue::Curve2D(c) = optional.value.as_mut() {
                                                                c.mapped_to.y_start = val;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                trigger_run(props.project_runner, props.running_state, props.console, props.release);
                                            }
                                        }
                                    }
                                    span { class: "hint", "–" }
                                    input {
                                        class: "input",
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "y_end",
                                        "aria-label": "y to",
                                        value: "{curve.mapped_to.y_end}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                let path = param_path.read().clone();
                                                if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &path) {
                                                    match param_value {
                                                        ProjectParamValue::Curve2D(c) => c.mapped_to.y_end = val,
                                                        ProjectParamValue::Optional(optional) => {
                                                            if let ProjectParamValue::Curve2D(c) = optional.value.as_mut() {
                                                                c.mapped_to.y_end = val;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                trigger_run(props.project_runner, props.running_state, props.console, props.release);
                                            }
                                        }
                                    }
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

                            let mut new_params = props.project_params.read().clone();
                            let path = param_path.read().clone();
                            if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &path) {
                                match param_value {
                                    ProjectParamValue::Curve2DNorm(g) => {
                                        let _ = g.add_point(plottery_lib::V2::new(x, y));
                                    }
                                    ProjectParamValue::Curve2D(c) => {
                                        let _ = c.add_point_norm(plottery_lib::V2::new(x, y));
                                    }
                                    ProjectParamValue::Optional(optional) => {
                                        match optional.value.as_mut() {
                                            ProjectParamValue::Curve2DNorm(g) => {
                                                let _ = g.add_point(plottery_lib::V2::new(x, y));
                                            }
                                            ProjectParamValue::Curve2D(c) => {
                                                let _ = c.add_point_norm(plottery_lib::V2::new(x, y));
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            props.project_params.set(new_params);

                            trigger_run(props.project_runner, props.running_state, props.console, props.release);
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
                        polyline {
                            points: {
                                graph.iter_points()
                                    .map(|p| {
                                        let x = p.x as f64 * GRAPH_WIDTH + MARGIN;
                                        let y = GRAPH_BOTTOM - (p.y as f64 * GRAPH_HEIGHT);
                                        format!("{},{}", x, y)
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            },
                            class: "curve-line",
                        }

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
                                                let mut new_params = props.project_params.read().clone();
                                                let path = param_path.read().clone();
                                                if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &path) {
                                                    match param_value {
                                                        ProjectParamValue::Curve2DNorm(c) => {
                                                            let _ = c.remove_point_at(i);
                                                        }
                                                        ProjectParamValue::Curve2D(c) => {
                                                            let _ = c.remove_point_at(i);
                                                        }
                                                        ProjectParamValue::Optional(optional) => {
                                                            match optional.value.as_mut() {
                                                                ProjectParamValue::Curve2DNorm(c) => {
                                                                    let _ = c.remove_point_at(i);
                                                                }
                                                                ProjectParamValue::Curve2D(c) => {
                                                                    let _ = c.remove_point_at(i);
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                props.project_params.set(new_params);

                                                trigger_run(props.project_runner, props.running_state, props.console, props.release);
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
