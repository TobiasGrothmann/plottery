use crate::{Line, V2};

#[derive(Debug, Clone, Copy)]
pub struct SamplePoint {
    pub pos: V2,
    pub value: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SampleTriangle {
    pub a: SamplePoint,
    pub b: SamplePoint,
    pub c: SamplePoint,
    pub min_value: f32,
    pub max_value: f32,
}

impl SampleTriangle {
    pub fn new(a: SamplePoint, b: SamplePoint, c: SamplePoint) -> Self {
        let min_value = a.value.min(b.value).min(c.value);
        let max_value = a.value.max(b.value).max(c.value);
        Self {
            a,
            b,
            c,
            min_value,
            max_value,
        }
    }

    /// Segment of the iso-line at `target_value` through this triangle, if it crosses it.
    pub fn get_intersection_line(&self, target_value: f32) -> Option<Line> {
        if target_value < self.min_value || target_value > self.max_value {
            return None;
        }

        // A value exactly on the threshold counts as above, so it never yields a degenerate crossing.
        let corners = [self.a, self.b, self.c];
        let above = |point: &SamplePoint| point.value >= target_value;
        let num_above = corners.iter().filter(|point| above(point)).count();
        if num_above == 0 || num_above == 3 {
            return None;
        }

        let mut points = Vec::with_capacity(2);
        for i in 0..3 {
            let (mut a, mut b) = (corners[i], corners[(i + 1) % 3]);
            if above(&a) == above(&b) {
                continue;
            }
            // Interpolate from the lower value so neighbouring triangles get the identical point.
            if a.value > b.value {
                std::mem::swap(&mut a, &mut b);
            }
            let t = (target_value - a.value) / (b.value - a.value);
            points.push(a.pos.lerp(b.pos, t));
        }

        if points[0] == points[1] {
            // the iso-line only touches a corner
            return None;
        }
        Some(Line::new(points[0], points[1]))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SampleRect {
    pub triangles: [SampleTriangle; 4],
}

impl SampleRect {
    pub fn new(
        tl: SamplePoint,
        tr: SamplePoint,
        br: SamplePoint,
        bl: SamplePoint,
        center: SamplePoint,
    ) -> Self {
        let triangles = [
            SampleTriangle::new(tl, tr, center),
            SampleTriangle::new(tr, br, center),
            SampleTriangle::new(br, bl, center),
            SampleTriangle::new(bl, tl, center),
        ];
        Self { triangles }
    }
}
