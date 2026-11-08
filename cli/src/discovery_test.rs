#[cfg(test)]
mod tests {
    use std::fs;

    use crate::discovery::find_project_file;

    #[test]
    fn finds_file_in_parent_folder() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("art.plottery");
        fs::write(&file, "").unwrap();
        let nested = temp.path().join("art/src");
        fs::create_dir_all(&nested).unwrap();

        assert_eq!(find_project_file(&nested).unwrap(), file);
        assert_eq!(find_project_file(temp.path()).unwrap(), file);
    }

    #[test]
    fn several_files_are_an_error() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("a.plottery"), "").unwrap();
        fs::write(temp.path().join("b.plottery"), "").unwrap();

        let error = find_project_file(temp.path()).unwrap_err().to_string();
        assert!(error.starts_with("Several projects in"), "{error}");
        assert!(error.contains("a.plottery") && error.contains("b.plottery"));
    }

    #[test]
    fn no_file_is_an_error() {
        let temp = tempfile::tempdir().unwrap();
        let error = find_project_file(temp.path()).unwrap_err().to_string();
        assert!(error.starts_with("No .plottery file in"), "{error}");
    }
}
