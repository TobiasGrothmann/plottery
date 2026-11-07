use crate::{Bezier, Line, Segment, Spline, V2};

/// Collects pen commands into one `Spline` per contour.
pub(crate) struct SplineSink {
    contours: Vec<Spline>,
    segments: Vec<Segment>,
    start: Option<V2>,
    current: V2,
}

impl SplineSink {
    pub(crate) fn new() -> Self {
        Self {
            contours: Vec::new(),
            segments: Vec::new(),
            start: None,
            current: V2::zero(),
        }
    }

    fn flush(&mut self, closed: bool) {
        if !self.segments.is_empty() {
            self.contours.push(Spline::from_segments(
                std::mem::take(&mut self.segments),
                closed,
            ));
        }
        self.segments.clear();
        self.start = None;
    }

    /// Starts a new contour at `p`, flushing an open one.
    pub(crate) fn move_to(&mut self, p: V2) {
        self.flush(false);
        self.start = Some(p);
        self.current = p;
    }

    fn begin_if_needed(&mut self) {
        if self.start.is_none() {
            self.start = Some(self.current);
        }
    }

    /// Zero-length lines are skipped (fonts emit them).
    pub(crate) fn line_to(&mut self, p: V2) {
        self.begin_if_needed();
        if p != self.current {
            self.segments.push(Line::new(self.current, p).into());
        }
        self.current = p;
    }

    pub(crate) fn quad_to(&mut self, ctrl: V2, p: V2) {
        self.begin_if_needed();
        self.segments
            .push(Bezier::quadratic(self.current, ctrl, p).into());
        self.current = p;
    }

    pub(crate) fn cubic_to(&mut self, c1: V2, c2: V2, p: V2) {
        self.begin_if_needed();
        self.segments
            .push(Bezier::cubic(self.current, c1, c2, p).into());
        self.current = p;
    }

    /// Closes the contour with a straight segment back to its start if needed.
    pub(crate) fn close(&mut self) {
        let Some(start) = self.start else {
            return;
        };
        if self.current != start {
            self.segments.push(Line::new(self.current, start).into());
        }
        self.flush(true);
        self.current = start;
    }

    pub(crate) fn current(&self) -> V2 {
        self.current
    }

    /// Flushes a trailing open contour; empty contours are dropped.
    pub(crate) fn finish(mut self) -> Vec<Spline> {
        self.flush(false);
        self.contours
    }
}
