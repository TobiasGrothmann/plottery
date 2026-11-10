use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use plottery_project::Project;

pub const LABEL_SEPARATOR: &str = " › ";

/// A folder in the sidebar tree. Single-child chains are compacted, so `path` is the deepest
/// directory of the chain and `label` lists all its segments.
#[derive(Debug, Clone)]
pub struct FolderNode {
    pub path: PathBuf,
    pub label: String,
    pub is_root: bool,
    pub children: Vec<FolderNode>,
    /// Projects whose parent directory is exactly `path`.
    pub projects: Vec<Project>,
}

impl PartialEq for FolderNode {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.label == other.label
            && self.is_root == other.is_root
            && self.children == other.children
            && self.projects.len() == other.projects.len()
            && self
                .projects
                .iter()
                .zip(&other.projects)
                .all(|(a, b)| a.get_project_config_path() == b.get_project_config_path())
    }
}

/// Projects of one subfolder when listing a node and all of its descendants.
#[derive(Debug, Clone, PartialEq)]
pub struct Section<'a> {
    /// Relative to the listed node, `""` for the node's own projects.
    pub label: String,
    pub path: PathBuf,
    pub projects: Vec<&'a Project>,
}

impl FolderNode {
    fn new(path: PathBuf, label: String, is_root: bool) -> Self {
        Self {
            path,
            label,
            is_root,
            children: Vec::new(),
            projects: Vec::new(),
        }
    }

    pub fn project_count(&self) -> usize {
        self.projects.len()
            + self
                .children
                .iter()
                .map(FolderNode::project_count)
                .sum::<usize>()
    }

    /// A copy with only the kept projects, or `None` if no project is kept.
    pub fn filtered(&self, keep: impl Fn(&Project) -> bool + Copy) -> Option<FolderNode> {
        let projects: Vec<Project> = self.projects.iter().filter(|p| keep(p)).cloned().collect();
        let children: Vec<FolderNode> = self
            .children
            .iter()
            .filter_map(|c| c.filtered(keep))
            .collect();
        (!projects.is_empty() || !children.is_empty()).then(|| FolderNode {
            path: self.path.clone(),
            label: self.label.clone(),
            is_root: self.is_root,
            children,
            projects,
        })
    }

    #[cfg(test)]
    pub fn find(&self, path: &Path) -> Option<&FolderNode> {
        if self.path == path {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(path))
    }

    /// The deepest node whose path is a prefix of `path`.
    pub fn find_closest(&self, path: &Path) -> Option<&FolderNode> {
        if !path.starts_with(&self.path) {
            return None;
        }
        self.children
            .iter()
            .find_map(|c| c.find_closest(path))
            .or(Some(self))
    }

    pub fn sections(&self) -> Vec<Section<'_>> {
        let mut sections = Vec::new();
        self.collect_sections(&self.path, &mut sections);
        sections
    }

    fn collect_sections<'a>(&'a self, base: &Path, out: &mut Vec<Section<'a>>) {
        if !self.projects.is_empty() {
            let label = self
                .path
                .strip_prefix(base)
                .ok()
                .map(|rel| {
                    rel.components()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(LABEL_SEPARATOR)
                })
                .unwrap_or_default();
            out.push(Section {
                label,
                path: self.path.clone(),
                projects: self.projects.iter().collect(),
            });
        }
        for child in &self.children {
            child.collect_sections(base, out);
        }
    }

    fn get_or_insert_child(&mut self, name: &str) -> &mut FolderNode {
        let index = match self.children.iter().position(|c| c.label == name) {
            Some(index) => index,
            None => {
                let path = self.path.join(name);
                self.children
                    .push(FolderNode::new(path, name.to_string(), false));
                self.children.len() - 1
            }
        };
        &mut self.children[index]
    }

    fn compact(&mut self) {
        for child in &mut self.children {
            child.compact();
        }
        while !self.is_root && self.projects.is_empty() && self.children.len() == 1 {
            let child = self.children.remove(0);
            self.label = format!("{}{LABEL_SEPARATOR}{}", self.label, child.label);
            self.path = child.path;
            self.children = child.children;
            self.projects = child.projects;
        }
    }

    fn sort(&mut self) {
        self.children.sort_by_cached_key(|c| c.label.to_lowercase());
        self.projects.sort_by(|a, b| {
            b.config
                .last_modified_date
                .cmp(&a.config.last_modified_date)
        });
        for child in &mut self.children {
            child.sort();
        }
    }
}

fn root_label(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string())
}

/// Builds one tree per root. Each project goes to the longest root containing it; projects
/// outside every root are dropped.
pub fn build_tree(roots: &[PathBuf], projects: &[Project]) -> Vec<FolderNode> {
    let mut forest: BTreeMap<usize, FolderNode> = roots
        .iter()
        .enumerate()
        .map(|(i, root)| (i, FolderNode::new(root.clone(), root_label(root), true)))
        .collect();

    for project in projects {
        let Some(parent) = project.dir.parent() else {
            continue;
        };
        let Some((root_index, root)) = roots
            .iter()
            .enumerate()
            .filter(|(_, root)| parent.starts_with(root))
            .max_by_key(|(_, root)| root.components().count())
        else {
            continue;
        };
        let Ok(relative) = parent.strip_prefix(root) else {
            continue;
        };
        let mut node = forest.get_mut(&root_index).unwrap();
        for component in relative.components() {
            node = node.get_or_insert_child(&component.as_os_str().to_string_lossy());
        }
        node.projects.push(project.clone());
    }

    forest
        .into_values()
        .map(|mut node| {
            node.compact();
            node.sort();
            node
        })
        .collect()
}

#[cfg(test)]
pub fn find_in_forest<'a>(forest: &'a [FolderNode], path: &Path) -> Option<&'a FolderNode> {
    forest.iter().find_map(|n| n.find(path))
}

pub fn find_closest_in_forest<'a>(forest: &'a [FolderNode], path: &Path) -> Option<&'a FolderNode> {
    forest
        .iter()
        .filter_map(|n| n.find_closest(path))
        .max_by_key(|n| n.path.components().count())
}
