use std::ops::Range;

use super::{FontMetrics, Glyph};
use crate::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Which edge of the text block sits on the anchor; `Baseline` is the first line's baseline.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
    Center,
    Bottom,
}

/// A glyph translated to its final position in cm.
#[derive(Clone, Debug)]
pub struct PlacedGlyph {
    pub glyph: Glyph,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub struct TextLayout {
    pub glyphs: Vec<PlacedGlyph>,
    pub line_widths: Vec<f32>,
    /// Hull of all contours, `None` without any.
    pub bounds: Option<Rect>,
}

/// Greedy word wrap; an over-long word gets its own line, zero-width words never break.
pub(crate) fn break_lines(
    word_widths: &[f32],
    space_width: f32,
    max_width: Option<f32>,
) -> Vec<Range<usize>> {
    if word_widths.is_empty() {
        return Vec::new();
    }
    let Some(max_width) = max_width else {
        return std::iter::once(0..word_widths.len()).collect();
    };
    let mut lines = Vec::new();
    let mut start = 0;
    let mut width = 0.0;
    for (i, &word) in word_widths.iter().enumerate() {
        if i > start && word > 0.0 && width + space_width + word > max_width {
            lines.push(start..i);
            start = i;
            width = word;
        } else if i == start {
            width = word;
        } else {
            width += space_width + word;
        }
    }
    lines.push(start..word_widths.len());
    lines
}

pub(crate) fn line_x_offset(align: TextAlign, line_width: f32) -> f32 {
    match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => -line_width / 2.0,
        TextAlign::Right => -line_width,
    }
}

/// Y of the first baseline relative to the anchor.
pub(crate) fn block_y_offset(
    align: VerticalAlign,
    metrics: &FontMetrics,
    line_height: f32,
    lines: usize,
) -> f32 {
    let top = -metrics.ascender;
    let bottom = lines.saturating_sub(1) as f32 * line_height - metrics.descender;
    match align {
        VerticalAlign::Baseline => 0.0,
        VerticalAlign::Top => top,
        VerticalAlign::Bottom => bottom,
        VerticalAlign::Center => (top + bottom) / 2.0,
    }
}

pub(crate) fn union(a: Rect, b: Rect) -> Rect {
    Rect::new(a.bl().min(b.bl()), a.tr().max(b.tr()))
}
