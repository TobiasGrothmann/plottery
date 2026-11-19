#[cfg(test)]
mod test_system {
    use crate::{
        text::system::{group_families, is_likely_file, FaceRecord},
        FontStyle, SystemFontFamily,
    };

    fn record(family: &str, style_name: &str, weight: f32, italic: bool, width: f32) -> FaceRecord {
        FaceRecord {
            family: family.to_string(),
            style_name: style_name.to_string(),
            style: FontStyle { weight, italic },
            width,
        }
    }

    fn style_names(family: &SystemFontFamily) -> Vec<&str> {
        family
            .faces
            .iter()
            .map(|face| face.style_name.as_str())
            .collect()
    }

    #[test]
    fn families_are_sorted_ignoring_case_and_merged() {
        let families = group_families([
            record("zapfino", "Regular", 400.0, false, 1.0),
            record("Arial", "Bold", 700.0, false, 1.0),
            record("arial", "Regular", 400.0, false, 1.0),
            record("Baskerville", "Regular", 400.0, false, 1.0),
        ]);
        let names: Vec<&str> = families.iter().map(|family| family.name.as_str()).collect();
        assert_eq!(names, ["Arial", "Baskerville", "zapfino"]);
        assert_eq!(style_names(&families[0]), ["Regular", "Bold"]);
    }

    #[test]
    fn faces_are_sorted_by_weight_then_slant() {
        let families = group_families([
            record("Futura", "Bold Italic", 700.0, true, 1.0),
            record("Futura", "Medium Italic", 500.0, true, 1.0),
            record("Futura", "Bold", 700.0, false, 1.0),
            record("Futura", "Medium", 500.0, false, 1.0),
        ]);
        assert_eq!(
            style_names(&families[0]),
            ["Medium", "Medium Italic", "Bold", "Bold Italic"]
        );
    }

    #[test]
    fn one_face_per_style_prefers_normal_width() {
        let families = group_families([
            record("Helvetica Neue", "Condensed Bold", 700.0, false, 0.75),
            record("Helvetica Neue", "Bold", 700.0, false, 1.0),
            record("Helvetica Neue", "Extended Bold", 700.0, false, 1.25),
        ]);
        assert_eq!(style_names(&families[0]), ["Bold"]);
    }

    #[test]
    fn hidden_families_are_dropped() {
        let families = group_families([
            record(".SF NS", "Regular", 400.0, false, 1.0),
            record("Menlo", "Regular", 400.0, false, 1.0),
        ]);
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].name, "Menlo");
    }

    #[test]
    fn closest_face_matches_weight_before_slant() {
        let families = group_families([
            record("Georgia", "Regular", 400.0, false, 1.0),
            record("Georgia", "Italic", 400.0, true, 1.0),
            record("Georgia", "Bold", 700.0, false, 1.0),
        ]);
        let closest = |weight, italic| {
            families[0]
                .closest_face(FontStyle { weight, italic })
                .unwrap()
                .style_name
                .as_str()
        };
        assert_eq!(closest(400.0, true), "Italic");
        assert_eq!(closest(600.0, false), "Bold");
        assert_eq!(closest(700.0, true), "Bold");
        assert_eq!(closest(300.0, false), "Regular");
    }

    #[test]
    fn likely_files_contain_the_family_or_are_contained_in_it() {
        assert!(is_likely_file("HelveticaNeue", "Helvetica Neue"));
        assert!(is_likely_file("Arial Narrow Bold", "Arial Narrow"));
        assert!(is_likely_file("PingFang", "PingFang SC"));
        assert!(is_likely_file("SF-Pro-Display-Bold", "SF Pro Display"));
        assert!(!is_likely_file("Times New Roman", "Arial"));
        // a short stem inside a long family name says nothing
        assert!(!is_likely_file("NY", "Sony Sketch"));
    }
}
