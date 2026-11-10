#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use plottery_project::{Project, ProjectConfig};

    use crate::model::folder_tree::{build_tree, find_closest_in_forest, find_in_forest};

    fn project(dir: &str) -> Project {
        let dir = PathBuf::from(dir);
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        Project {
            config: ProjectConfig::new(&name),
            dir,
        }
    }

    fn roots(paths: &[&str]) -> Vec<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn empty() {
        assert!(build_tree(&[], &[]).is_empty());
        let forest = build_tree(&roots(&["/r"]), &[]);
        assert_eq!(forest.len(), 1);
        assert_eq!(forest[0].label, "r");
        assert!(forest[0].is_root);
        assert_eq!(forest[0].project_count(), 0);
    }

    #[test]
    fn flat_root() {
        let forest = build_tree(&roots(&["/r"]), &[project("/r/a"), project("/r/b")]);
        assert!(forest[0].children.is_empty());
        assert_eq!(forest[0].projects.len(), 2);
        assert_eq!(forest[0].project_count(), 2);
    }

    #[test]
    fn compacts_single_child_chain() {
        let forest = build_tree(&roots(&["/r"]), &[project("/r/a/b/c/p")]);
        let root = &forest[0];
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].label, "a › b › c");
        assert_eq!(root.children[0].path, Path::new("/r/a/b/c"));
        assert_eq!(root.children[0].projects.len(), 1);
    }

    #[test]
    fn no_compaction_with_siblings() {
        let forest = build_tree(
            &roots(&["/r"]),
            &[project("/r/a/b/p1"), project("/r/a/c/p2")],
        );
        let a = &forest[0].children[0];
        assert_eq!(a.label, "a");
        let labels: Vec<&str> = a.children.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["b", "c"]);
    }

    #[test]
    fn no_compaction_when_intermediate_has_project() {
        let forest = build_tree(&roots(&["/r"]), &[project("/r/a/p1"), project("/r/a/b/p2")]);
        let a = &forest[0].children[0];
        assert_eq!(a.label, "a");
        assert_eq!(a.projects.len(), 1);
        assert_eq!(a.children[0].label, "b");
    }

    #[test]
    fn nested_roots_assign_to_longest() {
        let forest = build_tree(
            &roots(&["/r", "/r/sub"]),
            &[project("/r/p1"), project("/r/sub/p2")],
        );
        assert_eq!(forest.len(), 2);
        assert_eq!(forest[0].projects.len(), 1);
        assert!(forest[0].children.is_empty());
        assert_eq!(forest[1].projects.len(), 1);
    }

    #[test]
    fn project_outside_roots_is_ignored() {
        let forest = build_tree(&roots(&["/r"]), &[project("/elsewhere/p")]);
        assert_eq!(forest[0].project_count(), 0);
    }

    #[test]
    fn sections_labels() {
        let forest = build_tree(
            &roots(&["/r"]),
            &[project("/r/p0"), project("/r/a/p1"), project("/r/a/b/c/p2")],
        );
        let sections = forest[0].sections();
        let labels: Vec<&str> = sections.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, vec!["", "a", "a › b › c"]);

        let a = find_in_forest(&forest, Path::new("/r/a")).unwrap();
        let sections = a.sections();
        let labels: Vec<&str> = sections.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, vec!["", "b › c"]);
    }

    #[test]
    fn filtered_keeps_matches_and_prunes_empty_folders() {
        let forest = build_tree(
            &roots(&["/r"]),
            &[
                project("/r/keep_a"),
                project("/r/drop_a"),
                project("/r/a/keep_b"),
                project("/r/a/drop_b"),
                project("/r/c/drop_c"),
            ],
        );
        let filtered = forest[0]
            .filtered(|p| p.config.name.starts_with("keep"))
            .unwrap();
        assert!(filtered.is_root);
        assert_eq!(filtered.project_count(), 2);
        assert_eq!(filtered.projects[0].config.name, "keep_a");
        let children: Vec<&str> = filtered.children.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(children, vec!["a"]);
        assert_eq!(filtered.children[0].projects[0].config.name, "keep_b");
    }

    #[test]
    fn filtered_without_hits_is_none() {
        let forest = build_tree(&roots(&["/r"]), &[project("/r/p"), project("/r/a/q")]);
        assert!(forest[0].filtered(|_| false).is_none());
        let empty = build_tree(&roots(&["/r"]), &[]);
        assert!(empty[0].filtered(|_| true).is_none());
    }

    #[test]
    fn find_closest_falls_back_to_ancestor() {
        let forest = build_tree(&roots(&["/r"]), &[project("/r/a/b/p")]);
        let node = find_closest_in_forest(&forest, Path::new("/r/a/b/gone")).unwrap();
        assert_eq!(node.path, Path::new("/r/a/b"));
        let node = find_closest_in_forest(&forest, Path::new("/r/zzz")).unwrap();
        assert!(node.is_root);
        assert!(find_closest_in_forest(&forest, Path::new("/other")).is_none());
    }
}
