use crate::{router::editor::editor_ctx::EditorCtx, util::format_svg};
use dioxus::prelude::*;
use std::path::PathBuf;

/// Which folder an external app opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpensDir {
    Cargo,
    Project,
}

/// An app found on `PATH` by its `command` and launched as `command [args] <dir>`.
#[derive(Debug, PartialEq)]
pub struct ExternalEditor {
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub icon_svg: &'static [u8],
    pub opens: OpensDir,
}

pub const EXTERNAL_EDITORS: [ExternalEditor; 5] = [
    ExternalEditor {
        name: "VS Code",
        command: "code",
        args: &[],
        icon_svg: include_bytes!("../../../public/icons/vscode.svg"),
        opens: OpensDir::Cargo,
    },
    ExternalEditor {
        name: "Zed",
        command: "zed",
        args: &[],
        icon_svg: include_bytes!("../../../public/icons/zed.svg"),
        opens: OpensDir::Cargo,
    },
    ExternalEditor {
        name: "Cursor",
        command: "cursor",
        args: &[],
        icon_svg: include_bytes!("../../../public/icons/cursor.svg"),
        opens: OpensDir::Cargo,
    },
    ExternalEditor {
        name: "Sublime Text",
        command: "subl",
        args: &[],
        icon_svg: include_bytes!("../../../public/icons/sublime-text.svg"),
        opens: OpensDir::Cargo,
    },
    ExternalEditor {
        name: "GitKraken",
        command: "GitKraken",
        args: &["-p"],
        icon_svg: include_bytes!("../../../public/icons/gitkraken.svg"),
        opens: OpensDir::Project,
    },
];

/// The entries of `EXTERNAL_EDITORS` whose command is on `PATH`.
pub fn installed_external_editors() -> Vec<&'static ExternalEditor> {
    EXTERNAL_EDITORS
        .iter()
        .filter(|editor| which::which(editor.command).is_ok())
        .collect()
}

#[derive(PartialEq, Props, Clone)]
pub struct ExternalEditorButtonProps {
    pub editor: &'static ExternalEditor,
    pub dir: ReadSignal<PathBuf>,
}

#[component]
pub fn ExternalEditorButton(props: ExternalEditorButtonProps) -> Element {
    let console = use_context::<EditorCtx>().console;
    let editor = props.editor;
    rsx! {
        button {
            r#type: "button",
            class: "btn btn-ghost icon-btn app-btn",
            "aria-label": "Open in {editor.name}",
            title: "Open in {editor.name}",
            onclick: move |event| {
                let launched = std::process::Command::new(editor.command)
                    .args(editor.args)
                    .arg(&*props.dir.read())
                    .spawn()
                    .and_then(|mut child| child.wait());
                if let Err(err) = launched {
                    console.peek().error(&format!("Failed to open {}: {err}", editor.name));
                }
                event.stop_propagation();
            },
            img { src: "{format_svg(editor.icon_svg)}", alt: "" }
        }
    }
}
