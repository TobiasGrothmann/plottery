use std::{path::PathBuf, rc::Rc};

use dioxus::{
    html::{geometry::PixelsVector2D, MountedData, ScrollBehavior},
    prelude::*,
};
use dioxus_logger::tracing;
use dioxus_router::hooks::use_navigator;
use plottery_project::{scan_roots, Project, ScanOptions, ScanResult};

use crate::{
    components::{loading_spinner::Loading, navigation::Navigation},
    model::{
        app_state::AppState,
        folder_tree::{build_tree, find_closest_in_forest},
    },
    router::browser::{folder_tree::FolderTree, project_grid::ProjectGrid},
    routes::Route,
    util::{editor_route, format_svg},
};

/// Scroll offset of the project grid, kept across navigation so going back lands where you left.
static SCROLL_TOP: GlobalSignal<f64> = Signal::global(|| 0.0);

fn pick_root_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Pick a folder to watch for Plottery projects")
        .pick_folder()
}

#[component]
pub fn Browser() -> Element {
    let mut app_state = use_signal(AppState::load_or_default);
    // Only the roots trigger a rescan; selection and expansion changes must not.
    let roots = use_memo(move || app_state.read().roots.clone());
    let mut scan = use_resource(move || {
        let roots = roots();
        async move {
            tokio::task::spawn_blocking(move || scan_roots(&roots, &ScanOptions::default()))
                .await
                .unwrap_or_else(|err| ScanResult {
                    projects: vec![],
                    errors: vec![(PathBuf::new(), format!("Scan failed: {err}"))],
                })
        }
    });
    // Render missing or outdated thumbnails after each scan, then refresh the cards.
    let mut thumbnail_generation = use_signal(|| 0u32);
    let _thumbnails = use_resource(move || {
        let projects: Vec<Project> = scan
            .read()
            .as_ref()
            .map(|r| r.projects.clone())
            .unwrap_or_default();
        async move {
            let stale: Vec<Project> = projects
                .into_iter()
                .filter(Project::editor_thumbnail_is_stale)
                .collect();
            if stale.is_empty() {
                return;
            }
            let rendered = tokio::task::spawn_blocking(move || {
                stale
                    .iter()
                    .filter(|p| match p.update_editor_thumbnail() {
                        Ok(()) => true,
                        Err(err) => {
                            tracing::error!(
                                "Failed to render thumbnail for {}: {err}",
                                p.dir.display()
                            );
                            false
                        }
                    })
                    .count()
            })
            .await
            .unwrap_or(0);
            if rendered > 0 {
                thumbnail_generation += 1;
            }
        }
    });
    use_effect(move || app_state.read().save());

    // Restore the previous scroll position once the first scan has rendered the cards.
    let mut main_element: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let mut scroll_restored = use_signal(|| false);
    use_effect(move || {
        let scanned = scan.read().is_some();
        let Some(element) = main_element() else {
            return;
        };
        if !scanned || scroll_restored() {
            return;
        }
        scroll_restored.set(true);
        let top = *SCROLL_TOP.peek();
        if top > 0.0 {
            spawn(async move {
                let _ = element
                    .scroll(PixelsVector2D::new(0.0, top), ScrollBehavior::Instant)
                    .await;
            });
        }
    });

    let nav = use_navigator();
    let mut add_root = move || {
        if let Some(folder) = pick_root_folder() {
            let mut state = app_state.write();
            state.add_root(folder.clone());
            state.selected_folder = Some(folder);
        }
    };

    let roots_now = roots();
    let scan_guard = scan.read();
    let first_load = scan_guard.is_none();
    let is_scanning = matches!(*scan.state().read(), UseResourceState::Pending);
    let (tree, errors) = match scan_guard.as_ref() {
        Some(result) => (
            build_tree(&roots_now, &result.projects),
            result.errors.clone(),
        ),
        None => (vec![], vec![]),
    };

    let selected_folder = app_state.read().selected_folder.clone();
    let selected_node = selected_folder
        .as_deref()
        .and_then(|p| find_closest_in_forest(&tree, p))
        .or(tree.first())
        .cloned();
    let selected_path = selected_node.as_ref().map(|n| n.path.clone());
    let expanded = app_state.read().expanded_folders.clone();

    rsx! {
        style { { include_str!("./browser.css") } }
        Navigation {
            page_name: "Projects",
            body: rsx! {
                button { class: "img_button",
                    title: "Rescan watched folders",
                    onclick: move |_| scan.restart(),
                    img { src: "{format_svg(include_bytes!(\"../../../public/icons/refresh.svg\"))}" }
                }
                button { class: "img_button",
                    title: "Create project",
                    onclick: move |_| {
                        nav.push(Route::ProjectAdd {});
                    },
                    img { src: "{format_svg(include_bytes!(\"../../../public/icons/add.svg\"))}" }
                }
                button { class: "img_button",
                    title: "Remote",
                    onclick: move |_| {
                        nav.push(Route::Remote {});
                    },
                    img { src: "{format_svg(include_bytes!(\"../../../public/icons/remote.svg\"))}" }
                }
            }
        }
        div { class: "Browser",
            if first_load {
                Loading {}
            }
            div { class: "browser_sidebar",
                FolderTree {
                    nodes: tree.clone(),
                    selected: selected_path.clone(),
                    expanded,
                    on_select: move |path: PathBuf| {
                        app_state.write().selected_folder = Some(path);
                    },
                    on_toggle_expand: move |path: PathBuf| {
                        let mut state = app_state.write();
                        if !state.expanded_folders.remove(&path) {
                            state.expanded_folders.insert(path);
                        }
                    },
                    on_add_root: move |_| add_root(),
                    on_remove_root: move |path: PathBuf| {
                        app_state.write().remove_root(&path);
                    },
                }
                if is_scanning && !first_load {
                    p { class: "browser_scanning", "scanning…" }
                }
                if !errors.is_empty() {
                    details { class: "browser_scan_errors",
                        summary { "{errors.len()} scan problems" }
                        for (path, message) in errors.iter() {
                            div { class: "err_box browser_scan_error",
                                p { class: "browser_scan_error_path", "{path.to_string_lossy()}" }
                                p { "{message}" }
                            }
                        }
                    }
                }
            }
            div { class: "browser_main",
                onmounted: move |event| main_element.set(Some(event.data())),
                onscroll: move |event| *SCROLL_TOP.write() = event.data().scroll_top(),
                if roots_now.is_empty() {
                    div { class: "browser_empty",
                        p { "Add a folder to watch for Plottery projects." }
                        button { class: "img_button",
                            onclick: move |_| add_root(),
                            img { src: "{format_svg(include_bytes!(\"../../../public/icons/folder_open.svg\"))}" }
                            p { "Add folder" }
                        }
                    }
                } else if let Some(node) = selected_node {
                    ProjectGrid {
                        node,
                        thumbnail_generation: thumbnail_generation(),
                        on_open: move |project: Project| {
                            nav.push(editor_route(&project));
                        },
                    }
                }
            }
        }
    }
}
