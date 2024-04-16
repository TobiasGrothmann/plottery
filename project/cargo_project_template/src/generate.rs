use plottery_lib::*;

pub fn define_parameters() -> Vec<ProjectParam> {
    vec![ProjectParam {
        name: "number of circles".to_string(),
        category: "general".to_string(),
        value: ProjectParamValue::Float(0.0),
    }]
}

pub fn generate(params: HashMap<String, ProjectParam>) -> Layer {
    let mut l = Layer::new();

    // generate your art here:
    // ...

    for i in 0..params["number of circles"].get_float() as i32 {
        l.push(Circle::new_shape(
            V2::new((i as f32 * 0.5).sin(), i as f32 * 0.2) + V2::new(1.0, 1.0),
            0.2,
        ));
    }

    l.push(Path::new_shape_from(vec![
        V2::new(0.0, 0.0),
        V2::new(3.0, 5.0),
    ]));

    l.push(Rect::new_shape(V2::new(1.0, 1.0), V2::new(2.0, 5.0)));

    l
}
