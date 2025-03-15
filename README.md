# Plottery

**Plottery** is a genererative art engine for pen-plotters written in rust.

# Projects

### Create

To create a new project either use the [editor](./editor) or the [command line interface (CLI)](./cli) like so:
```sh
cargo install plottery_cli
plottery new /path/to/folder awesome_project
```

Projects in **Plottery** are a wrapper around a regular cargo project. They offer different commands when run directly:

```sh
cargo run svg # run project and open the result as .svg
cargo run -- --help # get some info
```

The [editor](./editor) can also be used to run projects and tweak their input parameters.

### Parameters

Projects expose parameters used to generate in a struct:
```rs
#[derive(PlotteryParamsDefinition)]
pub struct Params {
    num_points: i32,
    points_distance: f32,
}
```

### Generate

`generate.rs` defines the function used to generate the art. It receives an instance of [`Params`] as an argument and returns a [`Layer`](./lib/src/composition/layer.rs):

```rs
pub fn generate(params: Params) -> Layer {
    let mut l = Layer::new();

    /* Generate your art here! */

    l.with_name("root").optimize_recursive()
}
```

# Example

This project generates a spiral masked inside a rectangle:

<div align="center" >
  <img src="https://github.com/user-attachments/assets/8aa40c30-4c36-498a-87de-ca9f2cc3fdd3" width="50%" />
</div>

```rs
use plottery_lib::*;
use plottery_project::*;

// These project parameters are exposed in the Plottery Editor.
#[derive(PlotteryParamsDefinition)]
pub struct Params {
    #[value(2_000)]
    num_points: i32,

    #[value(10.0)]
    rotations: f32,
}

pub fn generate(params: Params) -> Layer {
    let mut l = Layer::new();
    let size = V2::a6(); // DIN-A6 paper size
    let frame = Frame::new(
        size,
        size.min_axis() * 0.1, // 10% margin of smallest side
    );

    // add outer border as a paper cutting guide
    l.push_rect(frame.outer_rect());

    // generate spiral by collecting from iterator of points
    let path: Path = (0..params.num_points)
        .map(|i| {
            let angle =
                Angle::from_rotations(i as f32 / params.num_points as f32) * params.rotations;
            let radius = i as f32 * 0.005;
            V2::polar(angle, radius)
        })
        .collect::<Path>() // collect points into a Path
        .translate(&frame.center()); // translate spiral to center of frame

    // mask path to the inner rect of the frame (outer frame without the margin)
    let masked_path = path.mask_brute_force(&frame.inner_rect().into(), &SampleSettings::default());

    // push all elements of inside the inner frame to `l` flat
    l.push_layer_flat(masked_path.inside);

    l.with_name("root").optimize_recursive()
}
```
