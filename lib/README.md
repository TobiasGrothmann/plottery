# Plottery Lib

Part of [**Plottery**](https://github.com/TobiasGrothmann/plottery), a generative art engine for pen-plotters.

Geometry, composition and art generation library. Everything is in `cm`, every shape is a set of pen strokes.

## Geometry

```rust
use plottery_lib::*;

// vector operations
let v1 = V2::new(5.0, 0.0) + V2::polar(Angle::from_degrees(60.0), 3.0);
let length = v1.length();

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

// fast neighbour queries over many points, each with a value
let points = [V2::new(1.0, 2.0), V2::new(4.0, 0.5), V2::new(2.5, 2.5)];
let grid: SpatialGrid<usize> = points.iter().copied().zip(0..).collect();
let close = grid.within(V2::new(2.0, 2.0), 1.0).count();
let nearest = grid.nearest(V2::zero());
```

## Shapes

```rust
use plottery_lib::*;

let circle = Circle::new(V2::zero(), 5.0);
let rect = Rect::new(V2::zero(), V2::new(10.0, 5.0));
let path: Path = [
    V2::zero(),
    V2::new(10.0, 0.0),
    V2::new(10.0, 20.0),
    V2::new(0.0, 5.0),
]
.into_iter()
.collect();

// every transformation has an immutable and a `_mut` variant
let moved = circle.translate(V2::new(2.0, 0.0));
let rotated = path.rotate(Angle::from_degrees(45.0));

// the `Shape` enum holds any shape
let shapes: Vec<Shape> = vec![circle.into(), rect.into(), moved.into(), rotated.into()];
```

## Layers

A `Layer` holds shapes and sublayers. `optimize_recursive` orders the paths to keep pen travel short.

```rust
use plottery_lib::*;

let frame = Frame::new_xy(V2::a6(), 1.0); // A6 paper with a 1 cm margin

let circles: Layer = (1..10)
    .map(|i| Circle::new_shape(frame.center(), i as f32 * 0.5))
    .collect();

let mut l = Layer::new();
l.push(frame.outer_rect());
l.push_layer(circles);

let l = l.with_name("root").optimize_recursive();
l.write_svg(std::env::temp_dir().join("circles.svg"))
    .expect("svg");
```

## Masking

Cut any shape at the outline of another. `inside` and `outside` are layers.

```rust
use plottery_lib::*;

let frame = Frame::new_xy(V2::a6(), 1.0);
let spiral: Path = (0..2_000)
    .map(|i| {
        let angle = Angle::from_rotations(i as f32 / 200.0);
        frame.center() + V2::polar(angle, i as f32 * 0.005)
    })
    .collect();

let masked = spiral.mask_brute_force(frame.inner_rect(), SampleSettings::default());
let mut l = Layer::new();
l.push_layer_flat(masked.inside);
```

## Blur

`Blur` scatters strokes into fuzzy fragments and dots, like a drawing out of focus. The amount is a
distance in cm and can change along each shape.

```rust
use plottery_lib::*;

let frame = Frame::new_xy(V2::a6(), 1.0);
let rings: Layer = (1..12)
    .map(|i| Circle::new_shape(frame.center(), i as f32 * 0.4))
    .collect();

let focus = frame.center().y;
let blur = Blur::new(0.05) // pen width in cm
    .with_amount_fn(move |sample| 0.08 * (sample.point.y - focus).abs())
    .with_seed(1);
let blurred = rings.blur(&blur);
```

## Text

Text is drawn with single-line fonts. 65 of them are bundled via [`plottery_fonts`](https://crates.io/crates/plottery_fonts). Layout, wrapping and fitting are shown in the [text samples](https://github.com/TobiasGrothmann/plottery/blob/main/lib/samples/text/README.md).
