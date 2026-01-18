use crate::util::format_svg;
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(PartialEq, Props, Clone)]
pub struct ExternalEditorButtonProps {
    pub editor_name: String,
    pub editor_command: String,
    pub icon_svg: &'static [u8],
    pub project_dir: ReadSignal<PathBuf>,
    #[props(default = vec![])]
    pub extra_args: Vec<String>,
}

#[component]
pub fn ExternalEditorButton(props: ExternalEditorButtonProps) -> Element {
    rsx! {
        button {
            class: "icon_button",
            title: "Open in {props.editor_name}",
            onclick: move |event| {
                let mut cmd = std::process::Command::new(&props.editor_command);
                for arg in &props.extra_args {
                    cmd.arg(arg);
                }
                cmd.arg(&*props.project_dir.read())
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
                event.stop_propagation();
            },
            img { src: "{format_svg(props.icon_svg)}" }
        }
    }
}
