use anyhow::Result;

use super::{
    layout::{block_y_offset, break_lines, line_x_offset, union},
    Font, PlacedGlyph, PositionedGlyph, TextAlign, TextLayout, VerticalAlign,
};
use crate::{BoundingBox, Layer, Rect, SampleSettings, Spline, Translate, V2};

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
    position: V2,
    align: TextAlign,
    vertical_align: VerticalAlign,
    letter_spacing: f32,
    line_spacing: f32,
    max_width: Option<f32>,
}

impl Text {
    /// Em size 1 cm at the origin, left aligned on the first baseline, line spacing 1.25.
    pub fn new(text: impl Into<String>, font: &Font) -> Self {
        Self {
            text: text.into(),
            font: font.clone(),
            size: 1.0,
            position: V2::zero(),
            align: TextAlign::default(),
            vertical_align: VerticalAlign::default(),
            letter_spacing: 0.0,
            line_spacing: 1.25,
            max_width: None,
        }
    }

    /// Em size in cm; capitals are roughly 0.5 to 0.7 of it depending on the font, see [`Text::cap_height`].
    pub fn size(mut self, em_cm: f32) -> Self {
        self.size = em_cm;
        self
    }

    /// Em size such that the font's capitals are `cm` tall (0.7 em if the font does not say).
    pub fn cap_height(self, cm: f32) -> Self {
        let cap_per_em = self.font.metrics(1.0).cap_height.unwrap_or(0.7);
        self.size(cm / cap_per_em)
    }

    /// Anchor: start of the first baseline unless aligned otherwise.
    pub fn at(mut self, position: V2) -> Self {
        self.position = position;
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

    /// Greedy word wrap at spaces. Words are shaped one by one, so kerning across a space is lost.
    pub fn max_width(mut self, cm: f32) -> Self {
        self.max_width = Some(cm);
        self
    }

    pub fn layout(&self) -> Result<TextLayout> {
        let letter_spacing = self.letter_spacing;
        let space_advance: f32 = self
            .font
            .shape(" ", self.size)?
            .iter()
            .map(|glyph| glyph.advance)
            .sum();
        let space_step = space_advance + letter_spacing;
        // between the trailing spacing of a word and the space's own
        let space_width = space_advance + 2.0 * letter_spacing;
        let word_width = |word: &[PositionedGlyph]| {
            if word.is_empty() {
                0.0
            } else {
                word.iter()
                    .map(|glyph| glyph.advance + letter_spacing)
                    .sum::<f32>()
                    - letter_spacing
            }
        };

        let paragraphs: Vec<Vec<Vec<PositionedGlyph>>> = self
            .text
            .replace('\r', "")
            .split('\n')
            .map(|line| {
                line.split(' ')
                    .map(|word| self.font.shape(word, self.size))
                    .collect()
            })
            .collect::<Result<_>>()?;

        let lines: Vec<(&[Vec<PositionedGlyph>], f32)> = paragraphs
            .iter()
            .flat_map(|words| {
                let widths: Vec<f32> = words.iter().map(|word| word_width(word)).collect();
                break_lines(&widths, space_width, self.max_width)
                    .into_iter()
                    .map(|range| {
                        let width = widths[range.clone()].iter().sum::<f32>()
                            + space_width * range.len().saturating_sub(1) as f32;
                        (&words[range], width)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let metrics = self.font.metrics(self.size);
        let line_height = self.line_spacing * self.size;
        let first_baseline =
            block_y_offset(self.vertical_align, &metrics, line_height, lines.len());

        let glyphs: Vec<PlacedGlyph> = lines
            .iter()
            .enumerate()
            .flat_map(|(line, (words, width))| {
                let origin = self.position
                    + V2::new(
                        line_x_offset(self.align, *width),
                        first_baseline - line as f32 * line_height,
                    );
                // `None` is the space between two words
                words
                    .iter()
                    .enumerate()
                    .flat_map(|(i, word)| {
                        (i > 0)
                            .then_some(None)
                            .into_iter()
                            .chain(word.iter().map(Some))
                    })
                    .scan(0.0, |pen, glyph| {
                        let x = *pen;
                        *pen += glyph.map_or(space_step, |glyph| glyph.advance + letter_spacing);
                        Some((glyph, x))
                    })
                    .filter_map(|(glyph, x)| glyph.map(|glyph| (glyph, x)))
                    .map(move |(glyph, x)| PlacedGlyph {
                        glyph: self
                            .font
                            .glyph_by_id(glyph.id, self.size)
                            .translate(origin + V2::new(x, 0.0) + glyph.offset),
                        line,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let bounds = glyphs
            .iter()
            .filter_map(|placed| placed.glyph.bounding_box())
            .reduce(union);

        Ok(TextLayout {
            glyphs,
            line_widths: lines.iter().map(|(_, width)| *width).collect(),
            bounds,
        })
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

    /// A flat layer named `"text"` with one path per contour; see [`Text::layout`] for per-glyph grouping.
    pub fn to_layer(&self, sample_settings: SampleSettings) -> Result<Layer> {
        let paths = self
            .layout()?
            .glyphs
            .iter()
            .flat_map(|placed| placed.glyph.to_paths(sample_settings))
            .collect::<Vec<_>>();
        Ok(Layer::new_from(paths).with_name("text"))
    }

    /// Hull of all contours, `None` for text without any.
    pub fn bounding_box(&self) -> Result<Option<Rect>> {
        Ok(self.layout()?.bounds)
    }
}
