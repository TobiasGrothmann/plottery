use crate::{Line, Rect, SampleSettings, V2};

use super::sample_triangle::{SamplePoint, SampleRect};

pub struct MarchingSquares {
    rects: Vec<SampleRect>,
}

impl MarchingSquares {
    pub fn new(bounds: Rect, sample_settings: SampleSettings, function: fn(V2) -> f32) -> Self {
        let num = (bounds.size() * sample_settings.points_per_unit).ceil_to_int();
        let mut rects: Vec<SampleRect> =
            Vec::with_capacity((num.x as usize + 1) * (num.y as usize + 1));

        let mut last_row: Vec<Option<SamplePoint>> = vec![None; num.x as usize + 1];
        for y in 0..(num.y as usize + 1) {
            let mut new_row: Vec<Option<SamplePoint>> = vec![None; num.x as usize + 1];
            for x in 0..(num.x as usize + 1) {
                let pos_br = (V2::new(x as f32, y as f32) / V2::new(num.x as f32, num.y as f32))
                    * bounds.size()
                    + bounds.bl();
                let br = SamplePoint {
                    pos: pos_br,
                    value: function(pos_br),
                };

                if y > 0 && x > 0 {
                    let pos_center = pos_br - V2::new(0.5, 0.5) / sample_settings.points_per_unit;
                    let center = SamplePoint {
                        pos: pos_center,
                        value: function(pos_center),
                    };

                    rects.push(SampleRect::new(
                        last_row[x - 1].unwrap(),
                        last_row[x].unwrap(),
                        br,
                        new_row[x - 1].unwrap(),
                        center,
                    ));
                }

                new_row[x] = Some(br);
            }
            last_row = new_row;
        }

        Self { rects }
    }

    pub fn get_lines(&self, target_value: f32) -> Vec<Line> {
        let mut lines = Vec::new();
        for rect in &self.rects {
            for triangle in &rect.triangles {
                if let Some(line) = triangle.get_intersection_line(target_value) {
                    lines.push(line);
                }
            }
        }
        lines
    }
}
