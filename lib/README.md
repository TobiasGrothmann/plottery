<!-- cargo-rdme start -->

# Plottery Lib

Geometry, composition and art generation utility library for [**Plottery**](https://github.com/TobiasGrothmann/plottery).

### Geometry
```rust
use plottery_lib::*;

// vector operations
let v1 = V2::new(5.0, 0.0) + V2::polar(Angle::from_degrees(60.0), 3.0);
let length = v1.len();

let v2 = V2::xy(10.0) * 2.0;
let distance = v2.dist(v1);

// transformation matrix
let matrix = TransformMatrix::builder()
    .scale(2.0)
    .rotate(Angle::quarter_rotation())
    .translate(V2::new(5.0, 10.0))
    .mirror_y()
    .build();
let transformed_v = matrix.mul_vector(v1);
```

### Shapes
```rust
use plottery_lib::*;

// basic shapes
let circle = Circle::new(V2::zero(), 5.0);
let rect = Rect::new(V2::zero(), V2::new(10.0, 5.0));
let path = Path::new_from(vec![
    V2::zero(),
    V2::new(10.0, 0.0),
    V2::new(10.0, 20.0),
    V2::new(0.0, 5.0),
    V2::zero(),
]);

// Shapes can be handled generally with the `Shape` enum.
let vector_of_shapes: Vec<Shape> = vec![
    circle.into(),
    rect.into(),
    Circle::new_shape(V2::new(10.0, 10.0), 5.0),
];

// Compose shapes using `Layer`.
let mut main = Layer::new_from(vec![
   Rect::new_shape(V2::zero(), V2::xy(20.0)),
]);

let mut circles = Layer::new();
for i in 1..10 {
    circles.push(Circle::new_shape(V2::xy(10.0), i as f32));
}

// `main` now contains a bounding rectangle and a sublayer of concentric circles.
main.push_layer(circles);
```

<!-- cargo-rdme end -->
