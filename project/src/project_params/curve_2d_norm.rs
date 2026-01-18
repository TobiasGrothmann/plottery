use anyhow::Result;
use plottery_lib::V2;
use serde::{Deserialize, Serialize};

/// A normalized 2D curve for transfer functions and easing curves.
///
/// Both x and y values are in the range \[0.0, 1.0\].
///
/// # Usage
///
/// Use `sample(x)` to get the y-value at any x position:
///
/// ```
/// use plottery_project::Curve2DNorm;
///
/// let curve = Curve2DNorm::default();
/// let y = curve.sample(0.5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Curve2DNorm {
    y_first: f32,
    points: Vec<V2>,
    y_last: f32,
}

impl Curve2DNorm {
    pub fn new(y_first: f32, mut points: Vec<V2>, y_last: f32) -> Self {
        let y_first = y_first.clamp(0.0, 1.0);
        let y_last = y_last.clamp(0.0, 1.0);

        for point in &mut points {
            point.x = point.x.clamp(0.0, 1.0);
            point.y = point.y.clamp(0.0, 1.0);
        }

        points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        points.retain(|p| p.x > 0.0 && p.x < 1.0);

        Self {
            y_first,
            points,
            y_last,
        }
    }

    pub fn iter_points(&self) -> impl Iterator<Item = V2> + '_ {
        std::iter::once(V2::new(0.0, self.y_first))
            .chain(self.points.iter().copied())
            .chain(std::iter::once(V2::new(1.0, self.y_last)))
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.points.len() + 2
    }

    /// Sample the curve at position x.
    ///
    /// Returns the y-value at the given x position using linear interpolation.
    /// Input x is clamped to \[0.0, 1.0\].
    pub fn sample(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);

        let mut prev_x = 0.0;
        let mut prev_y = self.y_first;

        for point in self.iter_points().skip(1) {
            if x <= point.x {
                let t = (x - prev_x) / (point.x - prev_x);
                return prev_y + (point.y - prev_y) * t;
            }
            prev_x = point.x;
            prev_y = point.y;
        }

        self.y_last
    }

    pub fn add_point(&mut self, point: V2) -> Result<()> {
        let clamped = V2::new(point.x.clamp(0.0, 1.0), point.y.clamp(0.0, 1.0));

        if clamped.x <= 0.0 || clamped.x >= 1.0 {
            return Err(anyhow::anyhow!(
                "Cannot add point at x={}, must be in range (0.0, 1.0)",
                clamped.x
            ));
        }

        let index = self
            .points
            .binary_search_by(|p| p.x.partial_cmp(&clamped.x).unwrap())
            .unwrap_or_else(|i| i);
        self.points.insert(index, clamped);
        Ok(())
    }

    pub fn remove_point_at(&mut self, index: usize) -> Result<()> {
        if index == 0 {
            return Err(anyhow::anyhow!(
                "Cannot remove point at index {}, endpoints are not removable",
                index
            ));
        }
        if index >= self.len() - 1 {
            return Err(anyhow::anyhow!(
                "Cannot remove point at index {} (of {}), endpoints are not removable",
                index,
                self.len()
            ));
        }

        let index_without_endpoints = index - 1;
        if index_without_endpoints < self.points.len() {
            self.points.remove(index_without_endpoints);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Cannot remove point at index {}, only {} points exist (excluding endpoints)",
                index,
                self.points.len()
            ))
        }
    }

    pub fn update_endpoint(&mut self, is_first: bool, y: f32) {
        if is_first {
            self.y_first = y.clamp(0.0, 1.0);
        } else {
            self.y_last = y.clamp(0.0, 1.0);
        }
    }

    pub fn update_point_y_norm(&mut self, index: usize, y: f32) -> Result<()> {
        if index >= self.points.len() {
            return Err(anyhow::anyhow!("Point index {} out of bounds", index));
        }

        self.points[index].y = y.clamp(0.0, 1.0);
        Ok(())
    }

    pub fn update_point_norm(&mut self, index: usize, new_loc_norm: V2) -> Result<()> {
        if index >= self.points.len() {
            return Err(anyhow::anyhow!("Point index {} out of bounds", index));
        }

        let x_min = if index == 0 {
            0.0
        } else {
            self.points[index - 1].x
        };
        let x_max = if index == self.points.len() - 1 {
            1.0
        } else {
            self.points[index + 1].x
        };

        self.points[index] = new_loc_norm.clamp(V2::new(x_min, 0.0), V2::new(x_max, 1.0));

        Ok(())
    }
}

impl Default for Curve2DNorm {
    fn default() -> Self {
        Self {
            y_first: 0.0,
            y_last: 1.0,
            points: vec![],
        }
    }
}
