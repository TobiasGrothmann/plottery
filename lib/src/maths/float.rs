pub struct FloatInterpolator {
    start: f32,
    end: f32,
    steps: usize,
    current_step: usize,
}

impl FloatInterpolator {
    pub fn new(start: f32, end: f32, steps: usize) -> Self {
        Self {
            start,
            end,
            steps,
            current_step: 0,
        }
    }
}

impl Iterator for FloatInterpolator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_step > self.steps {
            return None;
        }
        let t = self.current_step as f32 / self.steps as f32;
        let interpolated = self.start * (1.0 - t) + self.end * t;
        self.current_step += 1;
        Some(interpolated)
    }
}

pub trait FloatInterpolation {
    fn lerp(&self, end: f32, t: f32) -> f32;
    fn lerp_iter(&self, end: f32, t: f32) -> FloatInterpolator;
    fn lerp_iter_fixed(&self, end: f32, steps: usize) -> FloatInterpolator;
}

impl FloatInterpolation for f32 {
    fn lerp(&self, end: f32, t: f32) -> f32 {
        self * (1.0 - t) + end * t
    }

    fn lerp_iter(&self, end: f32, step_size: f32) -> FloatInterpolator {
        let steps = ((end - *self) / step_size).abs().round() as usize;
        FloatInterpolator::new(*self, end, steps)
    }

    fn lerp_iter_fixed(&self, end: f32, steps: usize) -> FloatInterpolator {
        FloatInterpolator::new(*self, end, steps)
    }
}
