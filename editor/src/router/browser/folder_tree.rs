use std::{collections::BTreeSet, path::PathBuf};

use dioxus::prelude::*;

use crate::{
    components::{
        icon::{Icon, IconName},
        ui::{IconButton, Size},
    },
    model::folder_tree::FolderNode,
};

#[derive(PartialEq, Props, Clone)]
pub struct FolderTreeProps {
    pub nodes: Vec<FolderNode>,
    pub selected: Option<PathBuf>,
    pub expanded: BTreeSet<PathBuf>,
    pub on_select: EventHandler<PathBuf>,
    pub on_toggle_expand: EventHandler<PathBuf>,
    pub on_add_root: EventHandler<()>,
    pub on_remove_root: EventHandler<PathBuf>,
}

#[component]
pub fn FolderTree(props: FolderTreeProps) -> Element {
    rsx! {
        style { { include_str!("./folder_tree.css") } }
        div { class: "FolderTree",
            div { class: "folder_tree_head",
                span { class: "caps", "Watched folders" }
                IconButton {
                    icon: IconName::Plus,
                    label: "Watch a folder",
                    size: Size::Sm,
                    onclick: move |_| props.on_add_root.call(()),
                }
            }
            div { class: "folder_tree_rows", role: "tree",
                for node in props.nodes.iter() {
                    FolderTreeNode {
                        node: node.clone(),
                        depth: 0,
                        selected: props.selected.clone(),
                        expanded: props.expanded.clone(),
                        on_select: props.on_select,
                        on_toggle_expand: props.on_toggle_expand,
                        on_remove_root: props.on_remove_root,
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Props, Clone)]
struct FolderTreeNodeProps {
    node: FolderNode,
    depth: usize,
    selected: Option<PathBuf>,
    expanded: BTreeSet<PathBuf>,
    on_select: EventHandler<PathBuf>,
    on_toggle_expand: EventHandler<PathBuf>,
    on_remove_root: EventHandler<PathBuf>,
}

#[component]
fn FolderTreeNode(props: FolderTreeNodeProps) -> Element {
    let path = props.node.path.clone();
    let is_selected = props.selected.as_ref() == Some(&path);
    let is_expanded = props.expanded.contains(&path);
    let has_children = !props.node.children.is_empty();
    let row_class = if is_selected {
        "tree-row sel"
    } else {
        "tree-row"
    };
    let count = props.node.project_count();

    let select_path = path.clone();
    let key_path = path.clone();
    let toggle_path = path.clone();
    let remove_path = path.clone();

    rsx! {
        div {
            class: "{row_class}",
            role: "treeitem",
            tabindex: "0",
            "aria-selected": if is_selected { "true" } else { "false" },
            "aria-expanded": if has_children { if is_expanded { "true" } else { "false" } },
            style: "padding-left: {props.depth * 16 + 4}px;",
            title: "{path.to_string_lossy()}",
            onclick: move |_| props.on_select.call(select_path.clone()),
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    props.on_select.call(key_path.clone());
                }
            },
            span {
                class: "folder_node_arrow",
                onclick: move |event| {
                    event.stop_propagation();
                    if has_children {
                        props.on_toggle_expand.call(toggle_path.clone());
                    }
                },
                if has_children {
                    Icon {
                        name: IconName::ChevronDown,
                        class: if is_expanded { "" } else { "rot-90" },
                    }
                }
            }
            span { class: "folder_node_label", "{props.node.label}" }
            span { class: "count", "{count}" }
            if props.node.is_root {
                button {
                    r#type: "button",
                    class: "folder_node_remove",
                    title: "Stop watching this folder",
                    "aria-label": "Stop watching {props.node.label}",
                    onclick: move |event| {
                        event.stop_propagation();
                        props.on_remove_root.call(remove_path.clone());
                    },
                    Icon { name: IconName::X }
                }
            }
        }
        if has_children && is_expanded {
            for child in props.node.children.iter() {
                FolderTreeNode {
                    node: child.clone(),
                    depth: props.depth + 1,
                    selected: props.selected.clone(),
                    expanded: props.expanded.clone(),
                    on_select: props.on_select,
                    on_toggle_expand: props.on_toggle_expand,
                    on_remove_root: props.on_remove_root,
                }
            }
        }
    }
}
