use crate::util::format_svg;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

#[derive(PartialEq, Props, Clone)]
pub struct NavProps {
    pub page_name: String,
}

#[component]
pub fn Navigation(props: NavProps) -> Element {
    let nav = use_navigator();

    rsx! {
        style { { include_str!("./navigation.css") } }
        div { class: "Navigation",
            if nav.can_go_back() {
                button { class: "img_button",
                    onclick: move |_event| {
                        let nav = use_navigator();
                        nav.go_back();
                    },
                    img { src: "{format_svg(include_bytes!(\"../../public/icons/back.svg\"))}" }
                }
            }

            h1 { "{props.page_name.clone()}" }
        }
    }
}
