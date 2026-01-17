use num_traits::{Float, FromPrimitive};

/// Iterator for interpolating between two float values.
pub struct FloatInterpolator<T> {
    start: T,
    end: T,
    steps: usize,
    current_step: usize,
}

impl<T: FloatInterpolation> FloatInterpolator<T> {
    /// Creates a new interpolator between two float values.
    pub fn new(start: T, end: T, steps: usize) -> Self {
        Self {
            start,
            end,
            steps,
            current_step: 0,
        }
    }
}

impl<T: FloatInterpolation> Iterator for FloatInterpolator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_step > self.steps {
            return None;
        }
        let t = T::from_usize(self.current_step).unwrap() / T::from_usize(self.steps).unwrap();
        let interpolated = self.start.lerp(self.end, t);
        self.current_step += 1;
        Some(interpolated)
    }
}

/// Trait for interpolating between float values.
pub trait FloatInterpolation: Float + FromPrimitive {
    /// Linearly interpolates between self and end with the given parameter t.
    fn lerp(self, end: Self, t: Self) -> Self {
        self * (Self::one() - t) + end * t
    }

    /// Creates an iterator that interpolates between self and end with the given step size.
    fn lerp_iter(self, end: Self, step_size: Self) -> FloatInterpolator<Self> {
        let steps = ((end - self) / step_size).abs().round().to_usize().unwrap();
        FloatInterpolator::new(self, end, steps)
    }

    /// Creates an iterator that interpolates between self and end with a fixed number of steps.
    fn lerp_iter_fixed(self, end: Self, steps: usize) -> FloatInterpolator<Self> {
        FloatInterpolator::new(self, end, steps)
    }

    /// Maps a value from one linear range to another.
    ///
    /// # Example
    /// ```
    /// use plottery_lib::maths::FloatInterpolation;
    /// let x = 5.0_f32;
    /// let y = x.linlin(0.0, 10.0, 0.0, 100.0); // Maps 5.0 from [0,10] to [0,100]
    /// assert_eq!(y, 50.0);
    /// ```
    fn linlin(self, from_start: Self, from_end: Self, to_start: Self, to_end: Self) -> Self {
        let normalized = (self - from_start) / (from_end - from_start);
        to_start + normalized * (to_end - to_start)
    }
}

impl FloatInterpolation for f32 {}

impl FloatInterpolation for f64 {}
