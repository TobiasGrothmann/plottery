use std::{fmt, iter::once, sync::Arc};

use rand::{rngs::StdRng, SeedableRng};

use crate::{
    maths::random::rand_seed, Circle, Layer, Path, Plottable, SampleSettings, Shape, LARGE_EPSILON,
    V2,
};

use super::{blur_cull::cull_dots, blur_noise::gaussian_noise};

const DEFAULT_DENSITY: f32 = 1.9;
/// Otter's 0.01 cm dot at a 0.5 mm pen.
const DOT_RADIUS_PER_PEN_WIDTH: f32 = 0.2;
const MIN_PEN_WIDTH: f32 = 0.005;
const MIN_DENSITY: f32 = 0.1;
const MIN_SPACING: f32 = 0.001;
const MIN_DOT_RADIUS: f32 = 0.001;

/// A point along a shape before scattering, the input of [`Blur::with_amount_fn`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct BlurSample {
    pub point: V2,
    /// Distance along the shape in cm.
    pub distance: f32,
    /// `distance` as a fraction of the shape's length.
    pub fraction: f32,
}

/// How [`Blur`] scatters a sample by its amount σ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scatter {
    /// Random direction, distance `|N(0, σ)|`, like otter's blur.
    #[default]
    Otter,
    /// `N(0, σ)` on each axis, a true gaussian blur. Spreads wider than `Otter` at the same σ.
    Gaussian,
}

type AmountFn<'a> = Arc<dyn Fn(&BlurSample) -> f32 + Send + Sync + 'a>;

/// Scatters strokes into fuzzy fragments and dots, like a randomly sampled gaussian blur.
///
/// Samples every `pen_width / density` cm and moves each sample by noise scaled with the amount σ in cm.
/// Steps shorter than `pen_width` are joined into paths, lone samples become dots.
/// Apply it with [`Plottable::blur`] or [`Layer::blur`].
#[derive(Clone)]
pub struct Blur<'a> {
    pen_width: f32,
    amount: AmountFn<'a>,
    scatter: Scatter,
    density: f32,
    smoothness: f32,
    cull_distance: f32,
    dot_radius: Option<f32>,
    seed: Option<u64>,
}

impl<'a> Blur<'a> {
    /// `pen_width` in cm is the join distance and sets the sample spacing.
    pub fn new(pen_width: f32) -> Self {
        Self {
            pen_width: pen_width.max(MIN_PEN_WIDTH),
            amount: Arc::new(|_| 0.0),
            scatter: Scatter::default(),
            density: DEFAULT_DENSITY,
            smoothness: 0.0,
            cull_distance: 0.0,
            dot_radius: None,
            seed: None,
        }
    }

    /// Constant amount σ in cm.
    #[must_use]
    pub fn with_amount(self, amount: f32) -> Self {
        self.with_amount_fn(move |_| amount)
    }

    /// Amount σ in cm per sample. Runs in order on the calling thread, so noise and random helpers are safe to use.
    #[must_use]
    pub fn with_amount_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&BlurSample) -> f32 + Send + Sync + 'a,
    {
        self.amount = Arc::new(f);
        self
    }

    #[must_use]
    pub fn with_scatter(mut self, scatter: Scatter) -> Self {
        self.scatter = scatter;
        self
    }

    /// Samples per pen width, default 1.9. Below 1 even an unblurred line falls apart into dots.
    #[must_use]
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density.max(MIN_DENSITY);
        self
    }

    /// Correlation length of the scatter in cm. 0 is white noise, larger values wobble instead of fuzz.
    #[must_use]
    pub fn with_smoothness(mut self, smoothness: f32) -> Self {
        self.smoothness = smoothness.max(0.0);
        self
    }

    /// Drops dots closer than `min_distance` cm to a path or a kept dot of the same layer. 0 keeps all.
    #[must_use]
    pub fn with_cull(mut self, min_distance: f32) -> Self {
        self.cull_distance = min_distance.max(0.0);
        self
    }

    /// Radius of a dot in cm, default `0.2 × pen_width`.
    #[must_use]
    pub fn with_dot_radius(mut self, radius: f32) -> Self {
        self.dot_radius = Some(radius.max(MIN_DOT_RADIUS));
        self
    }

    /// Without a seed every blur draws one from the global RNG, see [`crate::seed`].
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}

