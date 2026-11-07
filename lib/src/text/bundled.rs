use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use plottery_fonts::{BundledFont, FontKind};

use super::Font;

impl Font {
    /// A font shipped with Plottery, parsed once per process; see `plottery_fonts` for licences.
    ///
    /// ```
    /// # use plottery_lib::*;
    /// let font = Font::bundled(BundledFont::HersheySans);
    /// assert!(font.is_stroke());
    /// ```
    pub fn bundled(font: BundledFont) -> Font {
        static CACHE: OnceLock<Mutex<HashMap<BundledFont, Font>>> = OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        if let Some(loaded) = cache.lock().unwrap().get(&font) {
            return loaded.clone();
        }
        let loaded = match font.kind() {
            FontKind::Stroke => Font::from_svg_str(
                std::str::from_utf8(font.bytes()).expect("bundled SVG font is UTF-8"),
            ),
            FontKind::SingleLine => Font::from_bytes_single_line(font.bytes().to_vec()),
            FontKind::Outline => Font::from_bytes(font.bytes().to_vec()),
        }
        .unwrap_or_else(|error| panic!("bundled font {font:?} failed to load: {error:#}"));
        cache.lock().unwrap().insert(font, loaded.clone());
        loaded
    }
}
