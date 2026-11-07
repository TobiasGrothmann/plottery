use std::time::Duration;

use dioxus::prelude::*;
use plottery_lib::V2;
use plottery_server_lib::{
    server_state::{request_server_state, ServerState},
    task::send_task,
    PlotSettings,
};

use dioxus_router::hooks::use_navigator;

use crate::{
    components::{
        icon::IconName,
        ui::{Button, IconButton, StatusPill, Tone, Variant},
    },
    routes::Route,
};

#[component]
pub fn Remote() -> Element {
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

    let nav = use_navigator();

    rsx! {
        style { { include_str!("./remote.css") } }
        div { class: "Remote",
            header { class: "app-header",
                div { class: "hgroup",
                    IconButton {
                        icon: IconName::ChevronLeft,
                        label: "Back",
                        onclick: move |_| {
                            if nav.can_go_back() {
                                nav.go_back();
                            } else {
                                nav.replace(Route::Browser {});
                            }
                        },
                    }
                    h1 { "Plotter remote" }
                }
            }
            div { class: "remote_body",
                div { class: "remote_content",
                    if let Some(state) = &*state_resource.value().read() {
                        if let Ok(state) = state {
                            RemoteInternal {
                                state: *state
                            }
                        } else {
                            div { class: "remote_row",
                                StatusPill { tone: Tone::Err, text: "Plotter offline" }
                            }
                        }
                    } else {
                        div { class: "remote_row",
                            StatusPill { tone: Tone::Busy, text: "Connecting…" }
                        }
                    }
                }
            }
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
    let activity = if props.state.plotting {
        "Plotting"
    } else {
        "Idle"
    };

    let mut target = use_signal(|| props.state.location);
    let target_x_pos = format!("{:.2}", target.read().x);
    let target_y_pos = format!("{:.2}", target.read().y);

    rsx! {
        div { class: "remote_row",
            span { class: "remote_label", "Position" }
            span { class: "mono remote_value selectable", "{x_pos}" }
            span { class: "mono remote_value selectable", "{y_pos}" }
            StatusPill {
                tone: if props.state.plotting { Tone::Busy } else { Tone::Ok },
                text: activity,
            }
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
