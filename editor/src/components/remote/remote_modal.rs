use dioxus::prelude::*;
use plottery_lib::V2;
use plottery_server_lib::{
    server_state::ServerState, task::send_task, Activity, PlotOutcome, PlotSettings, Task,
};

use crate::{
    components::{
        modal::{Modal, ModalWidth},
        remote::{poke_plotter, PlotterState, PlotterStatus},
        ui::{Button, StatusPill, Switch, Tone, Variant},
    },
    router::editor::console_messages::ActiveConsole,
    util::format_plot_duration,
};

/// Sends `task` and polls the plotter right after, so the modal shows the effect.
async fn send(task: Task) {
    let console = consume_context::<ActiveConsole>();
    if let Err(err) = send_task(task).await {
        console.error(&format!("failed to send task to the plotter: {err:#}"));
    }
    poke_plotter();
}

fn activity_label(activity: Activity) -> &'static str {
    match activity {
        Activity::Idle => "Idle",
        Activity::Preparing => "Preparing",
        Activity::Traveling => "Traveling",
        Activity::Drawing => "Drawing",
        Activity::Returning => "Returning to origin",
        Activity::Moving => "Moving",
        Activity::Aborting => "Aborting",
    }
}

#[component]
pub fn RemoteModal(open: Signal<bool>) -> Element {
    let PlotterState(status) = use_context::<PlotterState>();
    let status = status();

    let (tone, text) = match status {
        PlotterStatus::Connecting => (Tone::Busy, "Connecting…"),
        PlotterStatus::Offline => (Tone::Err, "Plotter offline"),
        PlotterStatus::DifferentVersion => (Tone::Warn, "Plotter server is a different version"),
        PlotterStatus::Online(state) if state.activity == Activity::Idle => (Tone::Ok, "Idle"),
        PlotterStatus::Online(state) => (Tone::Busy, activity_label(state.activity)),
    };
    let busy = matches!(status, PlotterStatus::Online(state) if state.activity != Activity::Idle);

    rsx! {
        style { { include_str!("./remote_modal.css") } }
        Modal {
            open,
            width: ModalWidth::Form,
            title: "Plotter remote",
            body: rsx! {
                match status {
                    PlotterStatus::Online(state) => rsx! { RemoteInternal { state } },
                    PlotterStatus::Offline => rsx! { p { class: "hint", "Start the plotter server to control it." } },
                    PlotterStatus::DifferentVersion => rsx! {
                        p { class: "hint", "Install the same Plottery version on the plotter and here to control it." }
                    },
                    PlotterStatus::Connecting => rsx! { p { class: "hint", "Looking for the plotter…" } },
                }
            },
            footer: rsx! {
                StatusPill { tone, text }
                if busy {
                    Button {
                        class: "remote_abort",
                        onclick: move |event: MouseEvent| async move {
                            event.stop_propagation();
                            send(Task::Abort).await;
                        },
                        "Abort"
                    }
                }
            },
        }
    }
}

/// The running plot's numbers, or how the last plot ended.
fn plot_section(state: &ServerState) -> Element {
    if let Some(plot) = state.plot {
        let percent = (plot.fraction() * 100.0).clamp(0.0, 100.0);
        return rsx! {
            div { class: "remote_plot_row",
                span { "{activity_label(state.activity)}" }
                span { class: "mono", "shape {plot.shape} of {plot.shapes}" }
            }
            div {
                class: "remote_progress",
                role: "progressbar",
                "aria-label": "Plot progress",
                "aria-valuemin": "0",
                "aria-valuemax": "100",
                "aria-valuenow": "{percent:.0}",
                div { class: "remote_progress_fill", width: "{percent:.1}%" }
            }
            div { class: "remote_plot_row",
                span { class: "mono", "{format_plot_duration(plot.remaining)} left" }
                span { class: "mono remote_plot_dim", "{format_plot_duration(plot.elapsed)} elapsed" }
            }
        };
    }

    let text = match (state.activity, state.last_plot) {
        (Activity::Preparing, _) => "Preparing the plot…".to_string(),
        (_, Some(result)) => {
            let duration = format_plot_duration(result.duration);
            match result.outcome {
                PlotOutcome::Finished => format!("Finished, took {duration}"),
                PlotOutcome::Aborted => format!("Aborted after {duration}"),
            }
        }
        (_, None) => "No plot since the plotter server started.".to_string(),
    };
    rsx! { p { class: "hint", "{text}" } }
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
                span { class: "caps", "Plot" }
                {plot_section(&state)}
            }
            section { class: "remote_section",
                span { class: "caps", "Position" }
                div { class: "remote_grid",
                    {readout("x", state.location.x)}
                    {readout("y", state.location.y)}
                    Button {
                        onclick: move |event: MouseEvent| async move {
                            send(Task::SetOrigin()).await;
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
                            let target = *target.read();
                            send(Task::MoveTo(target, PlotSettings::default())).await;
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
                            send(Task::SetEnabled(enabled)).await;
                        },
                    }
                    Switch {
                        checked: state.head_down,
                        label: "Head down",
                        onchange: move |down| async move {
                            send(Task::SetHead(down)).await;
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
