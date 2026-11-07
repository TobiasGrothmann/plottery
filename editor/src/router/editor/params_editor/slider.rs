use std::{rc::Rc, sync::Arc};

use dioxus::{html::input_data::MouseButton, prelude::*};
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use tokio::sync::Mutex;

use crate::router::editor::{
    console_messages::ConsoleMessages, params_editor::param_tree::get_param_value_mut_by_path,
    params_editor::params_editor::trigger_run, params_editor::widgets::format_number,
    project_runner::ProjectRunner, running_state::RunningState,
};

/// Must match `.slider_thumb` width in slider.css.
const THUMB_WIDTH_PX: f64 = 40.0;
const ALT_SPEED_FACTOR: f64 = 0.25;

#[derive(Props, Clone)]
pub struct EditorSliderProps {
    param: ProjectParam,
    path: Vec<String>,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for EditorSliderProps {
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

#[derive(Clone, Copy)]
struct Ranged {
    val: f32,
    min: f32,
    max: f32,
    is_int: bool,
}

impl Ranged {
    fn from_value(value: &ProjectParamValue) -> Self {
        match value {
            ProjectParamValue::FloatRanged { val, min, max } => Ranged {
                val: *val,
                min: *min,
                max: *max,
                is_int: false,
            },
            ProjectParamValue::IntRanged { val, min, max } => Ranged {
                val: *val as f32,
                min: *min as f32,
                max: *max as f32,
                is_int: true,
            },
            ProjectParamValue::Optional(optional) => Self::from_value(optional.value.as_ref()),
            _ => panic!("Unexpected parameter value type in slider"),
        }
    }

    fn snap(&self, val: f32) -> f32 {
        let val = val.clamp(self.min, self.max);
        if self.is_int {
            val.round()
        } else {
            val
        }
    }

    fn fraction(&self, val: f32) -> f32 {
        if self.max > self.min {
            ((val - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

fn set_ranged_value(param_value: &mut ProjectParamValue, new_val: f32) {
    match param_value {
        ProjectParamValue::FloatRanged { val, min, max } => *val = new_val.clamp(*min, *max),
        ProjectParamValue::IntRanged { val, min, max } => {
            *val = (new_val.round() as i32).clamp(*min, *max)
        }
        ProjectParamValue::Optional(optional) => set_ranged_value(optional.value.as_mut(), new_val),
        _ => panic!("Unexpected parameter value type in slider"),
    }
}

#[derive(Clone, Copy)]
struct DragState {
    last_x: f64,
    /// Pixels the thumb travels over the full value range.
    px_range: f64,
}

#[component]
pub fn Slider(mut props: EditorSliderProps) -> Element {
    let ranged = Ranged::from_value(&props.param.value);

    let mut slider_value = use_signal(|| ranged.val);
    let mut drag = use_signal(|| None::<DragState>);
    let mut track = use_signal(|| None::<Rc<MountedData>>);

    let shown_value = use_memo(move || ranged.snap(slider_value()));
    let slider_value_string = use_memo(move || format_number(shown_value()));
    let thumb_fraction = ranged.fraction(shown_value());

    let commit = use_callback(move |_: ()| {
        let mut new_params = props.project_params.read().clone();
        if let Some(param_value) = get_param_value_mut_by_path(&mut new_params.list, &props.path) {
            set_ranged_value(param_value, shown_value());
        } else {
            tracing::error!("Param path not found: {:?}", props.path);
            return;
        }
        props.project_params.set(new_params);
        trigger_run(
            props.project_runner,
            props.running_state,
            props.console,
            props.release,
        );
    });

    let mut end_drag = move || {
        if drag().is_some() {
            drag.set(None);
            commit.call(());
        }
    };

    rsx! {
        style { { include_str!("slider.css") } }
        div { class: "Slider",
            p { class: "slider_value",
                "{slider_value_string}"
            }
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
                    spawn(async move {
                        if let Ok(rect) = track.get_client_rect().await {
                            let px_range = (rect.size.width - THUMB_WIDTH_PX).max(1.0);
                            drag.set(Some(DragState { last_x, px_range }));
                        }
                    });
                },
                div {
                    class: "slider_thumb",
                    style: "left: calc({thumb_fraction} * (100% - {THUMB_WIDTH_PX}px));",
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
                    let delta = (x - state.last_x) * speed / state.px_range
                        * (ranged.max - ranged.min) as f64;
                    slider_value.set((slider_value() + delta as f32).clamp(ranged.min, ranged.max));
                    drag.set(Some(DragState { last_x: x, ..state }));
                },
                onmouseup: move |_| end_drag(),
            }
        }
    }
}
