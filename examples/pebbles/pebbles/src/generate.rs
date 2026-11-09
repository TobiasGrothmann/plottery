use plottery_lib::*;
use plottery_project::*;

// A grid of pebbles: closed Hobby splines through random points, with smaller rings inside.
#[derive(PlotteryParams)]
pub struct Params {
    #[value(Seed::new(7))]
    pub seed: Seed,

    /// paper size, orientation and margin
    pub paper: Paper,

    /// side length of a grid cell in cm
    #[value(3.0)]
    #[range(1.0, 8.0)]
    pub cell_size: f32,

    #[section("Pebbles")]
    /// points each outline passes through
    #[value(Range::new(4, 7))]
    #[range(3, 12)]
    pub knots: Range<i32>,

    /// distance of the knots from the cell center, as a fraction of half the cell
    #[value(Range::new(0.55, 0.95))]
    #[range(0.1, 1.0)]
    pub radius: Range<f32>,

    /// higher pulls the curve tighter around the knots
    #[value(1.0)]
    #[range(0.75, 4.0)]
    pub tension: f32,

    /// smaller copies inside each pebble
    #[value(5)]
    #[range(0, 15)]
    pub rings: i32,
}

pub fn generate(params: Params) -> Layer {
    params.seed.apply();
    let frame = params.paper.frame();
    let grid = Grid::with_cell_size(frame.inner_rect(), params.cell_size, 0.0);
    let sample_settings = SampleSettings::default();

    let pebbles = grid.iter().flat_map(|cell| {
        let half = cell.width() * 0.5;
        let num_knots = params.knots.sample();
        let start = Angle::rand();

        // one knot per sector around the center, so the outline never crosses itself
        let knots = (0..num_knots).map(|i| {
            let sector = (i as f32 + rand_range(-0.3, 0.3)) / num_knots as f32;
            let angle = start + Angle::from_rotations(sector);
            cell.center() + V2::polar(angle, params.radius.sample() * half)
        });
        let outline = Spline::through_closed_with_tension(knots, params.tension);

        // the rings shrink towards a point near the center
        let core = cell.center() + V2::random_unit_disk() * half * 0.15;
        (0..=params.rings).map(move |ring| {
            let scale = 1.0 - ring as f32 / (params.rings + 1) as f32;
            outline.scale_around(core, scale).to_path(sample_settings)
        })
    });

    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.push_many(pebbles);
    l.with_name("root").optimize_recursive()
}
