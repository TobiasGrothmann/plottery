#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        path::{Path, PathBuf},
        time::{Duration, UNIX_EPOCH},
    };

    use anyhow::{Ok, Result};
    use resvg::tiny_skia;

    use crate::{
        cargo_project_template::generate_cargo_project_to_disk,
        project::{is_up_to_date, thumbnail_is_stale},
        validate_project_name, LibSource, Project, ProjectConfig, EDITOR_THUMBNAIL_SIZE,
    };
    use plottery_runtime::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};

    fn generate_temp_project(temp_dir: PathBuf, project_name: &str) -> Result<Project> {
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
    fn project_names_are_cargo_package_names() {
        for valid in ["spiral", "my_sketch", "_x", "ka-poster", "Lines2"] {
            assert!(validate_project_name(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "2lines",
            "-lines",
            "my sketch",
            "a/b",
            "linien_ä",
            "loop",
            "test",
            "plottery_lib",
        ] {
            assert!(validate_project_name(invalid).is_err(), "{invalid}");
        }
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
        let project = generate_temp_project(temp_dir.path().to_path_buf(), project_name)?;

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
        let project = generate_temp_project(temp_dir.path().to_path_buf(), project_name)?;
        assert!(project.exists());

        let release = true;

        // build
        project.build(release)?;

        // run to get params
        let params = project.run_get_params(release)?;
        assert_eq!(params.list.len(), 4);
        assert_eq!(params.list[3].section.as_deref(), Some("Circles"));

        // run
        let generated_layer = project.run(release, &params)?;
        assert!(!generated_layer.is_empty());

        // build again
        project.build(release)?;

        // test saving svg
        let temp_svg_path = temp_dir.path().join("temp.svg");
        project.write_svg(temp_svg_path.clone(), &generated_layer)?;
        assert!(temp_svg_path.exists());

        // test saving png
        let temp_png_path = temp_dir.path().join("temp.png");
        project.write_png(temp_png_path.clone(), &generated_layer, 512)?;
        assert!(temp_png_path.exists());

        Ok(())
    }

    /// All projects build into one shared target directory. Two projects with the same package
    /// name must still each run their own code there.
    #[test]
    fn same_package_name_in_shared_target_dir() -> Result<()> {
        let project_name: &str = "test_proj_same_name";
        let temp_dir_a = tempfile::tempdir()?;
        let temp_dir_b = tempfile::tempdir()?;
        let project_a = generate_temp_project(temp_dir_a.path().to_path_buf(), project_name)?;
        let project_b = generate_temp_project(temp_dir_b.path().to_path_buf(), project_name)?;
        assert_eq!(
            project_a.get_cargo_toml_name()?,
            project_b.get_cargo_toml_name()?
        );

        // project b draws one extra shape
        let generate_path = project_b.get_cargo_path()?.join("src/generate.rs");
        let generate_source = std::fs::read_to_string(&generate_path)?;
        let marker = "l.with_name(\"root\")";
        assert!(generate_source.contains(marker));
        std::fs::write(
            &generate_path,
            generate_source.replace(
                marker,
                "l.push(Circle::new_shape(V2::zero(), 1.0)); l.with_name(\"root\")",
            ),
        )?;

        let release = true;
        project_a.build(release)?;
        let params = project_a.run_get_params(release)?;
        let layer_a = project_a.run(release, &params)?;

        project_b.build(release)?;
        let layer_b = project_b.run(release, &params)?;
        assert_eq!(layer_b.len(), layer_a.len() + 1);

        // switching back must run a's code again, not b's
        project_a.build(release)?;
        let layer_a_again = project_a.run(release, &params)?;
        assert_eq!(layer_a_again.len(), layer_a.len());
        Ok(())
    }

    /// Previews of all projects share one cache folder, so the folder has to be part of the name.
    #[test]
    fn preview_file_name_tells_folders_apart() -> Result<()> {
        let parent_a = tempfile::tempdir()?;
        let parent_b = tempfile::tempdir()?;
        let file_name = |parent: &Path| -> Result<String> {
            let path =
                Project::new(parent.to_path_buf(), "same_name").get_editor_thumbnail_path()?;
            Ok(path.file_name().unwrap().to_string_lossy().to_string())
        };

        let name_a = file_name(parent_a.path())?;
        assert!(name_a.starts_with("same_name-") && name_a.ends_with(".png"));
        assert_eq!(name_a, file_name(parent_a.path())?);
        assert_ne!(name_a, file_name(parent_b.path())?);
        Ok(())
    }

    #[test]
    fn thumbnail_staleness_follows_the_layer() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let layer = dir.path().join("layer.plotl");
        let thumbnail = dir.path().join("thumbnail.png");
        let set_modified = |path: &Path, seconds: u64| -> Result<()> {
            File::options()
                .write(true)
                .open(path)?
                .set_modified(UNIX_EPOCH + Duration::from_secs(seconds))?;
            Ok(())
        };

        // nothing was drawn yet
        assert!(!thumbnail_is_stale(&layer, &thumbnail));

        std::fs::write(&layer, b"layer")?;
        set_modified(&layer, 1000)?;
        assert!(thumbnail_is_stale(&layer, &thumbnail));
        assert!(!is_up_to_date(&layer, &thumbnail));

        tiny_skia::Pixmap::new(EDITOR_THUMBNAIL_SIZE, 1)
            .unwrap()
            .save_png(&thumbnail)?;
        set_modified(&thumbnail, 2000)?;
        assert!(!thumbnail_is_stale(&layer, &thumbnail));
        assert!(is_up_to_date(&layer, &thumbnail));

        // a run or a git checkout rewrites the layer
        set_modified(&layer, 3000)?;
        assert!(thumbnail_is_stale(&layer, &thumbnail));
        assert!(!is_up_to_date(&layer, &thumbnail));
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

    #[test]
    fn saving_params_keeps_a_file_it_cannot_read() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let project = Project::new(temp_dir.path().to_path_buf(), "test");
        std::fs::create_dir_all(project.get_resource_dir())?;
        let params = ProjectParamsListWrapper::new(vec![ProjectParam::new(
            "size",
            ProjectParamValue::Float(1.0),
        )]);

        // no file: nothing saved yet, and saving creates it
        assert!(project.load_editor_params()?.0.list.is_empty());
        project.save_editor_params(&params)?;
        assert_eq!(project.load_editor_params()?.0.list.len(), 1);

        std::fs::write(project.get_editor_params_path(), [1, 2, 3])?;
        assert!(project.load_editor_params().is_err());
        assert!(project.save_editor_params(&params).is_err());
        assert_eq!(std::fs::read(project.get_editor_params_path())?, [1, 2, 3]);
        Ok(())
    }
}
