use plottery_lib::*;
use plottery_project::*;
use std::f32::consts::TAU;

// Rows whose spacing follows an easing curve, so the page fades from light to dark.
#[derive(PlotteryEnum, Clone, Copy)]
pub enum Style {
    Lines,
    #[default]
    Waves,
    Zigzag,
}

impl Style {
    /// Height of the row at `u` waves along it, in -1..1.
    fn wave(self, u: f32) -> f32 {
        match self {
            Style::Lines => 0.0,
            Style::Waves => (u * TAU).sin(),
            Style::Zigzag => 1.0 - 4.0 * (u.fract() - 0.5).abs(),
        }
    }

    fn samples_per_wave(self) -> i32 {
        match self {
            Style::Lines => 0,
            Style::Waves => 24,
            Style::Zigzag => 2,
        }
    }
}

#[derive(PlotteryParams)]
pub struct Border {
    /// nested outlines, 1 mm apart
    #[value(2)]
    #[range(1, 6)]
    pub lines: i32,

    /// space between the border and the rows in cm
    #[value(0.5)]
    #[range(0.0, 2.0)]
    pub gap: f32,
}

#[derive(PlotteryParams)]
pub struct Params {
    /// paper size, orientation and margin
    pub paper: Paper,

    #[section("Rows")]
    /// number of rows from top to bottom
    #[value(90)]
    #[range(10, 300)]
    pub rows: i32,

    /// where the rows sit between top and bottom, linear spaces them evenly
    #[value(Easing::bias(0.6))]
    pub spacing: Easing,

    #[section("Shape")]
    pub style: Style,

    /// wave height in cm, from the top row to the bottom row
    #[value(Range::new(0.0, 0.3))]
    #[range(0.0, 2.0)]
    pub amplitude: Range<f32>,

    /// waves or zigzags across one row
    #[value(16)]
    #[range(1, 80)]
    pub waves: i32,

    /// outlines around the rows
    #[value(Some(Border::new_with_defaults()))]
    pub border: Option<Border>,
}

pub fn generate(params: Params) -> Layer {
    let frame = params.paper.frame();
    let inner = frame.inner_rect();

    let (outlines, rows_rect): (Vec<Rect>, Rect) = match &params.border {
        Some(border) => (
            (0..border.lines)
                .map(|i| inner.shrink(i as f32 * 0.1))
                .collect(),
            inner.shrink((border.lines - 1) as f32 * 0.1 + border.gap),
        ),
        None => (Vec::new(), inner),
    };
    // keep the highest waves inside the rect
    let band = rows_rect.shrink_2d(V2::new(0.0, params.amplitude.max));

    let steps = (params.waves * params.style.samples_per_wave()).max(1);
    let rows = (0..params.rows).map(|row| {
        let t = row as f32 / (params.rows - 1).max(1) as f32;
        let y = band.tl().y - params.spacing.apply(t) * band.height();
        let amplitude = params.amplitude.lerp(t);
        (0..=steps)
            .map(|i| {
                let s = i as f32 / steps as f32;
                let offset = amplitude * params.style.wave(s * params.waves as f32);
                V2::new(band.bl().x + s * band.width(), y + offset)
            })
            .collect::<Path>()
    });

    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.push_many(outlines);
    l.push_many(rows);
    l.with_name("root").optimize_recursive()
}
