use std::{collections::BTreeSet, path::PathBuf};

use dioxus::prelude::*;

use crate::{model::folder_tree::FolderNode, util::format_svg};

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
            button { class: "img_button folder_tree_add_root",
                onclick: move |_| props.on_add_root.call(()),
                img { src: "{format_svg(include_bytes!(\"../../../public/icons/add.svg\"))}" }
                p { "Watch folder" }
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
        "folder_node_row folder_node_row_selected"
    } else {
        "folder_node_row"
    };
    let arrow = if is_expanded { "▾" } else { "▸" };
    let count = props.node.project_count();

    let select_path = path.clone();
    let toggle_path = path.clone();
    let remove_path = path.clone();

    rsx! {
        div { class: "FolderTreeNode",
            div { class: "{row_class}",
                style: "padding-left: {props.depth * 14 + 6}px;",
                title: "{path.to_string_lossy()}",
                onclick: move |_| props.on_select.call(select_path.clone()),
                div { class: "folder_node_arrow",
                    onclick: move |event| {
                        event.stop_propagation();
                        if has_children {
                            props.on_toggle_expand.call(toggle_path.clone());
                        }
                    },
                    if has_children { "{arrow}" }
                }
                p { class: "folder_node_label", "{props.node.label}" }
                p { class: "folder_node_count", "({count})" }
                if props.node.is_root {
                    button { class: "folder_node_remove",
                        title: "Stop watching this folder",
                        onclick: move |event| {
                            event.stop_propagation();
                            props.on_remove_root.call(remove_path.clone());
                        },
                        img { src: "{format_svg(include_bytes!(\"../../../public/icons/x.svg\"))}" }
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
}
