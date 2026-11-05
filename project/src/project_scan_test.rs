#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{scan_root, scan_roots, ProjectConfig, ScanOptions};

    fn fake_project(parent: &Path, name: &str) -> PathBuf {
        let dir = parent.join(name);
        std::fs::create_dir_all(dir.join(name)).unwrap();
        std::fs::write(dir.join(name).join("Cargo.toml"), "[package]\n").unwrap();
        ProjectConfig::new(name)
            .save_to_file(&dir.join(format!("{name}.plottery")))
            .unwrap();
        dir
    }

    fn names(result: &crate::ScanResult) -> Vec<String> {
        let mut names: Vec<String> = result
            .projects
            .iter()
            .map(|p| p.config.name.clone())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn finds_projects_at_different_depths() {
        let tmp = tempfile::tempdir().unwrap();
        fake_project(tmp.path(), "shallow");
        fake_project(&tmp.path().join("a/b"), "deep");

        let result = scan_root(tmp.path(), &ScanOptions::default());
        assert_eq!(names(&result), vec!["deep", "shallow"]);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn does_not_descend_into_project_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = fake_project(tmp.path(), "outer");
        fake_project(&dir.join("resources"), "decoy");

        let result = scan_root(tmp.path(), &ScanOptions::default());
        assert_eq!(names(&result), vec!["outer"]);
    }

    #[test]
    fn skips_hidden_and_build_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        fake_project(&tmp.path().join(".git"), "hidden");
        fake_project(&tmp.path().join("target"), "built");
        fake_project(&tmp.path().join("node_modules"), "node");
        fake_project(tmp.path(), "visible");

        let result = scan_root(tmp.path(), &ScanOptions::default());
        assert_eq!(names(&result), vec!["visible"]);
    }

    #[test]
    fn respects_max_depth() {
        let tmp = tempfile::tempdir().unwrap();
        fake_project(&tmp.path().join("a"), "depth2");
        fake_project(&tmp.path().join("a/b/c"), "depth4");

        let opts = ScanOptions {
            max_depth: 2,
            ..Default::default()
        };
        let result = scan_root(tmp.path(), &opts);
        assert_eq!(names(&result), vec!["depth2"]);
    }

    #[test]
    fn missing_root_is_an_error() {
        let result = scan_root(Path::new("/definitely/not/here"), &ScanOptions::default());
        assert!(result.projects.is_empty());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn config_without_cargo_package_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("broken");
        std::fs::create_dir_all(&dir).unwrap();
        ProjectConfig::new("broken")
            .save_to_file(&dir.join("broken.plottery"))
            .unwrap();

        let result = scan_root(tmp.path(), &ScanOptions::default());
        assert!(result.projects.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].0, dir);
    }

    #[test]
    fn nested_roots_are_deduplicated() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub");
        fake_project(&sub, "p");

        let roots = vec![tmp.path().to_path_buf(), sub];
        let result = scan_roots(&roots, &ScanOptions::default());
        assert_eq!(names(&result), vec!["p"]);
    }
}
