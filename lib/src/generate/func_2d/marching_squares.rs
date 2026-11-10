use crate::{Layer, Line, Rect, SampleSettings, V2};

use super::stitch::{stitch, Crossing, Segment};

/// Iso-lines of a scalar field sampled over `bounds`. The field is sampled once in [`MarchingSquares::new`].
pub struct MarchingSquares {
    bounds: Rect,
    nx: usize,
    ny: usize,
    /// Corner samples, row by row from the bottom.
    values: Vec<f32>,
}

/// Cell sides.
const B: usize = 0;
const R: usize = 1;
const T: usize = 2;
const L: usize = 3;

/// Crossed side pairs per case, with bits bl = 1, br = 2, tr = 4, tl = 8 set for corners above the level.
/// The saddles 5 and 10 cut off br and tl here, [`SADDLE_FLIPPED`] cuts off bl and tr.
const CASES: [&[[usize; 2]]; 16] = [
    &[],
    &[[L, B]],
    &[[B, R]],
    &[[L, R]],
    &[[R, T]],
    &[[B, R], [T, L]],
    &[[B, T]],
    &[[L, T]],
    &[[T, L]],
    &[[B, T]],
    &[[B, R], [T, L]],
    &[[R, T]],
    &[[L, R]],
    &[[B, R]],
    &[[L, B]],
    &[],
];
const SADDLE_FLIPPED: &[[usize; 2]] = &[[L, B], [R, T]];

impl MarchingSquares {
    pub fn new<F: Fn(V2) -> f32>(
        bounds: Rect,
        sample_settings: SampleSettings,
        function: F,
    ) -> Self {
        let num = (bounds.size() * sample_settings.points_per_unit).ceil_to_int();
        let grid = Self {
            bounds,
            nx: num.x as usize,
            ny: num.y as usize,
            values: Vec::new(),
        };
        let values = (0..=grid.ny)
            .flat_map(|j| (0..=grid.nx).map(move |i| (i, j)))
            .map(|corner| function(grid.corner_pos(corner)))
            .collect();
        Self { values, ..grid }
    }

    /// Line segments where the field crosses `target_value`, unordered.
    pub fn lines(&self, target_value: f32) -> Vec<Line> {
        self.segments(target_value)
            .map(|Segment([a, b])| Line::new(a.pos, b.pos))
            .filter(|line| line.from != line.to)
            .collect()
    }

    /// Iso-lines at `target_value` joined into continuous paths.
    #[must_use]
    pub fn paths(&self, target_value: f32) -> Layer {
        Layer::new_from(stitch(self.segments(target_value).collect()))
    }

    fn corner_pos(&self, (i, j): (usize, usize)) -> V2 {
        V2::new(i as f32 / self.nx as f32, j as f32 / self.ny as f32) * self.bounds.size()
            + self.bounds.bl()
    }

    fn value(&self, (i, j): (usize, usize)) -> f32 {
        self.values[j * (self.nx + 1) + i]
    }

    fn segments(&self, level: f32) -> impl Iterator<Item = Segment> + '_ {
        (0..self.ny)
            .flat_map(move |j| (0..self.nx).map(move |i| (i, j)))
            .flat_map(move |(i, j)| {
                let corners = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)]
                    .map(|corner| self.value(corner));
                // A value equal to the level counts as above.
                let case: usize = (0..4)
                    .filter(|&bit| corners[bit] >= level)
                    .map(|bit| 1 << bit)
                    .sum();
                let mean_above = corners.iter().sum::<f32>() / 4.0 >= level;
                let pairs = match case {
                    5 | 10 if (case == 5) != mean_above => SADDLE_FLIPPED,
                    _ => CASES[case],
                };
                pairs.iter().map(move |&[a, b]| {
                    Segment([self.crossing(i, j, a, level), self.crossing(i, j, b, level)])
                })
            })
    }

    /// Crossing on `side` of cell `(i, j)`. Horizontal edges are numbered first, then vertical ones.
    fn crossing(&self, i: usize, j: usize, side: usize, level: f32) -> Crossing {
        let vertical_offset = self.nx * (self.ny + 1);
        let (edge, a, b) = match side {
            B => (j * self.nx + i, (i, j), (i + 1, j)),
            T => ((j + 1) * self.nx + i, (i, j + 1), (i + 1, j + 1)),
            L => (vertical_offset + j * (self.nx + 1) + i, (i, j), (i, j + 1)),
            _ => (
                vertical_offset + j * (self.nx + 1) + i + 1,
                (i + 1, j),
                (i + 1, j + 1),
            ),
        };
        // Interpolating from the lower value gives both cells of an edge the identical point.
        let (low, high) = if self.value(a) <= self.value(b) {
            (a, b)
        } else {
            (b, a)
        };
        let t = (level - self.value(low)) / (self.value(high) - self.value(low));
        Crossing {
            edge,
            pos: self.corner_pos(low).lerp(self.corner_pos(high), t),
        }
    }
}
