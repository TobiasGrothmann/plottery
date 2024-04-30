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

    pub fn get_intersection_line(&self, target_value: f32) -> Option<Line> {
        if target_value < self.min_value || target_value > self.max_value {
            return None;
        }

        let num_above = [self.a, self.b, self.c]
            .iter()
            .fold(0, |acc, point: &SamplePoint| {
                acc + if point.value > target_value { 1 } else { 0 }
            });
        if num_above == 0 || num_above == 3 {
            return None;
        }

        let mut points = Vec::with_capacity(2);
        for i in 0..3 {
            let a = [self.a, self.b, self.c][i];
            let b = [self.a, self.b, self.c][(i + 1) % 3];
            if a.value > target_value && b.value > target_value
                || a.value < target_value && b.value < target_value
            {
                continue;
            }
            let err_a = (target_value - a.value).abs();
            let err_b = (target_value - b.value).abs();
            let t = err_a / (err_a + err_b);
            points.push(a.pos.lerp(&b.pos, t));
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
