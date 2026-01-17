use std::time::Duration;

use dioxus::prelude::*;
use plottery_lib::V2;
use plottery_server_lib::{
    server_state::{request_server_state, ServerState},
    task::send_task,
    PlotSettings,
};

use crate::components::navigation::Navigation;

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

    rsx! {
        Navigation { page_name: "Plotter Remote", body: rsx! {} }
        div { class: "Remote",
        style { { include_str!("./remote.css") } }
            div { class: "remote_content",
                if let Some(state) = &*state_resource.value().read() {
                    if let Ok(state) = state {
                        RemoteInternal {
                            state: *state
                        }
                    } else {
                        p {"Plotter offline"}
                    }
                } else {
                    p {"Loading..."}
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
        div { class: "row",
            h2 { "position" }
            p { class: "pos_item", "{x_pos}" }
            p { class: "pos_item", "{y_pos}" }
            p { class: "pos_item", "{activity}" }
        }

        div { class: "row",
            h2 { "target" }
            input {
                class: "target_item",
                value: target_x_pos,
                onchange: move |event| async move {
                    event.value().parse::<f32>().map(|x| {
                        let mut target_handle = target.write();
                        *target_handle = V2::new(x, target_handle.y)
                    }).ok();
                    event.stop_propagation();
                }
            }
            input {
                class: "target_item",
                value: target_y_pos,
                onchange: move |event| async move {
                    event.value().parse::<f32>().map(|y| {
                        let mut target_handle = target.write();
                        *target_handle = V2::new(target_handle.x, y)
                    }).ok();
                    event.stop_propagation();
                }
            }
            button {
                onclick: move |event| async move {
                    send_task(plottery_server_lib::Task::MoveTo(*target.read(), PlotSettings::default())).await.ok();
                    event.stop_propagation();
                },
                p { "move" }
            }
        }

        div { class: "row",
            button {
                onclick: move |event| async move {
                    send_task(plottery_server_lib::Task::SetOrigin()).await.ok();
                    *target.write() = V2::zero();
                    event.stop_propagation();
                },
                p { "set origin" }
            }
        }

        div { class: "row",
            button {
                onclick: move |event| async move {
                    send_task(plottery_server_lib::Task::SetEnabled(!props.state.motors_enabled)).await.ok();
                    event.stop_propagation();
                },
                p { if props.state.motors_enabled { "motors are enabled" } else { "motors are disabled" } }
            }
            button {
                onclick: move |event| async move {
                    send_task(plottery_server_lib::Task::SetHead(!props.state.head_down)).await.ok();
                    event.stop_propagation();
                },
                p { if props.state.head_down { "head is down" } else { "head is up" } }
            }
        }
    }
}
