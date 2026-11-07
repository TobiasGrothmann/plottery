use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use skrifa::font::{attribute::Style, name::NameId, Blob};

use super::FontStyle;

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn system_font_dirs() -> Vec<PathBuf> {
    let dirs: Vec<PathBuf> = if cfg!(target_os = "macos") {
        [
            "/System/Library/Fonts",
            "/System/Library/Fonts/Supplemental",
            "/Library/Fonts",
        ]
        .iter()
        .map(PathBuf::from)
        .chain(home().map(|home| home.join("Library/Fonts")))
        .collect()
    } else if cfg!(target_os = "windows") {
        std::iter::once(PathBuf::from(r"C:\Windows\Fonts"))
            .chain(
                std::env::var_os("LOCALAPPDATA")
                    .map(|local| PathBuf::from(local).join(r"Microsoft\Windows\Fonts")),
            )
            .collect()
    } else {
        ["/usr/share/fonts", "/usr/local/share/fonts"]
            .iter()
            .map(PathBuf::from)
            .chain(
                home()
                    .into_iter()
                    .flat_map(|home| [home.join(".fonts"), home.join(".local/share/fonts")]),
            )
            .collect()
    };
    dirs.into_iter().filter(|dir| dir.is_dir()).collect()
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc"))
}

/// All font files below `dirs`, recursively, sorted.
fn font_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut stack = dirs.to_vec();
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_dir() {
                stack.push(path);
            } else if is_font_file(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Every face in a file; a collection yields one per index.
fn faces_in_file(path: &Path) -> Vec<skrifa::Font> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    let blob = Blob::from(bytes);
    (0..64)
        .map_while(|index| skrifa::Font::new(blob.clone(), index))
        .collect()
}

fn has_family(font: &skrifa::Font, family: &str) -> bool {
    [NameId::FAMILY_NAME, NameId::TYPOGRAPHIC_FAMILY_NAME]
        .into_iter()
        .filter_map(|id| font.names().get(id))
        .flat_map(|name| name.iter().map(|localized| localized.name().to_string()))
        .any(|name| name.eq_ignore_ascii_case(family))
}

fn style_distance(font: &skrifa::Font, style: FontStyle) -> (i32, bool) {
    let attributes = font.default_attributes();
    let italic = !matches!(attributes.style, Style::Normal);
    (
        (attributes.weight.value() - style.weight).abs() as i32,
        italic != style.italic,
    )
}

fn best_match(files: &[PathBuf], family: &str, style: FontStyle) -> Option<skrifa::Font> {
    files
        .iter()
        .flat_map(|path| faces_in_file(path))
        .filter(|font| has_family(font, family))
        .min_by_key(|font| style_distance(font, style))
}

/// Scans the system font directories for `family`, closest `style` wins.
pub(crate) fn find_system_font(family: &str, style: FontStyle) -> Result<skrifa::Font> {
    let dirs = system_font_dirs();
    let files = font_files(&dirs);
    let compact: String = family
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    // files named after the family first, the full scan only if that finds nothing
    let (likely, rest): (Vec<PathBuf>, Vec<PathBuf>) = files.into_iter().partition(|path| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.to_ascii_lowercase().contains(&compact))
    });
    best_match(&likely, family, style)
        .or_else(|| best_match(&rest, family, style))
        .ok_or_else(|| anyhow!("no system font with family '{family}' in {dirs:?}"))
}
