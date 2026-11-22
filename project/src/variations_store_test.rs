#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::Utc;
    use plottery_lib::{Circle, Layer, V2};
    use plottery_runtime::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};

    use crate::{Project, ProjectConfig, VariationsMeta};

    fn project(dir: &Path) -> Project {
        Project {
            config: ProjectConfig::new("sketch"),
            dir: dir.to_path_buf(),
        }
    }

    fn sets(count: i32) -> Vec<ProjectParamsListWrapper> {
        (0..count)
            .map(|i| {
                ProjectParamsListWrapper::new(vec![ProjectParam::new(
                    "count",
                    ProjectParamValue::IntRanged {
                        val: i,
                        min: 0,
                        max: 100,
                    },
                )])
            })
            .collect()
    }

    fn meta() -> VariationsMeta {
        VariationsMeta {
            created: Utc::now(),
            params: vec!["count".to_string()],
        }
    }

    fn layer() -> Layer {
        let mut layer = Layer::new();
        layer.push(Circle::new(V2::new(1.0, 1.0), 1.0));
        layer
    }

    fn rolls(project: &Project) -> Vec<(usize, bool, String)> {
        project
            .load_variations()
            .unwrap()
            .unwrap()
            .rolls
            .into_iter()
            .map(|roll| (roll.index, roll.done, roll.values[0].1.clone()))
            .collect()
    }

    #[test]
    fn done_comes_from_the_png_and_a_new_set_replaces_only_roll_files() {
        let tmp = tempfile::tempdir().unwrap();
        let project = project(tmp.path());
        assert!(project.load_variations().unwrap().is_none());

        project.start_variations(&meta(), &sets(3)).unwrap();
        project.save_roll_output(0, &layer()).unwrap();
        project.save_roll_output(2, &layer()).unwrap();
        assert_eq!(
            rolls(&project),
            vec![
                (0, true, "0".to_string()),
                (1, false, "1".to_string()),
                (2, true, "2".to_string())
            ]
        );

        let notes = project.get_variations_dir().join("notes.txt");
        std::fs::write(&notes, "mine").unwrap();
        project.start_variations(&meta(), &sets(2)).unwrap();
        assert_eq!(
            rolls(&project),
            vec![(0, false, "0".to_string()), (1, false, "1".to_string())]
        );
        assert!(notes.exists());
        assert!(!project.get_roll_path(2, "plotp").exists());
        assert!(!project.get_roll_path(2, "png").exists());
        assert!(project.get_variations_dir().join(".gitignore").exists());
    }

    #[test]
    fn removing_unfinished_rolls_keeps_the_done_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let project = project(tmp.path());
        project.start_variations(&meta(), &sets(4)).unwrap();
        project.save_roll_output(1, &layer()).unwrap();
        project.save_roll_output(3, &layer()).unwrap();
        project.remove_unfinished_rolls().unwrap();
        let indices: Vec<usize> = rolls(&project).into_iter().map(|roll| roll.0).collect();
        assert_eq!(indices, vec![1, 3]);
        assert_eq!(project.load_roll_params(3).unwrap().list, sets(4)[3].list);
        assert!(project.load_roll_params(2).is_err());
    }
}
