/// A value [`Interpolator`] can step through, computing the fraction in its own precision.
pub trait Interpolate: Copy {
    /// The value `step / steps` of the way from `self` to `end`; `steps` is never 0.
    fn interpolate(self, end: Self, step: usize, steps: usize) -> Self;
}

/// `steps + 1` evenly spaced values from `start` to `end`, both inclusive; only `start` if `steps` is 0.
pub struct Interpolator<T> {
    start: T,
    end: T,
    steps: usize,
    current_step: usize,
}

impl<T: Interpolate> Interpolator<T> {
    pub fn new(start: T, end: T, steps: usize) -> Self {
        Self {
            start,
            end,
            steps,
            current_step: 0,
        }
    }
}

impl<T: Interpolate> Iterator for Interpolator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.current_step > self.steps {
            return None;
        }
        let value = if self.steps == 0 {
            self.start
        } else {
            self.start
                .interpolate(self.end, self.current_step, self.steps)
        };
        self.current_step += 1;
        Some(value)
    }
}
