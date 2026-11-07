use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::{FontMetrics, Glyph};
use crate::{Layer, Rect, SampleSettings, V2};

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    /// Spaces widen so lines end at `max_width`; a paragraph's last line stays left.
    Justify,
}

/// Which edge of the text block sits on the anchor; `Baseline` is the first line's baseline.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
    Center,
    Bottom,
}

/// Where a paragraph may break at `max_width`.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum Wrap {
    /// Greedy at spaces and after hyphens.
    #[default]
    Word,
    /// As many lines as `Word`, but with the breaks placed for the least ragged lines.
    Balanced,
    /// Between any two characters.
    Anywhere,
}

/// A glyph translated to its final position in cm.
#[derive(Clone, Debug)]
pub struct PlacedGlyph {
    pub glyph: Glyph,
    pub line: usize,
}

/// One laid out line: `origin` is the baseline start after alignment and rotation.
#[derive(Clone, Debug)]
pub struct TextLine {
    pub width: f32,
    pub origin: V2,
    /// Indices into [`TextLayout::glyphs`].
    pub glyphs: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct TextLayout {
    pub glyphs: Vec<PlacedGlyph>,
    pub lines: Vec<TextLine>,
    /// Hull of all contours, `None` without any.
    pub bounds: Option<Rect>,
}

impl TextLayout {
    /// A flat layer named `"text"` with one path per contour.
    pub fn to_layer(&self, sample_settings: SampleSettings) -> Layer {
        Layer::new_from(self.paths(0..self.glyphs.len(), sample_settings)).with_name("text")
    }

    /// Layer `"text"` with one sublayer per line, named `line 0`, `line 1`, …
    pub fn to_layer_by_line(&self, sample_settings: SampleSettings) -> Layer {
        let lines = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                Layer::new_from(self.paths(line.glyphs.clone(), sample_settings))
                    .with_name(&format!("line {i}"))
            })
            .collect();
        Layer::new_from_shapes_and_layers(Vec::new(), lines).with_name("text")
    }

    fn paths(&self, glyphs: Range<usize>, sample_settings: SampleSettings) -> Vec<crate::Path> {
        self.glyphs[glyphs]
            .iter()
            .flat_map(|placed| placed.glyph.to_paths(sample_settings))
            .collect()
    }
}

/// Greedy wrap of `(width, glue_after)` tokens; an over-long token gets its own line,
/// zero-width tokens never break.
pub(crate) fn break_lines(tokens: &[(f32, f32)], max_width: Option<f32>) -> Vec<Range<usize>> {
    if tokens.is_empty() {
        return Vec::new();
    }
    let Some(max_width) = max_width else {
        return std::iter::once(0..tokens.len()).collect();
    };
    let mut lines = Vec::new();
    let mut start = 0;
    let mut width = 0.0;
    for (i, &(token, _)) in tokens.iter().enumerate() {
        let glue = if i > start { tokens[i - 1].1 } else { 0.0 };
        if i > start && token > 0.0 && width + glue + token > max_width {
            lines.push(start..i);
            start = i;
            width = token;
        } else {
            width += glue + token;
        }
    }
    lines.push(start..tokens.len());
    lines
}

/// Width of `tokens[range]` on one line, the last token's glue excluded.
pub(crate) fn line_width(tokens: &[(f32, f32)], range: Range<usize>) -> f32 {
    let last = range.end.saturating_sub(1);
    tokens[range]
        .iter()
        .map(|(width, glue)| width + glue)
        .sum::<f32>()
        - tokens.get(last).map_or(0.0, |(_, glue)| *glue)
}

/// Same line count as [`break_lines`], break positions by the minimum sum of squared slack.
pub(crate) fn break_lines_balanced(tokens: &[(f32, f32)], max_width: f32) -> Vec<Range<usize>> {
    let greedy = break_lines(tokens, Some(max_width));
    let (n, k) = (tokens.len(), greedy.len());
    if k <= 1 {
        return greedy;
    }
    let cost = |range: Range<usize>| {
        let width = line_width(tokens, range.clone());
        if range.len() > 1 && width > max_width {
            f32::INFINITY
        } else {
            (max_width - width).powi(2)
        }
    };
    // best[l][j]: (cost, start of the last line) for tokens 0..j in l lines
    let mut best = vec![vec![(f32::INFINITY, 0); n + 1]; k + 1];
    best[0][0] = (0.0, 0);
    for l in 1..=k {
        for j in l..=n {
            best[l][j] = (l - 1..j)
                .filter(|&i| best[l - 1][i].0.is_finite())
                .map(|i| (best[l - 1][i].0 + cost(i..j), i))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap_or((f32::INFINITY, 0));
        }
    }
    if !best[k][n].0.is_finite() {
        return greedy;
    }
    let mut lines = Vec::with_capacity(k);
    let mut end = n;
    for l in (1..=k).rev() {
        let start = best[l][end].1;
        lines.push(start..end);
        end = start;
    }
    lines.reverse();
    lines
}

pub(crate) fn line_x_offset(align: TextAlign, line_width: f32) -> f32 {
    match align {
        TextAlign::Left | TextAlign::Justify => 0.0,
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
