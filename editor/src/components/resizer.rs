use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy)]
struct Drag {
    start_pointer: f64,
    start_value: f64,
    value: f64,
}

#[component]
pub fn Resizer(
    axis: Axis,
    value: f64,
    min: f64,
    max: f64,
    default: f64,
    #[props(default)] invert: bool,
    onchange: EventHandler<f64>,
    oncommit: EventHandler<f64>,
    label: String,
) -> Element {
    let mut drag = use_signal(|| None::<Drag>);
    let pointer = move |event: &PointerEvent| {
        let point = event.client_coordinates();
        match axis {
            Axis::Vertical => point.x,
            Axis::Horizontal => point.y,
        }
    };
    let (class, orientation, cursor) = match axis {
        Axis::Vertical => ("resizer", "vertical", "col-resize"),
        Axis::Horizontal => ("resizer-y", "horizontal", "row-resize"),
    };
    let active = if drag().is_some() { " active" } else { "" };

    rsx! {
        div {
            class: "{class}{active}",
            role: "separator",
            "aria-orientation": orientation,
            "aria-label": "{label}",
            "aria-valuenow": "{value.round()}",
            "aria-valuemin": "{min}",
            "aria-valuemax": "{max}",
            title: "Drag to resize · double-click to reset",
            onpointerdown: move |event| {
                event.prevent_default();
                drag.set(Some(Drag {
                    start_pointer: pointer(&event),
                    start_value: value,
                    value,
                }));
            },
            ondoubleclick: move |_| {
                onchange.call(default);
                oncommit.call(default);
            },
        }
        if let Some(state) = drag() {
            div {
                class: "resize-overlay",
                style: "cursor: {cursor};",
                onpointermove: move |event| {
                    if event.held_buttons().is_empty() {
                        drag.set(None);
                        oncommit.call(state.value);
                        return;
                    }
                    let delta = pointer(&event) - state.start_pointer;
                    let delta = if invert { -delta } else { delta };
                    let next = (state.start_value + delta).clamp(min, max);
                    drag.set(Some(Drag { value: next, ..state }));
                    onchange.call(next);
                },
                onpointerup: move |_| {
                    drag.set(None);
                    oncommit.call(state.value);
                },
            }
        }
    }
}
