use crate::{router::editor::editor_ctx::EditorCtx, util::format_svg};
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
    let console = use_context::<EditorCtx>().console;
    rsx! {
        button {
            r#type: "button",
            class: "btn btn-ghost icon-btn app-btn",
            "aria-label": "Open in {props.editor_name}",
            title: "Open in {props.editor_name}",
            onclick: move |event| {
                let mut cmd = std::process::Command::new(&props.editor_command);
                for arg in &props.extra_args {
                    cmd.arg(arg);
                }
                let launched = cmd
                    .arg(&*props.project_dir.read())
                    .spawn()
                    .and_then(|mut child| child.wait());
                if let Err(err) = launched {
                    console.peek().error(&format!("failed to open {}: {err}", props.editor_name));
                }
                event.stop_propagation();
            },
            img { src: "{format_svg(props.icon_svg)}", alt: "" }
        }
    }
}
