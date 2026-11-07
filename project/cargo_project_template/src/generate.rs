use plottery_lib::*;
use plottery_project::*;

#[derive(PlotteryParams)]
pub struct Params {
    #[value(0.1)]
    #[range(0.02, 0.35)]
    pub circle_size: f32,

    // paper size, orientation and margin
    pub paper: Paper,
}

pub fn generate(params: Params) -> Layer {
    let mut l = Layer::new();
    let frame = params.paper.frame();

    let spiral_center = frame.center();

    // create circles in a spiral
    let mut circles = vec![];

    let mut i = 0;
    let mut distance = 0.0;
    let mut angle = Angle::zero();
    while distance < frame.inner_rect().max_dist_to_any_corner(spiral_center) {
        i += 1;
        distance = (i as f32).sqrt() * 0.5;
        angle = (angle + Angle::golden_ratio()).mod_one_rotation();
        let pos = V2::polar(angle, distance);
        circles.push(Circle::new(spiral_center + pos, params.circle_size));
    }

    // generate plot with only the circles that fit in the frame
    l.push(frame.outer_rect());
    l.push_many(
        circles
            .into_iter()
            .filter(|circle| frame.inner_rect().contains_point(circle.center)),
    );

    l.with_name("root").optimize_recursive()
}
