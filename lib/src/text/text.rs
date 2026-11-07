use std::ops::Range;

use anyhow::Result;

use super::{
    layout::{block_y_offset, break_lines, break_lines_balanced, line_width, line_x_offset, union},
    Font, PlacedGlyph, PositionedGlyph, TextAlign, TextLayout, TextLine, VerticalAlign, Wrap,
};
use crate::{Angle, BoundingBox, Layer, Rect, Rotate, SampleSettings, Spline, Translate, V2};

/// A block of text in a [`Font`]; `\n` breaks lines, `\r` is dropped, `\t` is passed to the font.
///
/// ### Example
/// ```no_run
/// # use plottery_lib::*;
/// let font = Font::system("Helvetica").unwrap();
/// let layer = Text::new("Hello", &font)
///     .size(2.0)
///     .at(V2::new(5.0, 5.0))
///     .align(TextAlign::Center)
///     .to_layer(SampleSettings::default())
///     .unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Text {
    text: String,
    font: Font,
    size: f32,
    anchor: Anchor,
    align: TextAlign,
    vertical_align: VerticalAlign,
    letter_spacing: f32,
    line_spacing: f32,
    max_width: Option<f32>,
    wrap: Wrap,
    fit: Option<Fit>,
    rotation: Angle,
}

#[derive(Clone, Copy, Debug)]
enum Anchor {
    Point(V2),
    Rect(Rect),
}

#[derive(Clone, Copy, Debug)]
enum Fit {
    Width(f32),
    Rect,
}

/// A word, word part or character shaped at em size 1, followed by `spaces` space characters.
struct Token {
    glyphs: Vec<PositionedGlyph>,
    spaces: usize,
}

impl Token {
    fn advance(&self) -> f32 {
        self.glyphs.iter().map(|glyph| glyph.advance).sum()
    }
}

/// A paragraph measured at the final size and broken into lines.
struct Measured<'a> {
    tokens: &'a [Token],
    /// `(width, glue_after)` per token
    widths: Vec<(f32, f32)>,
    lines: Vec<Range<usize>>,
}

/// One line before placement.
struct LineBreak<'a> {
    tokens: &'a [Token],
    widths: &'a [(f32, f32)],
    range: Range<usize>,
    last_in_paragraph: bool,
}

impl Text {
    /// Em size 1 cm at the origin, left aligned on the first baseline, line spacing 1.25.
    pub fn new(text: impl Into<String>, font: &Font) -> Self {
        Self {
            text: text.into(),
            font: font.clone(),
            size: 1.0,
            anchor: Anchor::Point(V2::zero()),
            align: TextAlign::default(),
            vertical_align: VerticalAlign::default(),
            letter_spacing: 0.0,
            line_spacing: 1.25,
            max_width: None,
            wrap: Wrap::default(),
            fit: None,
            rotation: Angle::zero(),
        }
    }

    /// Em size in cm; capitals are roughly 0.5 to 0.7 of it depending on the font, see [`Text::cap_height`].
    pub fn size(mut self, em_cm: f32) -> Self {
        self.size = em_cm;
        self.fit = None;
        self
    }

    /// Em size such that the font's capitals are `cm` tall (0.7 em if the font does not say).
    pub fn cap_height(self, cm: f32) -> Self {
        let cap_per_em = self.font.metrics(1.0).cap_height.unwrap_or(0.7);
        self.size(cm / cap_per_em)
    }

    /// Em size such that the widest line is exactly `cm` wide; nothing wraps.
    pub fn fit_width(mut self, cm: f32) -> Self {
        self.fit = Some(Fit::Width(cm));
        self
    }

    /// [`Text::in_rect`] with the largest em size whose wrapped block fits the rect by its font metrics.
    pub fn fit_inside(mut self, rect: Rect) -> Self {
        self.anchor = Anchor::Rect(rect);
        self.fit = Some(Fit::Rect);
        self
    }

    /// Anchor: start of the first baseline unless aligned otherwise.
    pub fn at(mut self, position: V2) -> Self {
        self.anchor = Anchor::Point(position);
        self
    }

