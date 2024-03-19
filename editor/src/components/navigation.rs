use crate::util::format_svg;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

#[derive(PartialEq, Props)]
pub struct NavProps<'a> {
    pub page_name: &'a str,
}

#[component]
pub fn Navigation<'a>(cx: Scope<'a, NavProps>) -> Element<'a> {
    let nav = use_navigator(cx);

    cx.render(rsx!(
        style { include_str!("./navigation.css") }
        div { class: "Navigation",
            if nav.can_go_back() {
                cx.render(rsx!(
                    button { class: "img_button",
                    onclick: move |_event| {
                        let nav = use_navigator(cx);
                        nav.go_back();
                    },
                    img { src: "{format_svg(include_bytes!(\"../../public/icons/back.svg\"))}" }
                }
                ))
            }

            h1 { cx.props.page_name.clone() }
        }
    ))
}
