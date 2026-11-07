use itertools::Either;
use serde::{Deserialize, Serialize};

use crate::{Bezier, Line, SampleSettings, V2};

/// One piece of a [`Spline`](crate::Spline): a straight line or a Bézier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Segment {
    Line(Line),
    Bezier(Bezier),
}

impl Segment {
    pub fn from(&self) -> V2 {
        match self {
            Segment::Line(line) => line.from,
            Segment::Bezier(bezier) => bezier.from(),
        }
    }
    pub fn to(&self) -> V2 {
        match self {
            Segment::Line(line) => line.to,
            Segment::Bezier(bezier) => bezier.to(),
        }
    }
    pub fn tangent_at_start(&self) -> V2 {
        self.tangent_at(0.0)
    }
    pub fn tangent_at_end(&self) -> V2 {
        self.tangent_at(1.0)
    }
    pub fn tangent_at(&self, t: f32) -> V2 {
        match self {
            Segment::Line(line) => line.vector(),
            Segment::Bezier(bezier) => bezier.tangent_at(t),
        }
    }
    pub fn point_at(&self, t: f32) -> V2 {
        match self {
            Segment::Line(line) => line.point_at(t),
            Segment::Bezier(bezier) => bezier.point_at(t),
        }
    }
    pub fn length(&self) -> f32 {
        match self {
            Segment::Line(line) => line.length(),
            Segment::Bezier(bezier) => bezier.length(),
        }
    }
    /// Sampled points including both ends, a line yields exactly its two ends.
    pub fn points(&self, sample_settings: SampleSettings) -> Vec<V2> {
        match self {
            Segment::Line(line) => vec![line.from, line.to],
            Segment::Bezier(bezier) => bezier.points(sample_settings),
        }
    }
    /// All control points, a line's are its two ends.
    pub fn control_points(&self) -> impl Iterator<Item = V2> + '_ {
        match self {
            Segment::Line(line) => Either::Left([line.from, line.to].into_iter()),
            Segment::Bezier(bezier) => Either::Right(bezier.controls().iter().copied()),
        }
    }
    pub fn reversed(&self) -> Segment {
        match self {
            Segment::Line(line) => Segment::Line(Line::new(line.to, line.from)),
            Segment::Bezier(bezier) => Segment::Bezier(bezier.reversed()),
        }
    }
    pub(crate) fn map_points(&self, f: impl Fn(V2) -> V2) -> Segment {
        match self {
            Segment::Line(line) => Segment::Line(Line::new(f(line.from), f(line.to))),
            Segment::Bezier(bezier) => Segment::Bezier(bezier.map_points(f)),
        }
    }
}

impl From<Line> for Segment {
    fn from(line: Line) -> Self {
        Segment::Line(line)
    }
}

impl From<Bezier> for Segment {
    fn from(bezier: Bezier) -> Self {
        Segment::Bezier(bezier)
    }
}

/// Samples `curve` at `n + 1` parameters so the points are evenly spaced in arc length.
pub(crate) fn resample_even(
    curve: impl Fn(f32) -> V2,
    length: f32,
    sample_settings: SampleSettings,
) -> Vec<V2> {
    let n = sample_settings.get_num_points_for_length(length).max(1) as usize;
    let table_size = (4 * n).max(16);

    let table: Vec<V2> = (0..=table_size)
        .map(|i| curve(i as f32 / table_size as f32))
        .collect();
    let cumulative: Vec<f32> = std::iter::once(0.0)
        .chain(table.windows(2).scan(0.0, |acc, pair| {
            *acc += pair[0].dist(pair[1]);
            Some(*acc)
        }))
        .collect();
    let total = cumulative[table_size];
    if total <= 0.0 {
        return vec![table[0], table[table_size]];
    }

    let mut bracket = 0;
    (0..=n)
        .map(|i| {
            if i == n {
                return table[table_size];
            }
            let target = i as f32 / n as f32 * total;
            while bracket + 1 < table_size && cumulative[bracket + 1] < target {
                bracket += 1;
            }
            let span = cumulative[bracket + 1] - cumulative[bracket];
            let fraction = if span > 0.0 {
                (target - cumulative[bracket]) / span
            } else {
                0.0
            };
            let t = (bracket as f32 + fraction) / table_size as f32;
            curve(t)
        })
        .collect()
}
