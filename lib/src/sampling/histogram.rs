use crate::{Rect, V2};

pub const HISTOGRAM_BINS: usize = 256;

/// Distribution of values in `0..=1`, e.g. image lightness, noise or any scalar field.
///
/// Gives statistics (`min`, `mean`, `quantile`, `cdf`, ...) and remappings
/// (`equalize_fn`, `stretch_fn`). Values outside `0..=1` are clamped into the edge bins.
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// let noise = Noise::simplex();
/// let bounds = Rect::new(V2::zero(), V2::new(100.0, 50.0));
/// let histogram = Histogram::sample_field(bounds, 128, noise.as_fn());
/// let median = histogram.median();
/// let normalized = histogram.stretch_fn(0.0, 1.0); // noise min..max → 0..1
/// let quintiles = histogram.levels(5); // 4 values splitting the noise into 5 equal parts
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Histogram {
    bins: Vec<usize>,
    total: usize,
    min: f32,
    max: f32,
}

impl Histogram {
    /// Empty histogram, bin `i` covers `i/num_bins..(i+1)/num_bins`.
    pub fn new(num_bins: usize) -> Self {
        Self {
            bins: vec![0; num_bins.max(1)],
            total: 0,
            min: f32::INFINITY,
            max: f32::NEG_INFINITY,
        }
    }

    pub fn from_values<I: IntoIterator<Item = f32>>(values: I) -> Self {
        Self::from_values_binned(values, HISTOGRAM_BINS)
    }

    pub fn from_values_binned<I: IntoIterator<Item = f32>>(values: I, num_bins: usize) -> Self {
        values
            .into_iter()
            .fold(Self::new(num_bins), |mut histogram, value| {
                histogram.add(value);
                histogram
            })
    }

    /// Samples `f` on a grid over `bounds` with at most `max_samples_per_side` samples along the longer side,
    /// the shorter side gets proportionally fewer (at least 1).
    pub fn sample_field<F: Fn(V2) -> f32>(bounds: Rect, max_samples_per_side: usize, f: F) -> Self {
        let size = bounds.size();
        let n_long = max_samples_per_side.max(1);
        let n_short = ((n_long as f32 * size.min_axis() / size.max_axis()).round() as usize).max(1);
        let (n_x, n_y) = if size.x >= size.y {
            (n_long, n_short)
        } else {
            (n_short, n_long)
        };
        let cell_centers = |n: usize| (0..n).map(move |i| (i as f32 + 0.5) / n as f32);
        let f = &f;
        Self::from_values(
            cell_centers(n_y).flat_map(|v| {
                cell_centers(n_x).map(move |u| f(bounds.bl() + size * V2::new(u, v)))
            }),
        )
    }

    pub fn add(&mut self, value: f32) {
        let value = value.clamp(0.0, 1.0);
        let bin = ((value * self.bins.len() as f32) as usize).min(self.bins.len() - 1);
        self.bins[bin] += 1;
        self.total += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn bins(&self) -> &[usize] {
        &self.bins
    }

    pub fn total(&self) -> usize {
        self.total
    }

    /// Smallest added value, 0 when empty.
    pub fn min(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.min
        }
    }

    /// Largest added value, 0 when empty.
    pub fn max(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.max
        }
    }

    /// Mean of the bin centers, 0 when empty.
    pub fn mean(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let sum: f32 = self
            .bins
            .iter()
            .enumerate()
            .map(|(i, &count)| self.bin_center(i) * count as f32)
            .sum();
        sum / self.total as f32
    }

    pub fn median(&self) -> f32 {
        self.quantile(0.5)
    }

    /// Fraction of samples below `value` (0..=1), linear inside the bin. 0 when empty.
    pub fn cdf(&self, value: f32) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let position = value.clamp(0.0, 1.0) * self.bins.len() as f32;
        let bin = (position as usize).min(self.bins.len() - 1);
        let fraction_in_bin = position - bin as f32;
        let below: usize = self.bins[..bin].iter().sum();
        (below as f32 + fraction_in_bin * self.bins[bin] as f32) / self.total as f32
    }

    /// Value below which the fraction `p` of samples lies, linear inside the bin. Inverse of [`Histogram::cdf`].
    /// `0.0` and `1.0` give the exact min and max, 0 when empty.
    pub fn quantile(&self, p: f32) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        if p <= 0.0 {
            return self.min;
        }
        if p >= 1.0 {
            return self.max;
        }
        let target = p * self.total as f32;
        let mut below = 0;
        for (i, &count) in self.bins.iter().enumerate() {
            if count > 0 && (below + count) as f32 >= target {
                let fraction_in_bin = (target - below as f32) / count as f32;
                return (i as f32 + fraction_in_bin) / self.bins.len() as f32;
            }
            below += count;
        }
        self.max
    }

    /// `num_levels - 1` values that split the samples into `num_levels` equally populated parts.
    pub fn levels(&self, num_levels: usize) -> Vec<f32> {
        (1..num_levels)
            .map(|i| self.quantile(i as f32 / num_levels as f32))
            .collect()
    }

    /// Maps a value through the CDF so the field becomes uniformly distributed (histogram equalization).
    pub fn equalize_fn(&self) -> impl Fn(f32) -> f32 + '_ {
        move |value| self.cdf(value)
    }

    /// Contrast stretch: `quantile(low)..quantile(high)` → 0..1, clamped. `(0.0, 1.0)` is a plain min/max stretch.
    pub fn stretch_fn(&self, low: f32, high: f32) -> impl Fn(f32) -> f32 {
        let (from, to) = (self.quantile(low), self.quantile(high));
        let range = to - from;
        move |value| {
            if range <= 0.0 {
                0.0
            } else {
                ((value - from) / range).clamp(0.0, 1.0)
            }
        }
    }

    fn bin_center(&self, bin: usize) -> f32 {
        (bin as f32 + 0.5) / self.bins.len() as f32
    }
}
