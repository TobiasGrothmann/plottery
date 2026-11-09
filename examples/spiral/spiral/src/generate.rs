use plottery_lib::*;
use plottery_project::*;

// A spiral path masked to the inside of the paper margin.
#[derive(PlotteryParams)]
pub struct Params {
    /// paper size, orientation and margin
    #[value(Paper::din_a(6).with_landscape(true))]
    pub paper: Paper,

    #[section("Spiral")]
    /// points along the whole spiral
    #[value(2000)]
    #[range(100, 10000)]
    pub num_points: i32,

    /// turns from the center to the outer end
    #[value(10.0)]
    #[range(1.0, 40.0)]
    pub rotations: f32,

    /// radius of the outer end in cm, the margin cuts it off
    #[value(10.0)]
    #[range(1.0, 20.0)]
    pub radius: f32,
}

pub fn generate(params: Params) -> Layer {
    let frame = params.paper.frame();

    // collect polar points into a path, the radius grows with every point
    let spiral: Path = (0..params.num_points)
        .map(|i| {
            let t = i as f32 / params.num_points as f32;
            let angle = Angle::from_rotations(t * params.rotations);
            frame.center() + V2::polar(angle, t * params.radius)
        })
        .collect();

    // split the spiral at the inner rect and keep the parts inside
    let masked = spiral.mask_brute_force(frame.inner_rect(), SampleSettings::default());

    let mut l = Layer::new();
    l.push_layer(
        Layer::new_from([frame.outer_rect()])
            .with_name("frame")
            .with_color(ColorRgb::gray_scale(0.6)),
    );
    l.push_layer(
        masked
            .inside
            .with_name("spiral")
            .with_color(ColorRgb::new(0.1, 0.25, 0.6)),
    );
    l.with_name("root").optimize_recursive()
}
