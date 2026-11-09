#[cfg(test)]
mod tests {
    use std::time::Instant;

    use plottery_lib::BundledFont;

    use crate::router::editor::params_editor::font_preview::preview;

    #[test]
    fn every_bundled_font_has_a_preview() {
        let start = Instant::now();
        let total: usize = BundledFont::ALL
            .iter()
            .map(|font| {
                let preview = preview(*font);
                assert!(preview.d.starts_with('M'), "{font:?} has an empty preview");
                assert!(preview.width > 0.0 && preview.height > 0.0, "{font:?}");
                preview.d.len()
            })
            .sum();
        println!(
            "{} previews in {:?}, {total} bytes of path data",
            BundledFont::ALL.len(),
            start.elapsed()
        );
    }
}
