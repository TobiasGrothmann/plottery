use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use plottery_lib::{BundledFont, Font, Plottable, SampleSettings, Text, V2};

/// Shown when the font lacks most characters of its own name.
const FALLBACK_TEXT: &str = "Aa Bb 123";
/// Em range (y up) every preview covers, so all fonts render at the same scale unless they overflow.
const REFERENCE_TOP: f32 = 0.8;
const REFERENCE_BOTTOM: f32 = -0.25;
/// Rows are ~20 px per em, so a few points per em are plenty.
const POINTS_PER_EM: f32 = 16.0;
/// WebKit draws zero-length subpaths with an unscaled round cap, a blob one em wide.
const DOT_LENGTH: f32 = 0.01;

/// A font's label drawn in itself, as one SVG path in em units with y pointing down.
#[derive(Debug, Clone, PartialEq)]
pub struct FontPreview {
    pub d: String,
    pub width: f32,
    pub height: f32,
}

/// `HersheySansMedium` → `Hershey Sans Medium`
pub fn font_label(font: BundledFont) -> String {
    let name = font.name();
    let mut label = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            label.push(' ');
        }
        label.push(c);
    }
    label
}

fn cache() -> &'static Mutex<HashMap<BundledFont, Arc<FontPreview>>> {
    static CACHE: OnceLock<Mutex<HashMap<BundledFont, Arc<FontPreview>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Cached per process; the first call per font parses it and lays out the text.
pub fn preview(font: BundledFont) -> Arc<FontPreview> {
    if let Some(cached) = cached_preview(font) {
        return cached;
    }
    let rendered = Arc::new(render(font));
    cache().lock().unwrap().insert(font, rendered.clone());
    rendered
}

/// The preview if [`preview`] already rendered it.
pub fn cached_preview(font: BundledFont) -> Option<Arc<FontPreview>> {
    cache().lock().unwrap().get(&font).cloned()
}

/// `text` without the characters `font` has no glyph for, `None` if over a third of them are missing.
fn drawable_text(text: &str, font: &Font) -> Option<String> {
    let visible = text.chars().filter(|c| !c.is_whitespace()).count();
    let kept: String = text
        .chars()
        .filter(|c| c.is_whitespace() || font.has_glyph(*c))
        .collect();
    let kept_visible = kept.chars().filter(|c| !c.is_whitespace()).count();
    (kept_visible * 3 >= visible * 2).then_some(kept)
}

fn render(font: BundledFont) -> FontPreview {
    let loaded = Font::bundled(font);
    let label = font_label(font);
    let text = drawable_text(&label, &loaded)
        .or_else(|| drawable_text(FALLBACK_TEXT, &loaded))
        .unwrap_or_else(|| FALLBACK_TEXT.to_string());

    let polylines: Vec<_> = Text::new(text, &loaded)
        .size(1.0)
        .to_layer(SampleSettings::new(POINTS_PER_EM))
        .map(|layer| {
            layer
                .iter()
                .map(|shape| shape.get_points(SampleSettings::new(POINTS_PER_EM)))
                .filter(|points| points.len() >= 2)
                .collect()
        })
        .unwrap_or_default();

    let points = || polylines.iter().flatten();
    let (min_x, max_x) = points().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
        (lo.min(p.x), hi.max(p.x))
    });
    let (min_x, max_x) = if min_x <= max_x {
        (min_x, max_x)
    } else {
        (0.0, 1.0)
    };
    let (min_y, max_y) = points().fold((REFERENCE_BOTTOM, REFERENCE_TOP), |(lo, hi), p| {
        (lo.min(p.y), hi.max(p.y))
    });

    let d = polylines
        .iter()
        .map(|points| subpath(points, min_x, max_y))
        .collect::<Vec<_>>()
        .join(" ");

    FontPreview {
        d,
        width: (max_x - min_x).max(f32::EPSILON),
        height: max_y - min_y,
    }
}

/// `M…L…` with y flipped; a stroke that rounds to one point (Mistral's dots) becomes a short dash.
fn subpath(points: &[V2], min_x: f32, max_y: f32) -> String {
    let coords: Vec<String> = points
        .iter()
        .map(|p| format!("{:.2} {:.2}", p.x - min_x, max_y - p.y))
        .collect();
    if coords.iter().all(|c| *c == coords[0]) {
        return format!("M{} h{DOT_LENGTH}", coords[0]);
    }
    coords
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}{c}", if i == 0 { 'M' } else { 'L' }))
        .collect::<Vec<_>>()
        .join(" ")
}
