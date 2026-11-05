use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::Project;

/// Options for scanning folders for Plottery projects.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub max_depth: usize,
    pub follow_symlinks: bool,
    pub skip_dir_names: Vec<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            max_depth: 6,
            follow_symlinks: false,
            skip_dir_names: vec!["target".to_string(), "node_modules".to_string()],
        }
    }
}

/// Projects found by a scan plus any directories that could not be read or loaded.
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub projects: Vec<Project>,
    pub errors: Vec<(PathBuf, String)>,
}

fn is_hidden_or_skipped(entry: &walkdir::DirEntry, opts: &ScanOptions) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name.starts_with('.') || opts.skip_dir_names.iter().any(|s| *s == name)
}

fn find_project_config(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "plottery"))
}

/// Scans `root` for folders containing a `*.plottery` file. Project folders are not descended into.
pub fn scan_root(root: &Path, opts: &ScanOptions) -> ScanResult {
    let mut result = ScanResult::default();
    if !root.is_dir() {
        result.errors.push((
            root.to_path_buf(),
            "Folder does not exist or is not a directory".to_string(),
        ));
        return result;
    }

    let mut walker = WalkDir::new(root)
        .max_depth(opts.max_depth)
        .follow_links(opts.follow_symlinks)
        .into_iter()
        .filter_entry(|e| !is_hidden_or_skipped(e, opts));

    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                let path = err.path().unwrap_or(root).to_path_buf();
                result.errors.push((path, err.to_string()));
                continue;
            }
        };
        if !entry.file_type().is_dir() {
            continue;
        }
        let Some(config_path) = find_project_config(entry.path()) else {
            continue;
        };
        match Project::load_from_file(config_path) {
            Ok(project) => result.projects.push(project),
            Err(err) => result
                .errors
                .push((entry.path().to_path_buf(), err.to_string())),
        }
        walker.skip_current_dir();
    }
    result
}

/// Scans all `roots` and merges the results, dropping projects found more than once.
pub fn scan_roots(roots: &[PathBuf], opts: &ScanOptions) -> ScanResult {
    let mut merged = ScanResult::default();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for root in roots {
        let ScanResult { projects, errors } = scan_root(root, opts);
        merged.errors.extend(errors);
        for project in projects {
            let key = project
                .dir
                .canonicalize()
                .unwrap_or_else(|_| project.dir.clone());
            if seen.insert(key) {
                merged.projects.push(project);
            }
        }
    }
    merged
}
