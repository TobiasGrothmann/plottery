use dioxus::prelude::*;
use plottery_lib::V2;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages, project_runner::ProjectRunner, running_state::RunningState,
};

// Margin is constant, dimensions are detected from CSS
const MARGIN: f64 = 10.0;

#[derive(PartialEq, Props, Clone)]
pub struct Curve2DNormProps {
    param: ProjectParam,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

#[component]
pub fn Curve2DField(mut props: Curve2DNormProps) -> Element {
    let mut dragging_index = use_signal(|| None::<usize>);
    let param_name = use_memo(move || props.param.name.clone());

    // Fixed 150x150 dimensions
    const CANVAS_WIDTH: f64 = 150.0;
    const CANVAS_HEIGHT: f64 = 150.0;
    const GRAPH_WIDTH: f64 = CANVAS_WIDTH - 2.0 * MARGIN;
    const GRAPH_HEIGHT: f64 = CANVAS_HEIGHT - 2.0 * MARGIN;
    const GRAPH_BOTTOM: f64 = CANVAS_HEIGHT - MARGIN;

    // Generate strings from constants
    let svg_width = format!("{}", CANVAS_WIDTH as i32);
    let svg_height = format!("{}", CANVAS_HEIGHT as i32);
    let view_box = format!("0 0 {} {}", CANVAS_WIDTH as i32, CANVAS_HEIGHT as i32);

    rsx! {
        style { { include_str!("curve_2d_field.css") } }
        div { class: "Curve2DNorm",
            {
                let param_value = props
                    .project_params
                    .read()
                    .list
                    .iter()
                    .find(|p| p.name == *param_name.read())
                    .map(|p| p.value.clone())
                    .unwrap_or_else(|| {
                        log::warn!("Curve2D parameter '{}' not found, using default", param_name.read());
                        ProjectParamValue::Curve2DNorm(Default::default())
                    });

                let is_curve2d = matches!(param_value, ProjectParamValue::Curve2D(_));

                let graph = match &param_value {
                    ProjectParamValue::Curve2DNorm(g) => g.clone(),
                    ProjectParamValue::Curve2D(c) => c.get_curve_norm().clone(),
                    _ => {
                        log::warn!("Unexpected parameter type for '{}', using default", param_name.read());
                        Default::default()
                    }
                };

                let graph_len = graph.len();

                rsx! {
                    // Domain input fields (only for Curve2D)
                    if is_curve2d {
                        if let ProjectParamValue::Curve2D(curve) = &param_value {
                            div { class: "domain-fields",
                                div { class: "domain-row",
                                    label { "X:" }
                                    input {
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "x_start",
                                        value: "{curve.mapped_to.x_start}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == *param_name.read() {
                                                        if let ProjectParamValue::Curve2D(ref mut c) = param_field.value {
                                                            c.mapped_to.x_start = val;
                                                        }
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                match props.project_runner.read().try_lock() {
                                                    Ok(mut runner) => {
                                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                                    },
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
                                        }
                                    }
                                    span { "to" }
                                    input {
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "x_end",
                                        value: "{curve.mapped_to.x_end}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == *param_name.read() {
                                                        if let ProjectParamValue::Curve2D(ref mut c) = param_field.value {
                                                            c.mapped_to.x_end = val;
                                                        }
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                match props.project_runner.read().try_lock() {
                                                    Ok(mut runner) => {
                                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                                    },
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "domain-row",
                                    label { "Y:" }
                                    input {
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "y_start",
                                        value: "{curve.mapped_to.y_start}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == *param_name.read() {
                                                        if let ProjectParamValue::Curve2D(ref mut c) = param_field.value {
                                                            c.mapped_to.y_start = val;
                                                        }
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                match props.project_runner.read().try_lock() {
                                                    Ok(mut runner) => {
                                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                                    },
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
                                        }
                                    }
                                    span { "to" }
                                    input {
                                        r#type: "number",
                                        step: "any",
                                        placeholder: "y_end",
                                        value: "{curve.mapped_to.y_end}",
                                        onchange: move |event| {
                                            if let Ok(val) = event.value().parse::<f32>() {
                                                let mut new_params = props.project_params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == *param_name.read() {
                                                        if let ProjectParamValue::Curve2D(ref mut c) = param_field.value {
                                                            c.mapped_to.y_end = val;
                                                        }
                                                    }
                                                }
                                                props.project_params.set(new_params);
                                                match props.project_runner.read().try_lock() {
                                                    Ok(mut runner) => {
                                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                                    },
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
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
                            event.prevent_default();
                            let rect = event.data.element_coordinates();
                            props.console.read().info(&format!("🖱️ Double click at ({}, {})", rect.x, rect.y));
                            let x = ((rect.x - MARGIN) / GRAPH_WIDTH).clamp(0.0, 1.0) as f32;
                            let y = ((GRAPH_BOTTOM - rect.y) / GRAPH_HEIGHT).clamp(0.0, 1.0) as f32;
                            props.console.read().info(&format!("📍 Normalized point: ({}, {})", x, y));

                            let mut new_params = props.project_params.read().clone();
                            for param_field in new_params.list.iter_mut() {
                                if param_field.name == *param_name.read() {
                                    match &mut param_field.value {
                                        ProjectParamValue::Curve2DNorm(ref mut g) => {
                                            let _ = g.add_point(plottery_lib::V2::new(x, y));
                                        }
                                        ProjectParamValue::Curve2D(ref mut c) => {
                                            let _ = c.add_point_norm(plottery_lib::V2::new(x, y));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            props.project_params.set(new_params);

                            match props.project_runner.read().try_lock() {
                                Ok(mut runner) => {
                                    runner.trigger_run_project(props.release, props.running_state, props.console);
                                },
                                Err(e) => {
                                    log::error!("Error preparing to run: {:?}", e);
                                    props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                },
                            }
                        },
                        onmousemove: move |event| {
                            if let Some(index) = dragging_index() {
                                let rect = event.data.element_coordinates();
                                let mut x = ((rect.x - MARGIN) / GRAPH_WIDTH).clamp(0.0, 1.0) as f32;
                                let mut y = ((GRAPH_BOTTOM - rect.y) / GRAPH_HEIGHT).clamp(0.0, 1.0) as f32;

                                // Snap to 0.1 grid when Alt key is held
                                if event.data.modifiers().alt() {
                                    x = (x * 10.0).round() / 10.0;
                                    y = (y * 10.0).round() / 10.0;
                                }

                                let mut new_params = props.project_params.read().clone();
                                for param_field in new_params.list.iter_mut() {
                                    if param_field.name == *param_name.read() {
                                        match &mut param_field.value {
                                            ProjectParamValue::Curve2DNorm(ref mut g) => {
                                                let total_points = g.len();
                                                if index == 0 {
                                                    g.update_endpoint(true, y);
                                                } else if index == total_points - 1 {
                                                    g.update_endpoint(false, y);
                                                } else {
                                                    let _ = g.update_point_norm(index - 1, V2::new(x, y));
                                                }
                                            }
                                            ProjectParamValue::Curve2D(ref mut c) => {
                                                let total_points = c.len();
                                                if index == 0 {
                                                    c.update_endpoint_norm(true, y);
                                                } else if index == total_points - 1 {
                                                    c.update_endpoint_norm(false, y);
                                                } else {
                                                    let _ = c.update_point_norm(index - 1, V2::new(x, y));
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                props.project_params.set(new_params);
                            }
                        },
                        onmouseup: move |_| {
                            if dragging_index().is_some() {
                                dragging_index.set(None);
                                match props.project_runner.read().try_lock() {
                                    Ok(mut runner) => {
                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                    },
                                    Err(e) => {
                                        log::error!("Error preparing to run: {:?}", e);
                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                    },
                                }
                            }
                        },
                        onmouseleave: move |_| {
                            if dragging_index().is_some() {
                                dragging_index.set(None);
                                // Commit changes when dragging outside bounds
                                match props.project_runner.read().try_lock() {
                                    Ok(mut runner) => {
                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                    },
                                    Err(e) => {
                                        log::error!("Error preparing to run: {:?}", e);
                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                    },
                                }
                            }
                        },

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
                            fill: "none",
                            stroke: "#333",
                            stroke_width: "2",
                        }

                        for (i, point) in graph.iter_points().enumerate() {
                            {
                                let cx = point.x as f64 * GRAPH_WIDTH + MARGIN;
                                let cy = GRAPH_BOTTOM - (point.y as f64 * GRAPH_HEIGHT);
                                rsx! {
                                    circle {
                                        cx: "{cx}",
                                        cy: "{cy}",
                                        r: "6",
                                        fill: "#333",
                                        style: "cursor: pointer;",
                                        onmousedown: move |event| {
                                            event.stop_propagation();
                                            dragging_index.set(Some(i));
                                        },
                                        ondoubleclick: move |event| {
                                            event.stop_propagation();
                                            if i > 0 && i < graph_len - 1 {
                                                let mut new_params = props.project_params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == *param_name.read() {
                                                        match &mut param_field.value {
                                                            ProjectParamValue::Curve2DNorm(c) => {
                                                                let _ = c.remove_point_at(i);
                                                            }
                                                            ProjectParamValue::Curve2D(ref mut c) => {
                                                                let _ = c.remove_point_at(i);
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                props.project_params.set(new_params);

                                                match props.project_runner.read().try_lock() {
                                                    Ok(mut runner) => {
                                                        runner.trigger_run_project(props.release, props.running_state, props.console);
                                                    },
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        props.running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
