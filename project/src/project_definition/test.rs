#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{project_definition::ProjectConfig, Project};

    #[test]
    fn save_and_load_project() {
        let project_file = ProjectConfig::new("project_name".to_string());
        let project_file_path = Path::new("test_project.plottery");

        project_file.save_to_file(&project_file_path).unwrap();

        let result = std::panic::catch_unwind(|| {
            let loaded_project_file = ProjectConfig::new_from_file(&project_file_path).unwrap();
            assert_eq!(project_file, loaded_project_file);
        });

        std::fs::remove_file(project_file_path).unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn save_project() {
        let project_name: &str = "test_proj";

        let project = Project::new(Path::new(".".into()).to_path_buf(), project_name.into());
        let result = project.save();
        assert!(result.is_ok());

        // check if dir test_project exists
        let project_dir = Path::new(project_name);
        assert!(project_dir.exists());

        // check if dir test_project/resources exists
        let resource_dir = Path::new("test_proj/resources");
        assert!(resource_dir.exists());

        // check if file test_project/test_project.plottery exists
        let project_config_path = Path::new("test_proj/test_proj.plottery");
        assert!(project_config_path.exists());

        // test loading the project
        let result = std::panic::catch_unwind(|| {
            let loaded_project = Project::new_from_file(project_config_path.to_path_buf()).unwrap();
            assert_eq!(project, loaded_project);
        });

        // clean up
        std::fs::remove_dir_all(project_dir).unwrap();
        assert!(result.is_ok());
    }
}
