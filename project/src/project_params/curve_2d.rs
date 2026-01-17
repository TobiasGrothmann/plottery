use anyhow::Result;
use plottery_lib::maths::FloatInterpolation;
use plottery_lib::V2;
use serde::{Deserialize, Serialize};

use super::curve_2d_norm::Curve2DNorm;

/// A 2D domain defining x and y ranges.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Domain {
    pub x_start: f32,
    pub x_end: f32,
    pub y_start: f32,
    pub y_end: f32,
}

impl Domain {
    pub fn new(x_start: f32, x_end: f32, y_start: f32, y_end: f32) -> Self {
        Self {
            x_start,
            x_end,
            y_start,
            y_end,
        }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            x_start: 0.0,
            x_end: 1.0,
            y_start: 0.0,
            y_end: 1.0,
        }
    }
}

/// A 2D curve with domain mapping.
///
/// This wraps a `Curve2DNorm` (which operates in normalized [0,1] space)
/// and maps it to a custom domain via the `mapped_to` field.
///
/// # Usage
///
/// ```
/// use plottery_project::project_params::curve_2d::{Curve2D, Domain};
///
/// let curve = Curve2D::new(Domain::new(0.0, 10.0, 0.0, 100.0));
/// let y = curve.sample(5.0); // Sample at x=5.0 in the [0,10] range
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Curve2D {
    curve: Curve2DNorm,
    pub mapped_to: Domain,
}

impl Curve2D {
    /// Create a new Curve2D with the specified domain.
    pub fn new(mapped_to: Domain) -> Self {
        Self {
            curve: Curve2DNorm::default(),
            mapped_to,
        }
    }

    /// Create a Curve2D from an existing Curve2DNorm and domain.
    pub fn from_curve(curve: Curve2DNorm, mapped_to: Domain) -> Self {
        Self { curve, mapped_to }
    }

    /// Sample the curve at position x.
    ///
    /// Input x is mapped from [mapped_to.x_start, mapped_to.x_end] to [0.0, 1.0] for sampling,
    /// then the result is mapped from [0.0, 1.0] to [mapped_to.y_start, mapped_to.y_end].
    pub fn sample(&self, x: f32) -> f32 {
        // Map input x from [mapped_to.x_start, mapped_to.x_end] to [0.0, 1.0]
        let normalized_x = x
            .linlin(self.mapped_to.x_start, self.mapped_to.x_end, 0.0, 1.0)
            .clamp(0.0, 1.0);

        // Sample the normalized curve
        let normalized_y = self.curve.sample(normalized_x);

        // Map output y from [0.0, 1.0] to [mapped_to.y_start, mapped_to.y_end]
        normalized_y.linlin(0.0, 1.0, self.mapped_to.y_start, self.mapped_to.y_end)
    }

    /// Get a reference to the underlying normalized curve.
    pub fn curve(&self) -> &Curve2DNorm {
        &self.curve
    }

    /// Get a mutable reference to the underlying normalized curve.
    pub fn curve_mut(&mut self) -> &mut Curve2DNorm {
        &mut self.curve
    }

    /// Get an iterator over all points (including endpoints) in normalized space.
    pub fn iter_points(&self) -> impl Iterator<Item = V2> + '_ {
        self.curve.iter_points()
    }

    /// Get the total number of points including endpoints.
    pub fn len(&self) -> usize {
        self.curve.len()
    }

    /// Add a point to the curve in normalized coordinates.
    pub fn add_point(&mut self, point: V2) -> Result<()> {
        self.curve.add_point(point)
    }

    /// Remove a point from the curve by index (not including endpoints).
    pub fn remove_point(&mut self, index: usize) -> Result<()> {
        self.curve.remove_point(index)
    }

    /// Update an endpoint's y value in normalized coordinates.
    pub fn update_endpoint(&mut self, is_first: bool, y: f32) {
        self.curve.update_endpoint(is_first, y)
    }

    /// Update a point's y value in normalized coordinates.
    pub fn update_point(&mut self, index: usize, y: f32) -> Result<()> {
        self.curve.update_point(index, y)
    }

    /// Update a point's x and y values in normalized coordinates.
    pub fn update_point_xy(&mut self, index: usize, x: f32, y: f32) -> Result<()> {
        self.curve.update_point_xy(index, x, y)
    }
}

impl Default for Curve2D {
    fn default() -> Self {
        Self {
            curve: Curve2DNorm::default(),
            mapped_to: Domain::default(),
        }
    }
}
