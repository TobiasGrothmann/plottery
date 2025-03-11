use std::time::Duration;

use dioxus::prelude::*;
use dioxus_timer::use_timer;
use plottery_server_lib::server_state::{request_server_state, ServerState};

use crate::{
    components::navigation::Navigation, router::remote::plotter_position::PlotterPosition,
};

#[component]
pub fn Remote() -> Element {
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
                RemoteInternal {
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
struct RemoteInternalProps {
    state: ServerState,
}

fn RemoteInternal(props: RemoteInternalProps) -> Element {
    rsx! {
        Navigation { page_name: "Plotter Remote", body: rsx! {} }
        div {
            PlotterPosition {
                location: props.state.location
            }
        }
    }
}
