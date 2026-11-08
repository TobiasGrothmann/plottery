use std::time::Duration;

use dioxus::prelude::*;
use plottery_lib::V2;
use plottery_server_lib::{
    server_state::{request_server_state, ServerState},
    task::send_task,
    PlotSettings,
};

use crate::components::{
    modal::{Modal, ModalWidth},
    ui::{Button, StatusPill, Tone, Variant},
};

#[component]
pub fn RemoteModal(open: Signal<bool>) -> Element {
    let mut state_resource =
        use_resource(async move || request_server_state(Some(Duration::from_millis(1000))).await);

    // Poll server state every 1200ms
    use_future(move || async move {
        let mut interval = tokio::time::interval(Duration::from_millis(1200));
        loop {
            interval.tick().await;
            state_resource.restart();
        }
    });

    // None while connecting, Some(None) when offline.
    let state = state_resource
        .value()
        .read()
        .as_ref()
        .map(|result| result.as_ref().ok().copied());
    let (tone, status) = match &state {
        None => (Tone::Busy, "Connecting…"),
        Some(None) => (Tone::Err, "Plotter offline"),
        Some(Some(state)) if state.plotting => (Tone::Busy, "Plotting"),
        Some(Some(_)) => (Tone::Ok, "Idle"),
    };

    rsx! {
        style { { include_str!("./remote_modal.css") } }
        Modal {
            open,
            width: ModalWidth::Form,
            title: "Plotter remote",
            subtitle: "Press R anywhere to open or close.",
            body: rsx! {
                match state {
                    Some(Some(state)) => rsx! { RemoteInternal { state } },
                    Some(None) => rsx! { p { class: "hint", "Start the plotter server to control it." } },
                    None => rsx! { p { class: "hint", "Looking for the plotter…" } },
                }
            },
            footer: rsx! {
                StatusPill { tone, text: status }
                Button {
                    variant: Variant::Primary,
                    onclick: move |_| open.set(false),
                    "Done"
                }
            },
        }
    }
}

#[derive(PartialEq, Props, Clone)]
struct RemoteInternalProps {
    state: ServerState,
}

fn RemoteInternal(props: RemoteInternalProps) -> Element {
    let x_pos = format!("{:.2}", props.state.location.x);
    let y_pos = format!("{:.2}", props.state.location.y);

    let mut target = use_signal(|| props.state.location);
    let target_x_pos = format!("{:.2}", target.read().x);
    let target_y_pos = format!("{:.2}", target.read().y);

    rsx! {
        div { class: "remote_rows",
            div { class: "remote_row",
                span { class: "remote_label", "Position" }
                span { class: "mono remote_value selectable", "{x_pos}" }
                span { class: "mono remote_value selectable", "{y_pos}" }
            }

            div { class: "remote_row",
                span { class: "remote_label", "Target" }
                label { class: "affix remote_input",
                    span { "x" }
                    input {
                        class: "input",
                        "aria-label": "Target x",
                        value: target_x_pos,
                        onchange: move |event| async move {
                            event.value().parse::<f32>().map(|x| {
                                let mut target_handle = target.write();
                                *target_handle = V2::new(x, target_handle.y)
                            }).ok();
                            event.stop_propagation();
                        }
                    }
                }
                label { class: "affix remote_input",
                    span { "y" }
                    input {
                        class: "input",
                        "aria-label": "Target y",
                        value: target_y_pos,
                        onchange: move |event| async move {
                            event.value().parse::<f32>().map(|y| {
                                let mut target_handle = target.write();
                                *target_handle = V2::new(target_handle.x, y)
                            }).ok();
                            event.stop_propagation();
                        }
                    }
                }
                Button {
                    variant: Variant::Primary,
                    onclick: move |event: MouseEvent| async move {
                        send_task(plottery_server_lib::Task::MoveTo(*target.read(), PlotSettings::default())).await.ok();
                        event.stop_propagation();
                    },
                    "Move"
                }
            }

            div { class: "remote_row",
                span { class: "remote_label", "Origin" }
                Button {
                    onclick: move |event: MouseEvent| async move {
                        send_task(plottery_server_lib::Task::SetOrigin()).await.ok();
                        *target.write() = V2::zero();
                        event.stop_propagation();
                    },
                    "Set origin"
                }
            }

            div { class: "remote_row",
                span { class: "remote_label", "Motors" }
                StatusPill {
                    tone: if props.state.motors_enabled { Tone::Ok } else { Tone::Warn },
                    text: if props.state.motors_enabled { "motors are enabled" } else { "motors are disabled" },
                }
                Button {
                    variant: Variant::Ghost,
                    onclick: move |event: MouseEvent| async move {
                        send_task(plottery_server_lib::Task::SetEnabled(!props.state.motors_enabled)).await.ok();
                        event.stop_propagation();
                    },
                    if props.state.motors_enabled { "Disable" } else { "Enable" }
                }
            }

            div { class: "remote_row",
                span { class: "remote_label", "Head" }
                StatusPill {
                    tone: if props.state.head_down { Tone::Busy } else { Tone::Ok },
                    text: if props.state.head_down { "head is down" } else { "head is up" },
                }
                Button {
                    variant: Variant::Ghost,
                    onclick: move |event: MouseEvent| async move {
                        send_task(plottery_server_lib::Task::SetHead(!props.state.head_down)).await.ok();
                        event.stop_propagation();
                    },
                    if props.state.head_down { "Raise" } else { "Lower" }
                }
            }
        }
    }
}
