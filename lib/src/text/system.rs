use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use skrifa::font::{attribute::Style, name::NameId, Blob};

use super::{outline_font::name, Font};

/// Requested style for system font lookup; `weight` is CSS-like, 400 regular, 700 bold.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontStyle {
    pub weight: f32,
    pub italic: bool,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            weight: 400.0,
            italic: false,
        }
    }
}

/// An installed font by family and style, looked up on the machine that runs the code.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemFont {
    pub family: String,
    pub style: FontStyle,
}

impl SystemFont {
    /// Regular upright face of `family`.
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            style: FontStyle::default(),
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.style.weight = weight;
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.style.italic = italic;
        self
    }

    /// Fails when the family is not installed; the closest style wins.
    pub fn load(&self) -> Result<Font> {
        Font::system_with(&self.family, self.style)
    }
}

impl Default for SystemFont {
    fn default() -> Self {
        Self::new(if cfg!(target_os = "macos") {
            "Helvetica"
        } else {
            "DejaVu Sans"
        })
    }
}

/// One style of a [`SystemFontFamily`], e.g. `Bold Italic`.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemFontFace {
    pub style_name: String,
    pub style: FontStyle,
}

/// An installed font family and the styles it comes in, lightest first.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemFontFamily {
    pub name: String,
    pub faces: Vec<SystemFontFace>,
}

impl SystemFontFamily {
    /// The face [`SystemFont::load`] picks for `style`.
    pub fn closest_face(&self, style: FontStyle) -> Option<&SystemFontFace> {
        self.faces
            .iter()
            .min_by_key(|face| style_distance(face.style, 1.0, style))
    }
}

/// What the listing needs of one face; `width` is a ratio, 1 is normal.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FaceRecord {
    pub(crate) family: String,
    pub(crate) style_name: String,
    pub(crate) style: FontStyle,
    pub(crate) width: f32,
}

fn face_style(font: &skrifa::Font) -> (FontStyle, f32) {
    let attributes = font.default_attributes();
    let style = FontStyle {
        weight: attributes.weight.value(),
        italic: !matches!(attributes.style, Style::Normal),
    };
    (style, attributes.width.ratio())
}

/// `None` for faces that have no outlines to draw, like bitmap emoji.
fn face_record(font: &skrifa::Font) -> Option<FaceRecord> {
    if font.outline_glyphs().is_empty() {
        return None;
    }
    let family =
        name(font, NameId::TYPOGRAPHIC_FAMILY_NAME).or_else(|| name(font, NameId::FAMILY_NAME))?;
    let style_name = name(font, NameId::TYPOGRAPHIC_SUBFAMILY_NAME)
        .or_else(|| name(font, NameId::SUBFAMILY_NAME))
        .unwrap_or_else(|| "Regular".to_string());
    let (style, width) = face_style(font);
    Some(FaceRecord {
        family,
        style_name,
        style,
        width,
    })
}

/// Families by name ignoring case, one face per weight and slant (the most normal width), without
/// the hidden families whose name starts with a dot.
pub(crate) fn group_families(
    records: impl IntoIterator<Item = FaceRecord>,
) -> Vec<SystemFontFamily> {
    let mut families: BTreeMap<String, (String, Vec<FaceRecord>)> = BTreeMap::new();
    for record in records {
        if record.family.starts_with('.') || record.family.trim().is_empty() {
            continue;
        }
        families
            .entry(record.family.to_lowercase())
            .or_insert_with(|| (record.family.clone(), Vec::new()))
            .1
            .push(record);
    }
    families
        .into_values()
        .map(|(name, mut records)| {
            records.sort_by(|a, b| {
                a.style
                    .weight
                    .total_cmp(&b.style.weight)
                    .then(a.style.italic.cmp(&b.style.italic))
                    .then(width_distance(a.width).cmp(&width_distance(b.width)))
            });
            records.dedup_by(|later, kept| later.style == kept.style);
            let faces = records
                .into_iter()
                .map(|record| SystemFontFace {
                    style_name: record.style_name,
                    style: record.style,
                })
                .collect();
            SystemFontFamily { name, faces }
        })
        .collect()
}

/// The installed outline font families, sorted by name. Reads every font file, so call it once and keep the result.
pub fn system_font_families() -> Vec<SystemFontFamily> {
    let records: Vec<FaceRecord> = font_files(&system_font_dirs())
        .par_iter()
        .flat_map_iter(|path| {
            faces_in_file(path)
                .iter()
                .filter_map(face_record)
                .collect::<Vec<_>>()
        })
        .collect();
    group_families(records)
}

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

fn width_distance(width: f32) -> i32 {
    ((width - 1.0).abs() * 1000.0) as i32
}

/// Smaller is closer to `wanted`: weight first, then slant, then the most normal width.
fn style_distance(candidate: FontStyle, width: f32, wanted: FontStyle) -> (i32, bool, i32) {
    (
        (candidate.weight - wanted.weight).abs() as i32,
        candidate.italic != wanted.italic,
        width_distance(width),
    )
}

fn best_match(files: &[PathBuf], family: &str, style: FontStyle) -> Option<skrifa::Font> {
    files
        .iter()
        .flat_map(|path| faces_in_file(path))
        .filter(|font| has_family(font, family))
        .min_by_key(|font| {
            let (candidate, width) = face_style(font);
            style_distance(candidate, width, style)
        })
}

/// Lowercase letters and digits only, so `Helvetica Neue` and `HelveticaNeue-Bold` compare.
fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether a file named `stem` probably holds `family`: one name contains the other, as in
/// `Arial Bold.ttf` for Arial and `PingFang.ttc` for PingFang SC.
pub(crate) fn is_likely_file(stem: &str, family: &str) -> bool {
    let (stem, family) = (compact(stem), compact(family));
    stem.contains(&family) || (stem.len() >= 3 && family.contains(&stem))
}

/// Scans the system font directories for `family`, closest `style` wins.
pub(crate) fn find_system_font(family: &str, style: FontStyle) -> Result<skrifa::Font> {
    let dirs = system_font_dirs();
    let files = font_files(&dirs);
    // files named after the family first, the full scan only if that finds nothing
    let (likely, rest): (Vec<PathBuf>, Vec<PathBuf>) = files.into_iter().partition(|path| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| is_likely_file(stem, family))
    });
    best_match(&likely, family, style)
        .or_else(|| best_match(&rest, family, style))
        .ok_or_else(|| anyhow!("no system font with family '{family}' in {dirs:?}"))
}
