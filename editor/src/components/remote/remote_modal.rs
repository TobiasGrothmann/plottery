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
    ui::{Button, StatusPill, Switch, Tone, Variant},
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
    let plotting = matches!(state, Some(Some(state)) if state.plotting);

    rsx! {
        style { { include_str!("./remote_modal.css") } }
        Modal {
            open,
            width: ModalWidth::Form,
            title: "Plotter remote",
            body: rsx! {
                match state {
                    Some(Some(state)) => rsx! { RemoteInternal { state } },
                    Some(None) => rsx! { p { class: "hint", "Start the plotter server to control it." } },
                    None => rsx! { p { class: "hint", "Looking for the plotter…" } },
                }
            },
            footer: rsx! {
                StatusPill { tone, text: status }
                if plotting {
                    Button {
                        class: "remote_abort",
                        onclick: move |event: MouseEvent| async move {
                            send_task(plottery_server_lib::Task::Abort).await.ok();
                            event.stop_propagation();
                        },
                        "Abort"
                    }
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
    let state = props.state;
    let mut target = use_signal(|| state.location);
    let target_now = *target.read();

    let mut set_target = move |event: FormEvent, set: fn(&mut V2, f32)| {
        if let Ok(value) = event.value().parse::<f32>() {
            set(&mut target.write(), value);
        }
        event.stop_propagation();
    };

    rsx! {
        div { class: "remote_sections",
            section { class: "remote_section",
                span { class: "caps", "Position" }
                div { class: "remote_grid",
                    {readout("x", state.location.x)}
                    {readout("y", state.location.y)}
                    Button {
                        onclick: move |event: MouseEvent| async move {
                            send_task(plottery_server_lib::Task::SetOrigin()).await.ok();
                            *target.write() = V2::zero();
                            event.stop_propagation();
                        },
                        "Set origin"
                    }
                }
            }
            section { class: "remote_section",
                span { class: "caps", "Move to" }
                div { class: "remote_grid",
                    label { class: "affix remote_field",
                        span { "x" }
                        input {
                            class: "input",
                            "aria-label": "Target x in cm",
                            value: format!("{:.2}", target_now.x),
                            onchange: move |event| set_target(event, |t, x| t.x = x),
                        }
                        span { class: "remote_unit", "cm" }
                    }
                    label { class: "affix remote_field",
                        span { "y" }
                        input {
                            class: "input",
                            "aria-label": "Target y in cm",
                            value: format!("{:.2}", target_now.y),
                            onchange: move |event| set_target(event, |t, y| t.y = y),
                        }
                        span { class: "remote_unit", "cm" }
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
            }
            section { class: "remote_section",
                span { class: "caps", "Machine" }
                div { class: "fgrid",
                    Switch {
                        checked: state.motors_enabled,
                        label: "Motors enabled",
                        onchange: move |enabled| async move {
                            send_task(plottery_server_lib::Task::SetEnabled(enabled)).await.ok();
                        },
                    }
                    Switch {
                        checked: state.head_down,
                        label: "Head down",
                        onchange: move |down| async move {
                            send_task(plottery_server_lib::Task::SetHead(down)).await.ok();
                        },
                    }
                }
            }
        }
    }
}

fn readout(axis: &str, value: f32) -> Element {
    rsx! {
        div { class: "affix remote_field remote_readout",
            span { "{axis}" }
            span { class: "mono selectable", "{value:.2}" }
            span { class: "remote_unit", "cm" }
        }
    }
}
