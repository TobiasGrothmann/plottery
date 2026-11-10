use std::f32::consts::TAU;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{maths::RNG, Angle, Plottable, Rect, SpatialGrid, V2};

/// Evenly spaced random points (blue noise) in a rect, with an optional spacing that varies by position.
///
/// Spacing ratios `max_dist / min_dist` above about 5 get slow; split such fills into one sampler per spacing range.
pub struct PoissonDisk<'a> {
    bounds: Rect,
    min_dist: f32,
    max_dist: f32,
    tries: usize,
    dist_fn: Option<Box<dyn Fn(V2) -> f32 + 'a>>,
}

impl<'a> PoissonDisk<'a> {
    /// No two points closer than `min_dist`.
    pub fn new(bounds: Rect, min_dist: f32) -> Self {
        assert!(min_dist > 0.0, "min_dist must be positive, got {min_dist}");
        Self {
            bounds,
            min_dist,
            max_dist: min_dist,
            tries: 30,
            dist_fn: None,
        }
    }

    /// Spacing at each point from `dist`, clamped to `min_dist..=max_dist`.
    #[must_use]
    pub fn with_dist_fn(mut self, max_dist: f32, dist: impl Fn(V2) -> f32 + 'a) -> Self {
        self.max_dist = max_dist.max(self.min_dist);
        self.dist_fn = Some(Box::new(dist));
        self
    }

    /// Point density from `density` in `0..=1`: 1 gives `min_dist`, 0 gives `max_dist`.
    #[must_use]
    pub fn with_density_fn(self, max_dist: f32, density: impl Fn(V2) -> f32 + 'a) -> Self {
        let min_dist = self.min_dist;
        self.with_dist_fn(max_dist, move |p| min_dist / density(p).max(0.0).sqrt())
    }

    /// Candidates tried around each point before it is done, default 30.
    #[must_use]
    pub fn with_tries(mut self, tries: usize) -> Self {
        self.tries = tries;
        self
    }

    /// The points in insertion order, all inside the bounds.
    pub fn points(&self) -> Vec<V2> {
        // one draw from the global RNG keeps it reproducible, a local RNG keeps the loop lock-free
        let mut rng = StdRng::seed_from_u64(RNG.lock().expect("Failed to acquire RNG lock").gen());
        let first = self.bounds.bl() + self.bounds.size() * V2::new(rng.gen(), rng.gen());
        let mut points = vec![first];
        let mut radii = vec![self.dist_at(first)];
        let mut grid = SpatialGrid::new(self.min_dist);
        grid.insert(first, radii[0]);
        let mut active = vec![0];

        while !active.is_empty() {
            let slot = rng.gen_range(0..active.len());
            let (center, radius) = (points[active[slot]], radii[active[slot]]);
            let accepted = (0..self.tries)
                .map(|_| {
                    let angle = Angle::from_rad(rng.gen_range(0.0..TAU));
                    center + V2::polar(angle, rng.gen_range(radius..2.0 * radius))
                })
                .filter(|candidate| self.bounds.contains_point(*candidate))
                .map(|candidate| (candidate, self.dist_at(candidate)))
                .find(|&(candidate, candidate_radius)| {
                    self.fits(&grid, candidate, candidate_radius)
                });
            match accepted {
                Some((point, point_radius)) => {
                    active.push(points.len());
                    points.push(point);
                    radii.push(point_radius);
                    grid.insert(point, point_radius);
                }
                None => {
                    active.swap_remove(slot);
                }
            }
        }
        points
    }

    fn dist_at(&self, point: V2) -> f32 {
        // max/min instead of clamp so a NaN spacing falls back to min_dist
        self.dist_fn.as_ref().map_or(self.min_dist, |dist| {
            dist(point).max(self.min_dist).min(self.max_dist)
        })
    }

    /// Every neighbour is at least the mean of both spacings away.
    fn fits(&self, grid: &SpatialGrid<f32>, candidate: V2, radius: f32) -> bool {
        grid.within(candidate, (radius + self.max_dist) * 0.5).all(
            |(neighbour, neighbour_radius)| {
                let needed = (radius + neighbour_radius) * 0.5;
                candidate.dist_squared(neighbour) >= needed * needed
            },
        )
    }
}
