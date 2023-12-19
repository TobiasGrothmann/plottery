use plottery_lib::{Layer, Circle, V2};

pub fn generate() -> Layer {
    let mut l = Layer::new();

    // generate art here:
    // ...

    for i in 0..100 {
        l.push(Circle::new(V2::new((i as f32 * 0.5).sin(), i as f32 * 0.2), 0.5));
    }

    l
}

