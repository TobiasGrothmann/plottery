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

    /// `sin` mapped from -1..1 to 0..1.
    fn sin01(self) -> Self {
        (self.sin() + Self::one()) / (Self::one() + Self::one())
    }
    /// `cos` mapped from -1..1 to 0..1.
    fn cos01(self) -> Self {
        (self.cos() + Self::one()) / (Self::one() + Self::one())
    }
    /// Clamps to 0..1.
    fn clamp01(self) -> Self {
        self.max(Self::zero()).min(Self::one())
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

/// `steps + 1` evenly spaced values from 0.0 to 1.0, both inclusive.
pub fn fractions(steps: usize) -> FloatInterpolator<f32> {
    0.0.lerp_iter_fixed(1.0, steps)
}

/// `steps` evenly spaced values from 0.0 up to but excluding 1.0, e.g. for closed loops.
pub fn fractions_exclusive(steps: usize) -> impl Iterator<Item = f32> {
    (0..steps).map(move |i| i as f32 / steps as f32)
}

/// `count` evenly spaced values from `start` to `end`, both inclusive.
pub fn linspace(start: f32, end: f32, count: usize) -> FloatInterpolator<f32> {
    start.lerp_iter_fixed(end, count.saturating_sub(1))
}
