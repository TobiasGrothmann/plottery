pub mod remote_modal;

use dioxus::prelude::*;

use remote_modal::RemoteModal;

const RESTORE_FOCUS: &str = r#"
const el = window.__remoteReturnFocus;
if (el && el.isConnected && !document.querySelector('.scrim')) el.focus();
"#;

/// Whether the plotter remote modal is open; provided at the app root.
#[derive(Clone, Copy)]
pub struct RemoteOpen(pub Signal<bool>);

/// Hosts the plotter remote modal and its global R shortcut.
#[component]
pub fn PlotterRemote() -> Element {
    let RemoteOpen(mut open) = use_context::<RemoteOpen>();

    use_future(move || async move {
        let mut eval = document::eval(include_str!("remote_shortcut.js"));
        while eval.recv::<String>().await.is_ok() {
            open.toggle();
        }
    });

    use_effect(move || {
        if !open() {
            document::eval(RESTORE_FOCUS);
        }
    });

    rsx! {
        if open() {
            RemoteModal { open }
        }
    }
}
