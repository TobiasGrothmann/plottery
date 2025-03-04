use plottery_lib::*;
use plottery_project::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PlotteryParamsDefinition, Serialize, Deserialize, PartialEq)]
pub struct Params {
    #[value(0.1)]
    #[range(0.02, 0.35)]
    pub circle_size: f32,

    #[value(2)]
    #[range(0, 6)]
    // e.g. A0, A1, A2, ...
    pub din_size: i32,
}

pub fn generate(params: Params) -> Layer {
    // setup
    let mut l = Layer::new();
    let size = V2::din_a(params.din_size as u8);
    let frame = Frame::new(size, size.min_axis() * 0.12);

    // create circles in a spiral
    let mut circles = vec![];

    let mut i = 0;
    let mut distance = 0.0;
    let mut angle = Angle::zero();
    while distance < size.max_axis() * 1.5 {
        i += 1;
        distance = (i as f32).sqrt() * 0.5;
        angle = (angle + Angle::golden_ratio()).mod_one_rotation();
        let pos = V2::polar(angle, distance);
        circles.push(Circle::new(size * 0.5 + pos, params.circle_size));
    }

    // generate plot with only the circles that fit in the frame
    l.push_rect(frame.outer_rect());
    l.push_many(
        circles
            .iter()
            .filter(|circle| frame.inner_rect().contains_point(&circle.center))
            .map(|circle| circle.to_shape()),
    );

    l
}
