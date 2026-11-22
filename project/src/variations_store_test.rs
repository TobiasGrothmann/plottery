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

    fn rolls(project: &Project) -> Vec<(usize, String)> {
        project
            .load_variations()
            .unwrap()
            .unwrap()
            .rolls
            .into_iter()
            .map(|roll| (roll.index, roll.values[0].1.clone()))
            .collect()
    }

    #[test]
    fn rolls_are_the_saved_ones_and_a_new_set_removes_only_roll_files() {
        let tmp = tempfile::tempdir().unwrap();
        let project = project(tmp.path());
        assert!(project.load_variations().unwrap().is_none());

        let sets = sets(3);
        project.start_variations(&meta()).unwrap();
        project.save_roll(2, &sets[2], &layer()).unwrap();
        project.save_roll(0, &sets[0], &layer()).unwrap();
        // a roll that died while it was stored
        std::fs::write(project.get_roll_path(1, "png"), "half").unwrap();
        assert_eq!(
            rolls(&project),
            vec![(0, "0".to_string()), (2, "2".to_string())]
        );
        assert_eq!(project.load_roll_params(2).unwrap().list, sets[2].list);
        assert!(project.load_roll_params(1).is_err());

        project.save_skipped_roll(1, &sets[1]).unwrap();
        let skipped: Vec<(usize, bool)> = project
            .load_variations()
            .unwrap()
            .unwrap()
            .rolls
            .iter()
            .map(|roll| (roll.index, roll.skipped))
            .collect();
        assert_eq!(skipped, vec![(0, false), (1, true), (2, false)]);
        assert!(project.load_roll_layer(1).is_err());

        let notes = project.get_variations_dir().join("notes.txt");
        std::fs::write(&notes, "mine").unwrap();
        project.start_variations(&meta()).unwrap();
        assert_eq!(rolls(&project), vec![]);
        assert!(notes.exists());
        assert!(!project.get_roll_path(1, "png").exists());
        assert!(!project.get_roll_path(1, "skipped").exists());
        assert!(!project.get_roll_path(2, "plotl").exists());
        assert!(project.get_variations_dir().join(".gitignore").exists());
    }
}