impl Blur<'_> {
    pub(crate) fn blur_shape<P: Plottable>(&self, shape: &P) -> Layer {
        Layer::new_from(self.blur_shapes(std::slice::from_ref(shape), self.base_seed(), &mut 0))
    }

    pub(crate) fn blur_layer(&self, layer: &Layer) -> Layer {
        self.blur_layer_from(layer, self.base_seed(), &mut 0)
    }

    /// Shapes are numbered depth first, each gets its own seed from that number.
    fn blur_layer_from(&self, layer: &Layer, seed: u64, index: &mut u64) -> Layer {
        let shapes = self.blur_shapes(&layer.shapes, seed, index);
        let sublayers = layer
            .sublayers
            .iter()
            .map(|sublayer| self.blur_layer_from(sublayer, seed, index))
            .collect();
        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(layer.props_inheritable.clone())
            .with_props(layer.props.clone())
    }

    fn blur_shapes<P: Plottable>(&self, shapes: &[P], seed: u64, index: &mut u64) -> Vec<Shape> {
        let first = *index;
        *index += shapes.len() as u64;
        let blurred = shapes
            .iter()
            .zip(first..)
            .flat_map(|(shape, i)| self.blur_one(shape, shape_seed(seed, i)))
            .collect();
        if self.cull_distance > 0.0 {
            cull_dots(blurred, self.cull_distance, self.pen_width)
        } else {
            blurred
        }
    }

    fn blur_one<P: Plottable>(&self, shape: &P, seed: u64) -> Vec<Shape> {
        let Some(samples) = self.sample(shape) else {
            return Vec::new();
        };
        let mut rng = StdRng::seed_from_u64(seed);
        let noise = gaussian_noise(
            &mut rng,
            samples.samples.len(),
            samples.spacing,
            self.smoothness,
            samples.closed,
        );
        let scattered: Vec<V2> = samples
            .samples
            .iter()
            .zip(noise)
            .map(|(sample, g)| sample.point + self.scatter.unit_offset(g) * self.sigma(sample))
            .collect();
        self.join(&scattered, samples.closed)
    }

    fn sigma(&self, sample: &BlurSample) -> f32 {
        let sigma = (self.amount)(sample);
        if sigma.is_finite() {
            sigma.max(0.0)
        } else {
            0.0
        }
    }

    /// Even steps along the shape of at most `pen_width / density`. Closed shapes skip the repeated end.
    fn sample<P: Plottable>(&self, shape: &P) -> Option<Samples> {
        let max_spacing = (self.pen_width / self.density).max(MIN_SPACING);
        let points = shape.get_points(SampleSettings::new(1.0 / max_spacing));
        let lengths: Vec<f32> = points
            .windows(2)
            .map(|pair| pair[0].dist(pair[1]))
            .collect();
        let length: f32 = lengths.iter().sum();
        if !length.is_finite() || length <= LARGE_EPSILON {
            return None;
        }

        let closed = shape.is_closed();
        let steps = (length / max_spacing).ceil() as usize;
        let spacing = length / steps as f32;
        let count = if closed { steps } else { steps + 1 };
        let mut segment = 0;
        let mut segment_start = 0.0;
        let samples = (0..count)
            .map(|i| {
                let distance = i as f32 * spacing;
                while segment + 1 < lengths.len() && segment_start + lengths[segment] < distance {
                    segment_start += lengths[segment];
                    segment += 1;
                }
                let t = if lengths[segment] > 0.0 {
                    ((distance - segment_start) / lengths[segment]).min(1.0)
                } else {
                    0.0
                };
                BlurSample {
                    point: points[segment].lerp(points[segment + 1], t),
                    distance,
                    fraction: distance / length,
                }
            })
            .collect();
        Some(Samples {
            samples,
            spacing,
            closed,
        })
    }

    /// Splits where a step reaches a pen width, joining across the seam of closed shapes.
    fn join(&self, points: &[V2], closed: bool) -> Vec<Shape> {
        let max_step_squared = self.pen_width * self.pen_width;
        let joined = |from: V2, to: V2| from.dist_squared(to) < max_step_squared;
        let breaks: Vec<usize> = (1..points.len())
            .filter(|&i| !joined(points[i - 1], points[i]))
            .collect();
        let starts = once(0).chain(breaks.iter().copied());
        let ends = breaks.iter().copied().chain(once(points.len()));
        let fragments: Vec<&[V2]> = starts
            .zip(ends)
            .map(|(start, end)| &points[start..end])
            .collect();

        let wraps = closed && points.len() > 1 && joined(points[points.len() - 1], points[0]);
        match (wraps, fragments.as_slice()) {
            (true, [_]) => vec![Path::new_shape_from(points.iter().chain(once(&points[0])))],
            (true, [first, middle @ .., last]) => {
                once(Path::new_shape_from(last.iter().chain(*first)))
                    .chain(middle.iter().map(|fragment| self.fragment_shape(fragment)))
                    .collect()
            }
            _ => fragments
                .iter()
                .map(|fragment| self.fragment_shape(fragment))
                .collect(),
        }
    }

    fn fragment_shape(&self, fragment: &[V2]) -> Shape {
        match fragment {
            [point] => Circle::new_shape(*point, self.dot_radius()),
            _ => Path::new_shape_from(fragment),
        }
    }

    fn dot_radius(&self) -> f32 {
        self.dot_radius
            .unwrap_or(self.pen_width * DOT_RADIUS_PER_PEN_WIDTH)
            .max(MIN_DOT_RADIUS)
    }

    fn base_seed(&self) -> u64 {
        self.seed.unwrap_or_else(rand_seed)
    }
}

impl fmt::Debug for Blur<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Blur")
            .field("pen_width", &self.pen_width)
            .field("scatter", &self.scatter)
            .field("density", &self.density)
            .field("smoothness", &self.smoothness)
            .field("cull_distance", &self.cull_distance)
            .field("dot_radius", &self.dot_radius())
            .field("seed", &self.seed)
            .finish_non_exhaustive()
    }
}

struct Samples {
    samples: Vec<BlurSample>,
    spacing: f32,
    closed: bool,
}

/// splitmix64 of seed and index, so shapes don't share noise and keep it when other shapes change.
fn shape_seed(seed: u64, index: u64) -> u64 {
    let mut z = seed.wrapping_add(index.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
