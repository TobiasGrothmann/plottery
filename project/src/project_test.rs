#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{LibSource, Project, ProjectConfig};

    #[test]
    fn save_project_config() -> anyhow::Result<()> {
        let project_file = ProjectConfig::new("project_name".to_string());
        let project_file_path = Path::new("test_project.plottery");

        project_file.save_to_file(&project_file_path)?;

        let result = std::panic::catch_unwind(|| {
            let loaded_project_file = ProjectConfig::new_from_file(&project_file_path).unwrap();
            assert_eq!(project_file, loaded_project_file);
        });

        std::fs::remove_file(project_file_path)?;
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn save_project() -> anyhow::Result<()> {
        let project_name: &str = "test_proj";
        let project_dir = Path::new(project_name);
        if project_dir.exists() {
            std::fs::remove_dir_all(project_dir)?;
        }

        let project = Project::new(
            Path::new(&".".to_string()).to_path_buf(),
            project_name.into(),
        );
        assert_eq!(project.exists(), false);

        let cargo_workspace = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        project.generate_to_disk(LibSource::Path {
            path: cargo_workspace.join("lib"),
        })?;

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
        let loaded_project = Project::load_from_file(project_config_path.to_path_buf())?;
        assert_eq!(project, loaded_project);

        // clean up
        std::fs::remove_dir_all(project_dir)?;

        Ok(())
    }

    #[test]
    fn build_and_run() -> anyhow::Result<()> {
        let mut project_path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        project_path.push("test/test_project/test_project.plottery");
        assert!(project_path.exists());

        let project = Project::load_from_file(project_path)?;
        assert!(project.exists());

        // build debug
        project.build(false)?;

        // run release
        let generated_layer = project.run(true)?;
        assert!(!generated_layer.is_empty());

        Ok(())
    }

    #[test]
    fn render() -> anyhow::Result<()> {
        let mut project_path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        project_path.push("test/test_project/test_project.plottery");
        assert!(project_path.exists());

        let project = Project::load_from_file(project_path)?;
        assert!(project.exists());

        project.build(true)?;

        let temp_dir = tempfile::tempdir()?;

        let temp_svg_path = temp_dir.path().join("temp.svg");
        project.write_svg(temp_svg_path.clone(), true)?;
        assert!(temp_svg_path.exists());

        let temp_png_path = temp_dir.path().join("temp.png");
        project.write_png(temp_png_path.clone(), true)?;
        assert!(temp_png_path.exists());

        Ok(())
    }
}
