use dioxus::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IconName {
    ChevronLeft,
    ChevronRight,
    ChevronDown,
    Play,
    Send,
    Sliders,
    External,
    PanelLeft,
    PanelRight,
    PanelBottom,
    Folder,
    Crosshair,
    Dice,
    X,
    Plus,
    Minus,
    Eye,
    EyeOff,
    Trash,
    Refresh,
    Reset,
    Image,
    Plotter,
    Fit,
    Alert,
    Code,
    GitBranch,
}

fn frame() -> Element {
    rsx! { rect { x: "3", y: "4", width: "18", height: "16", rx: "2" } }
}

fn eye() -> Element {
    rsx! {
        path { d: "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" }
        circle { cx: "12", cy: "12", r: "3" }
    }
}

fn children(name: IconName) -> Element {
    match name {
        IconName::ChevronLeft => rsx! { path { d: "M15 18l-6-6 6-6" } },
        IconName::ChevronRight => rsx! { path { d: "M9 18l6-6-6-6" } },
        IconName::ChevronDown => rsx! { path { d: "M6 9l6 6 6-6" } },
        IconName::Play => rsx! { path { d: "M7 4.5v15l12-7.5z", fill: "currentColor" } },
        IconName::Send => rsx! { path { d: "M22 2L11 13M22 2l-7 20-4-9-9-4z" } },
        IconName::Sliders => rsx! {
            path { d: "M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12" }
            circle { cx: "16", cy: "6", r: "2" }
            circle { cx: "10", cy: "12", r: "2" }
            circle { cx: "18", cy: "18", r: "2" }
        },
        IconName::External => rsx! {
            path { d: "M14 3h7v7M21 3l-9 9M19 14v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h5" }
        },
        IconName::PanelLeft => rsx! { {frame()} path { d: "M9 4v16" } },
        IconName::PanelRight => rsx! { {frame()} path { d: "M15 4v16" } },
        IconName::PanelBottom => rsx! { {frame()} path { d: "M3 15h18" } },
        IconName::Folder => rsx! {
            path { d: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
        },
        IconName::Crosshair => rsx! {
            circle { cx: "12", cy: "12", r: "7" }
            path { d: "M12 2v4M12 18v4M2 12h4M18 12h4" }
        },
        IconName::Dice => rsx! {
            rect { x: "4", y: "4", width: "16", height: "16", rx: "3" }
            circle { cx: "9", cy: "9", r: ".6", fill: "currentColor" }
            circle { cx: "12", cy: "12", r: ".6", fill: "currentColor" }
            circle { cx: "15", cy: "15", r: ".6", fill: "currentColor" }
        },
        IconName::X => rsx! { path { d: "M6 6l12 12M18 6L6 18" } },
        IconName::Plus => rsx! { path { d: "M12 5v14M5 12h14" } },
        IconName::Minus => rsx! { path { d: "M5 12h14" } },
        IconName::Eye => eye(),
        IconName::EyeOff => rsx! { {eye()} path { d: "M3 3l18 18" } },
        IconName::Trash => {
            rsx! { path { d: "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3" } }
        }
        IconName::Refresh => rsx! { path { d: "M20 11a8 8 0 1 0-2.3 5.7M20 4v7h-7" } },
        IconName::Reset => rsx! { path { d: "M4 11a8 8 0 1 1 2.3 5.7M4 4v7h7" } },
        IconName::Image => rsx! {
            {frame()}
            circle { cx: "9", cy: "10", r: "2" }
            path { d: "M21 16l-5-5-9 9" }
        },
        IconName::Plotter => rsx! { {frame()} path { d: "M3 9h18M12 9v5M9.5 14h5" } },
        IconName::Fit => rsx! { path { d: "M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5" } },
        IconName::Alert => rsx! {
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 8v5M12 16v.5" }
        },
        IconName::Code => rsx! { path { d: "M8 6l-6 6 6 6M16 6l6 6-6 6" } },
        IconName::GitBranch => rsx! {
            circle { cx: "6", cy: "6", r: "2" }
            circle { cx: "6", cy: "18", r: "2" }
            circle { cx: "18", cy: "7", r: "2" }
            path { d: "M6 8v8M18 9c0 5-7 3-11 7" }
        },
    }
}

#[component]
pub fn Icon(name: IconName, #[props(default)] class: String) -> Element {
    rsx! {
        svg {
            class: "i {class}",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            {children(name)}
        }
    }
}
