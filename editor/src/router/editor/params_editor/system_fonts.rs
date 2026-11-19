use std::sync::{Arc, OnceLock};

use plottery_lib::{system_font_families, FontStyle, SystemFontFamily};

static FAMILIES: OnceLock<Arc<Vec<SystemFontFamily>>> = OnceLock::new();

/// The installed font families. The first call per process scans the font folders, which blocks
/// for about a second; fonts installed later show up after a restart.
pub fn families() -> Arc<Vec<SystemFontFamily>> {
    FAMILIES
        .get_or_init(|| Arc::new(system_font_families()))
        .clone()
}

/// The list if [`families`] already scanned it.
pub fn cached_families() -> Option<Arc<Vec<SystemFontFamily>>> {
    FAMILIES.get().cloned()
}

/// Inline CSS that sets text in an installed font, without faked bold or italic.
pub fn sample_style(family: &str, style: FontStyle) -> String {
    let family: String = family
        .chars()
        .filter(|ch| !ch.is_control())
        .flat_map(|ch| match ch {
            '"' | '\\' => vec!['\\', ch],
            _ => vec![ch],
        })
        .collect();
    let slant = if style.italic { "italic" } else { "normal" };
    format!(
        "font-family: \"{family}\"; font-weight: {}; font-style: {slant};",
        style.weight
    )
}
