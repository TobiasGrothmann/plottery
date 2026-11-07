use serde::{Deserialize, Serialize};

use crate::{resample_even, SampleSettings, LARGE_EPSILON, V2};

/// Bézier curve of any degree: 2 controls is a line, 3 quadratic, 4 cubic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bezier {
    controls: Vec<V2>,
}

impl Bezier {
    /// Panics with fewer than 2 controls.
    pub fn new<I: IntoIterator<Item = V2>>(controls: I) -> Self {
        let controls: Vec<V2> = controls.into_iter().collect();
        assert!(controls.len() >= 2, "Bezier needs at least 2 controls");
        Self { controls }
    }
    pub fn quadratic(from: V2, ctrl: V2, to: V2) -> Self {
        Self {
            controls: vec![from, ctrl, to],
        }
    }
    pub fn cubic(from: V2, ctrl1: V2, ctrl2: V2, to: V2) -> Self {
        Self {
            controls: vec![from, ctrl1, ctrl2, to],
        }
    }

    pub fn controls(&self) -> &[V2] {
        &self.controls
    }
    pub fn degree(&self) -> usize {
        self.controls.len() - 1
    }
    pub fn from(&self) -> V2 {
        self.controls[0]
    }
    pub fn to(&self) -> V2 {
        self.controls[self.controls.len() - 1]
    }

    /// Point at `t` in `0..=1` (de Casteljau).
    pub fn point_at(&self, t: f32) -> V2 {
        de_casteljau(&self.controls, t)
    }

    /// Derivative at `t`, falls back to the chord direction where the derivative vanishes.
    pub fn tangent_at(&self, t: f32) -> V2 {
        let differences: Vec<V2> = self
            .controls
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect();
        let tangent = de_casteljau(&differences, t) * self.degree() as f32;
        if tangent.length_squared() > LARGE_EPSILON * LARGE_EPSILON {
            tangent
        } else {
            self.to() - self.from()
        }
    }

    /// Splits at `t` into two curves of the same degree.
    pub fn split(&self, t: f32) -> (Bezier, Bezier) {
        let mut left = Vec::with_capacity(self.controls.len());
        let mut right = Vec::with_capacity(self.controls.len());
        let mut level = self.controls.clone();
        while let (Some(first), Some(last)) = (level.first(), level.last()) {
            left.push(*first);
            right.push(*last);
            level = level
                .windows(2)
                .map(|pair| pair[0].lerp(pair[1], t))
                .collect();
        }
        right.reverse();
        (Bezier { controls: left }, Bezier { controls: right })
    }

    /// Same curve with one more control point.
    pub fn elevated(&self) -> Bezier {
        let n = self.controls.len() as f32;
        let inner = self.controls.windows(2).enumerate().map(|(i, pair)| {
            let alpha = (i + 1) as f32 / n;
            pair[0] * alpha + pair[1] * (1.0 - alpha)
        });
        let controls = std::iter::once(self.from())
            .chain(inner)
            .chain(std::iter::once(self.to()))
            .collect();
        Bezier { controls }
    }

    /// Arc length, computed by adaptive subdivision.
    pub fn length(&self) -> f32 {
        self.length_recursive(0)
    }

    fn length_recursive(&self, depth: usize) -> f32 {
        let chord = self.from().dist(self.to());
        let polygon: f32 = self
            .controls
            .windows(2)
            .map(|pair| pair[0].dist(pair[1]))
            .sum();
        if polygon - chord < 1e-4 * chord.max(LARGE_EPSILON) || depth >= 20 {
            return (polygon + chord) * 0.5;
        }
        let (left, right) = self.split(0.5);
        left.length_recursive(depth + 1) + right.length_recursive(depth + 1)
    }

    /// Points evenly spaced in arc length, including both ends.
    pub fn points(&self, sample_settings: SampleSettings) -> Vec<V2> {
        resample_even(|t| self.point_at(t), self.length(), sample_settings)
    }

    pub fn reversed(&self) -> Bezier {
        Bezier {
            controls: self.controls.iter().rev().copied().collect(),
        }
    }

    pub(crate) fn map_points(&self, f: impl Fn(V2) -> V2) -> Bezier {
        Bezier {
            controls: self.controls.iter().map(|p| f(*p)).collect(),
        }
    }
}

fn de_casteljau(controls: &[V2], t: f32) -> V2 {
    let mut level = controls.to_vec();
    while level.len() > 1 {
        level = level
            .windows(2)
            .map(|pair| pair[0].lerp(pair[1], t))
            .collect();
    }
    level[0]
}
