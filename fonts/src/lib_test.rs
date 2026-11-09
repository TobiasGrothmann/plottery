use super::BundledFont;

#[test]
fn names_round_trip() {
    for font in BundledFont::ALL {
        assert_eq!(BundledFont::from_name(font.name()), Some(*font));
    }
    assert_eq!(BundledFont::from_name("hersheysans"), None);
}
