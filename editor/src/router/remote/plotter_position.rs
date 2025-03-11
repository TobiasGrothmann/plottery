use dioxus::prelude::*;
use plottery_lib::V2;

#[derive(PartialEq, Props, Clone)]
pub struct PlotterPositionProps {
    location: V2,
}

#[component]
pub fn PlotterPosition(props: PlotterPositionProps) -> Element {
    let location_text = format!("X: {:.2}, Y: {:.2}", props.location.x, props.location.y);
    rsx! {
        p {"{location_text}"}
    }
}
