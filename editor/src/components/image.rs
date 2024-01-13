use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct ImageProps<'a> {
    pub class: &'a str,
    pub img_path: String,
}

pub fn Image<'a>(cx: Scope<'a, ImageProps>) -> Element<'a> {
    let img_path_str: &str = cx.props.img_path.as_str();

    cx.render(rsx!(
        style { include_str!("./image.css") }
        div { class: cx.props.class,
            img {
                class: "image",
                src: img_path_str
            }
        }
    ))
}
