use plottery_lib::*;
use plottery_project::*;

#[derive(Debug, Clone, PlotteryParamsDefinition)]
pub struct Params {
    #[value(0.2)]
    #[range(0.2, 0.8)]
    pub circle_size: f32,

    #[value(10)]
    #[range(5, 500)]
    pub num_circles: i32,
}

pub fn generate(params: Params) -> Layer {
    let mut l = Layer::new();

    for i in 0..params.num_circles {
        l.push(Circle::new_shape(
            V2::new((i as f32 * 0.5).sin(), i as f32 * 0.2) + V2::new(1.0, 1.0),
            params.circle_size,
        ));
    }

    l.push(Path::new_shape_from(vec![
        V2::new(0.0, 0.0),
        V2::new(3.0, 5.0),
    ]));

    l.push(Rect::new_shape(V2::new(1.0, 1.0), V2::new(2.0, 5.0)));

    l
}
