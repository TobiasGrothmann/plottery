use std::time::Duration;

use dioxus::prelude::*;
use dioxus_timer::use_timer;
use plottery_server_lib::server_state::{request_server_state, ServerState};

#[component]
pub fn PlotterPosition() -> Element {
    let polling_interval = Duration::from_millis(1000);
    let timer = use_timer(polling_interval);

    let mut state_resource = use_resource(async move || request_server_state().await);
    use_effect(move || {
        timer.read();
        state_resource.restart();
    });

    if let Some(state) = &*state_resource.value().read() {
        if let Ok(state) = state {
            rsx! {
                PlotterPositionInternal {
                    state: state.clone()
                }
            }
        } else {
            rsx! { p {"Plotter offline"} }
        }
    } else {
        rsx! { p {"Loading..."} }
    }
}

#[derive(PartialEq, Props, Clone)]
struct PlotterPositionInternalProps {
    state: ServerState,
}

fn PlotterPositionInternal(props: PlotterPositionInternalProps) -> Element {
    let location_text = format!(
        "X: {:.2}, Y: {:.2}",
        props.state.location.x, props.state.location.y
    );
    rsx! {
        p {"{location_text}"}
    }
}
