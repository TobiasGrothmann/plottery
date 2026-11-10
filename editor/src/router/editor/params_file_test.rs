#[cfg(test)]
mod test_params_file {
    use plottery_project::{
        project_params_list_wrapper::ProjectParamsListWrapper, Project, ProjectParam,
        ProjectParamValue,
    };
    use tempfile::TempDir;

    use crate::router::editor::params_file::ParamsFile;

    fn params(size: f32) -> ProjectParamsListWrapper {
        ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "size",
            ProjectParamValue::Float(size),
        )])
    }

    fn size(params: &ProjectParamsListWrapper) -> f32 {
        match params.list[0].value {
            ProjectParamValue::Float(size) => size,
            _ => panic!("not a float"),
        }
    }

    fn temp_project() -> (TempDir, Project) {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new(temp_dir.path().to_path_buf(), "test");
        std::fs::create_dir_all(project.get_resource_dir()).unwrap();
        (temp_dir, project)
    }

    #[test]
    fn own_save_is_not_external() {
        let (_temp_dir, project) = temp_project();
        let params_file = ParamsFile::new(project);
        params_file.save(&params(1.0)).unwrap();
        assert!(params_file.read_external().unwrap().is_none());
    }

    #[test]
    fn cli_write_is_external_once() {
        let (_temp_dir, project) = temp_project();
        let params_file = ParamsFile::new(project.clone());
        params_file.save(&params(1.0)).unwrap();

        project.save_editor_params(&params(2.0)).unwrap();
        let changed = params_file.read_external().unwrap().unwrap();
        assert_eq!(size(&changed), 2.0);
        assert!(params_file.read_external().unwrap().is_none());

        // saving what was just read is no new write
        params_file.save(&changed).unwrap();
        assert!(params_file.read_external().unwrap().is_none());
    }

    #[test]
    fn broken_file_errors_and_next_write_is_seen() {
        let (_temp_dir, project) = temp_project();
        let params_file = ParamsFile::new(project.clone());
        params_file.save(&params(1.0)).unwrap();

        std::fs::write(project.get_editor_params_path(), [1, 2, 3]).unwrap();
        assert!(params_file.read_external().is_err());

        project.save_editor_params(&params(3.0)).unwrap();
        let changed = params_file.read_external().unwrap().unwrap();
        assert_eq!(size(&changed), 3.0);
    }
}