    /// Anchors to the rect edge chosen by the alignments (`Baseline` puts the last baseline on
    /// the bottom edge) and wraps at the rect's width unless [`Text::max_width`] is set.
    pub fn in_rect(mut self, rect: Rect) -> Self {
        self.anchor = Anchor::Rect(rect);
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    /// Extra advance in cm after every glyph, spaces included.
    pub fn letter_spacing(mut self, cm: f32) -> Self {
        self.letter_spacing = cm;
        self
    }

    /// Baseline distance as a factor of the em size.
    pub fn line_spacing(mut self, factor: f32) -> Self {
        self.line_spacing = factor;
        self
    }

    /// Wraps at spaces and after hyphens. Words are shaped one by one, so kerning across a space is lost.
    pub fn max_width(mut self, cm: f32) -> Self {
        self.max_width = Some(cm);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    /// Rotates the finished block around its anchor point.
    pub fn rotation(mut self, angle: Angle) -> Self {
        self.rotation = angle;
        self
    }

    pub fn layout(&self) -> Result<TextLayout> {
        let spacing = self.letter_spacing;
        let space_advance: f32 = self
            .font
            .shape(" ", 1.0)?
            .iter()
            .map(|glyph| glyph.advance)
            .sum();
        let paragraphs: Vec<Vec<Token>> = self
            .text
            .replace('\r', "")
            .split('\n')
            .map(|paragraph| self.tokenize(paragraph))
            .collect::<Result<_>>()?;

        // (width, glue_after) of every token of a paragraph at an em size
        let measure = |tokens: &[Token], size: f32| -> Vec<(f32, f32)> {
            tokens
                .iter()
                .map(|token| {
                    let width = token.advance() * size
                        + spacing * token.glyphs.len().saturating_sub(1) as f32;
                    let glue = spacing + token.spaces as f32 * (space_advance * size + spacing);
                    (width, glue)
                })
                .collect()
        };
        let rect = match self.anchor {
            Anchor::Rect(rect) => Some(rect),
            Anchor::Point(_) => None,
        };
        let max_width = match self.fit {
            Some(Fit::Width(_)) => None,
            _ => self.max_width.or(rect.map(|rect| rect.width())),
        };
        let break_paragraph = |tokens: &[(f32, f32)]| -> Vec<Range<usize>> {
            match (self.wrap, max_width) {
                (Wrap::Balanced, Some(max_width)) => break_lines_balanced(tokens, max_width),
                _ => break_lines(tokens, max_width),
            }
        };
        let metrics_1 = self.font.metrics(1.0);
        let block_height = |size: f32, lines: usize| {
            (metrics_1.ascender - metrics_1.descender) * size
                + lines.saturating_sub(1) as f32 * self.line_spacing * size
        };

        let size = match self.fit {
            None => self.size,
            Some(Fit::Width(target)) => paragraphs
                .iter()
                .filter_map(|tokens| {
                    let advance: f32 = tokens
                        .iter()
                        .map(|token| token.advance() + token.spaces as f32 * space_advance)
                        .sum();
                    let glyphs: usize = tokens
                        .iter()
                        .map(|token| token.glyphs.len() + token.spaces)
                        .sum();
                    (advance > 0.0)
                        .then(|| (target - spacing * glyphs.saturating_sub(1) as f32) / advance)
                })
                .min_by(f32::total_cmp)
                .unwrap_or(self.size),
            Some(Fit::Rect) => {
                let rect = rect.expect("fit_inside sets a rect anchor");
                let fits = |size: f32| {
                    let (lines, widest) = paragraphs.iter().fold((0, 0.0_f32), |acc, tokens| {
                        let measured = measure(tokens, size);
                        break_paragraph(&measured).into_iter().fold(
                            acc,
                            |(lines, widest), range| {
                                (lines + 1, widest.max(line_width(&measured, range)))
                            },
                        )
                    });
                    widest <= rect.width() && block_height(size, lines) <= rect.height()
                };
                let (mut low, mut high) = (0.0, block_height(1.0, 1).recip() * rect.height());
                for _ in 0..24 {
                    let mid = (low + high) / 2.0;
                    if fits(mid) {
                        low = mid;
                    } else {
                        high = mid;
                    }
                }
                low
            }
        };

        let measured: Vec<Measured> = paragraphs
            .iter()
            .map(|tokens| {
                let widths = measure(tokens, size);
                let lines = break_paragraph(&widths);
                Measured {
                    tokens,
                    widths,
                    lines,
                }
            })
            .collect();
        let lines: Vec<LineBreak> = measured
            .iter()
            .flat_map(|paragraph| {
                paragraph
                    .lines
                    .iter()
                    .enumerate()
                    .map(move |(i, range)| LineBreak {
                        tokens: paragraph.tokens,
                        widths: &paragraph.widths,
                        range: range.clone(),
                        last_in_paragraph: i + 1 == paragraph.lines.len(),
                    })
            })
            .collect();

        let metrics = metrics_1.scale(size);
        let line_height = self.line_spacing * size;
        let anchor = match self.anchor {
            Anchor::Point(point) => point,
            Anchor::Rect(rect) => V2::new(
                match self.align {
                    TextAlign::Left | TextAlign::Justify => rect.bl().x,
                    TextAlign::Center => rect.center().x,
                    TextAlign::Right => rect.tr().x,
                },
                match self.vertical_align {
                    VerticalAlign::Top => rect.tr().y,
                    VerticalAlign::Center => rect.center().y,
                    VerticalAlign::Bottom | VerticalAlign::Baseline => rect.bl().y,
                },
            ),
        };
        let first_baseline = match (self.anchor, self.vertical_align) {
            (Anchor::Rect(_), VerticalAlign::Baseline) => {
                lines.len().saturating_sub(1) as f32 * line_height
            }
            _ => block_y_offset(self.vertical_align, &metrics, line_height, lines.len()),
        };

        let mut glyphs: Vec<PlacedGlyph> = Vec::new();
        let mut placed_lines: Vec<TextLine> = Vec::with_capacity(lines.len());
        for (index, line) in lines.iter().enumerate() {
            let range = line.range.clone();
            let natural = line_width(line.widths, range.clone());
            let gaps = line.tokens[range.start..range.end.saturating_sub(1)]
                .iter()
                .filter(|token| token.spaces > 0)
                .count();
            let stretch = match (self.align, max_width) {
                (TextAlign::Justify, Some(max_width)) if !line.last_in_paragraph && gaps > 0 => {
                    (max_width - natural) / gaps as f32
                }
                _ => 0.0,
            };
            let width = natural + stretch * gaps as f32;
            let origin = anchor
                + V2::new(
                    line_x_offset(self.align, width),
                    first_baseline - index as f32 * line_height,
                );
            let start = glyphs.len();
            let mut pen = 0.0;
            for (token, &(token_width, glue)) in
                line.tokens[range.clone()].iter().zip(&line.widths[range])
            {
                glyphs.extend(
                    token
                        .glyphs
                        .iter()
                        .scan(pen, |x, glyph| {
                            let at = origin + V2::new(*x, 0.0) + glyph.offset * size;
                            *x += glyph.advance * size + spacing;
                            Some((at, glyph.id))
                        })
                        .map(|(at, id)| PlacedGlyph {
                            glyph: self.font.glyph_by_id(id, size).translate(at),
                            line: index,
                        }),
                );
                pen += token_width + glue + if token.spaces > 0 { stretch } else { 0.0 };
            }
            placed_lines.push(TextLine {
                width,
                origin,
                glyphs: start..glyphs.len(),
            });
        }

        if self.rotation.to_rad() != 0.0 {
            for placed in &mut glyphs {
                placed.glyph = placed.glyph.rotate_around(anchor, self.rotation);
            }
            for line in &mut placed_lines {
                line.origin = line.origin.rotate_around(anchor, self.rotation);
            }
        }

        let bounds = glyphs
            .iter()
            .filter_map(|placed| placed.glyph.bounding_box())
            .reduce(union);

        Ok(TextLayout {
            glyphs,
            lines: placed_lines,
            bounds,
        })
    }

    /// Words and word parts ending in a hyphen, or single characters for [`Wrap::Anywhere`].
    fn tokenize(&self, paragraph: &str) -> Result<Vec<Token>> {
        let is_hyphen = |ch: char| matches!(ch, '-' | '\u{2010}' | '\u{2013}');
        let mut tokens: Vec<Token> = Vec::new();
        let mut push = |text: &str, spaces: usize| -> Result<()> {
            match tokens.last_mut() {
                Some(last) if text.is_empty() => last.spaces += spaces,
                _ => tokens.push(Token {
                    glyphs: self.font.shape(text, 1.0)?,
                    spaces,
                }),
            }
            Ok(())
        };
        match self.wrap {
            Wrap::Anywhere => {
                for ch in paragraph.chars() {
                    let mut buffer = [0; 4];
                    match ch {
                        ' ' => push("", 1)?,
                        _ => push(ch.encode_utf8(&mut buffer), 0)?,
                    }
                }
            }
            Wrap::Word | Wrap::Balanced => {
                for word in paragraph.split(' ') {
                    let mut parts = word.split_inclusive(is_hyphen).peekable();
                    if parts.peek().is_none() {
                        push("", 1)?;
                    }
                    while let Some(part) = parts.next() {
                        push(part, usize::from(parts.peek().is_none()))?;
                    }
                }
            }
        }
        if let Some(last) = tokens.last_mut() {
            last.spaces = 0;
        }
        Ok(tokens)
    }

    /// Exact curves in cm, one `Vec` per glyph in reading order.
    pub fn glyph_splines(&self) -> Result<Vec<Vec<Spline>>> {
        Ok(self
            .layout()?
            .glyphs
            .into_iter()
            .map(|placed| placed.glyph.contours)
            .collect())
    }

    /// A flat layer named `"text"` with one path per contour; see [`Text::layout`] for lines and glyphs.
    pub fn to_layer(&self, sample_settings: SampleSettings) -> Result<Layer> {
        Ok(self.layout()?.to_layer(sample_settings))
    }

    /// Hull of all contours, `None` for text without any.
    pub fn bounding_box(&self) -> Result<Option<Rect>> {
        Ok(self.layout()?.bounds)
    }
}
