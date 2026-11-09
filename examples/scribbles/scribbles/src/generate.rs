use plottery_lib::*;
use plottery_project::*;

// A grid of squares that dissolve into noisy scribbles down the page.
#[derive(PlotteryParams)]
pub struct Params {
    #[value(Seed::new(3))]
    pub seed: Seed,

    /// paper size, orientation and margin
    pub paper: Paper,

    #[section("Grid")]
    /// side length of a grid cell in cm
    #[value(1.2)]
    #[range(0.4, 4.0)]
    pub cell_size: f32,

    /// each square fills this fraction of its cell
    #[value(0.6)]
    #[range(0.1, 1.0)]
    pub square_size: f32,

    /// random turn of each square in rotations, fully applied at the bottom row
    #[value(Range::new(-0.15, 0.15))]
    #[range(-0.5, 0.5)]
    pub turn: Range<f32>,

    #[section("Scribble")]
    /// noise that pushes the outline points around
    #[value(NoiseSettings::default().with_frequency(1.5).with_octaves(2))]
    pub noise: NoiseSettings,

    /// largest push in cm, reached at the bottom row
    #[value(0.8)]
    #[range(0.0, 3.0)]
    pub strength: f32,

    /// how quickly the squares dissolve, 1 is linear from top to bottom
    #[value(2.0)]
    #[range(0.5, 5.0)]
    pub falloff: f32,
}

pub fn generate(params: Params) -> Layer {
    params.seed.apply();
    let noise = params.noise.noise_for(params.paper.size());
    let frame = params.paper.frame();
    let inner = frame.inner_rect();
    let grid = Grid::with_cell_size(inner, params.cell_size, 0.0);
    let sample_settings = SampleSettings::new(40.0);

    let squares = grid.iter().map(|cell| {
        // 0 at the top row, 1 at the bottom
        let t = ((inner.tl().y - cell.center().y) / inner.height()).powf(params.falloff);
        let turn = Angle::from_rotations(params.turn.sample() * t);
        Rect::new_from_center(cell.center(), cell.size() * params.square_size)
            .rotated(turn)
            .distort(sample_settings, |point| {
                let direction = Angle::from_rotations(noise.get(point) * 4.0);
                let distance = noise.get_3d(point, 10.0) * params.strength * t;
                point + V2::polar(direction, distance)
            })
    });

    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.push_many(squares);
    l.with_name("root").optimize_recursive()
}
