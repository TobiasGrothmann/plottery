#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use anyhow::{Ok, Result};

    use crate::{
        cargo_project_template::generate_cargo_project_to_disk, LibSource, Project, ProjectConfig,
    };

    fn genrate_temp_project(temp_dir: PathBuf, project_name: &str) -> Result<Project> {
        // project dir should not exist
        let project_dir = temp_dir.join(project_name);
        assert!(!project_dir.exists());

        // create project object
        let project = Project::new(temp_dir, project_name);
        assert!(!project.exists());

        // get path to plottery libraries
        let cargo_workspace_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        assert!(cargo_workspace_path.exists());
        println!("workspace path: {:?}", cargo_workspace_path);

        // generate to disk and check
        project.generate_to_disk(
            LibSource::Path {
                path: cargo_workspace_path,
            },
            true,
        )?;
        assert!(project.exists());

        // check if project dir was generated
        assert!(project_dir.exists());
        Ok(project)
    }

    #[test]
    fn save_project_config() -> Result<()> {
        let project_file = ProjectConfig::new("project_name");
        let project_file_path = Path::new("test_project.plottery");

        project_file.save_to_file(project_file_path)?;

        let result = std::panic::catch_unwind(|| {
            let loaded_project_file = ProjectConfig::new_from_file(project_file_path).unwrap();
            assert_eq!(project_file, loaded_project_file);
        });

        std::fs::remove_file(project_file_path)?;
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn generate_project() -> Result<()> {
        let project_name: &str = "test_proj";
        let temp_dir = tempfile::tempdir()?;
        let project = genrate_temp_project(temp_dir.path().to_path_buf(), project_name)?;

        // check if dir test_project/resources exists
        let resource_dir = project.dir.join("resources");
        println!("{:?}", resource_dir);
        assert!(resource_dir.exists());

        // check if file test_project/test_project.plottery exists
        let project_config_path = project.dir.join(format!("{}.plottery", project_name));
        assert!(project_config_path.exists());

        // test loading the project
        let loaded_project = Project::load_from_file(project_config_path.to_path_buf())?;
        assert_eq!(project, loaded_project);

        Ok(())
    }

    #[test]
    fn build_and_run() -> Result<()> {
        let project_name: &str = "test_proj";
        let temp_dir = tempfile::tempdir()?;
        let project = genrate_temp_project(temp_dir.path().to_path_buf(), project_name)?;
        assert!(project.exists());

        let release = true;

        // build
        project.build(release)?;

        // run to get params
        let params = project.run_get_params(release)?;
        assert!(params.list.len() == 2);

        // run
        let generated_layer = project.run(release, &params)?;
        assert!(!generated_layer.is_empty());

        // build debug
        project.build(release)?;

        // test saving svg
        let temp_svg_path = temp_dir.path().join("temp.svg");
        project.write_svg(temp_svg_path.clone(), release, &params)?;
        assert!(temp_svg_path.exists());

        // test saving png
        let temp_png_path = temp_dir.path().join("temp.png");
        project.write_png(temp_png_path.clone(), release, &params)?;
        assert!(temp_png_path.exists());

        Ok(())
    }

    #[test]
    fn test_embed_project_template_generation() -> Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        generate_cargo_project_to_disk(
            temp_dir.path().to_path_buf(),
            "a_test_project",
            LibSource::CratesIO,
        )?;

        assert!(temp_dir.path().join("Cargo.toml").exists());
        assert!(temp_dir.path().join("src").exists());
        Ok(())
    }
}
