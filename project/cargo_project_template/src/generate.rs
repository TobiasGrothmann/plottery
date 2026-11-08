use plottery_lib::*;
use plottery_project::*;

#[derive(PlotteryParams)]
pub struct Params {
    #[value(Seed::new(1))]
    pub seed: Seed,

    // paper size, orientation and margin
    pub paper: Paper,

    // each circle picks a random radius in this range
    #[value(Range::new(0.05, 0.15))]
    #[range(0.02, 0.35)]
    pub circle_size: Range<f32>,
}

pub fn generate(params: Params) -> Layer {
    params.seed.apply();

    let frame = params.paper.frame();
    let inner = frame.inner_rect();
    let spiral_center = frame.center();
    let max_distance = inner.max_dist_to_any_corner(spiral_center);

    // circles in a golden-angle spiral, keeping only those inside the frame
    let circles = (1..)
        .map(|i| {
            let angle = Angle::golden_ratio() * i as f32;
            let distance = (i as f32).sqrt() * 0.5;
            spiral_center + V2::polar(angle, distance)
        })
        .take_while(|pos| pos.dist(spiral_center) < max_distance)
        .filter(|pos| inner.contains_point(*pos))
        .map(|pos| Circle::new(pos, params.circle_size.sample()));

    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.push_many(circles);
    l.with_name("root").optimize_recursive()
}
