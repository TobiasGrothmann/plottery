# Plottery

**Plottery** is a **genererative art engine** for **pen-plotters** written in Rust.

* [**plottery_lib**](./lib) ([crates.io](https://crates.io/crates/plottery_lib)): Core library – Contains the functionality to generate art. This includes shapes, geometry helpers, structs for composition and other tools.
* [**plottery_editor**](./editor): Plottery Editor ([releases](https://github.com/TobiasGrothmann/plottery/releases)) – The GUI Application to create, manage and run projects, preview artworks and control the plotter hardware.
* [**plottery_server**](./server): Controller for the pen-plotter hardware – The server offers a http interface to send tasks to execute on the hardware and controls the motors. This is highly customized to my personal DIY pen-plotter.
* [**plottery_cli**](./cli) ([crates.io](https://docs.rs/crate/plottery_cli/latest)): Command line interface to create and run projects.
* [**plottery_project**](./project) ([crates.io](https://docs.rs/plottery_project/latest/plottery_project/)): Library containing functions and tools to handle **Plottery** projects.

<br/>

<img src="https://github.com/user-attachments/assets/2c9b8785-5c05-45ed-a600-78e6745d751b" width="24%" />
<img src="https://github.com/user-attachments/assets/35af3c86-744a-43a0-8ceb-d81f6b636df1" width="24%" />
<img src="https://github.com/user-attachments/assets/3a7c9629-53da-4e5b-b066-5aa1c4d690da" width="24%" />
<img src="https://github.com/user-attachments/assets/158e20c1-e47e-4761-9cd7-4c19440e6218" width="24%" />
<img src="https://github.com/user-attachments/assets/9eef5667-a12d-4828-9d7a-542f53a05553" width="32%" />
<img src="https://github.com/user-attachments/assets/53f7fe21-4cd4-425b-8ca7-b8774c218d3d" width="32%" />
<img src="https://github.com/user-attachments/assets/7a70875e-5313-44c5-974e-9132feb5af13" width="32%" />
<img src="https://github.com/user-attachments/assets/1fb9d9f2-1a31-43c6-90ea-fab8513b320d" width="24%" />
<img src="https://github.com/user-attachments/assets/c6d8ca23-a703-401a-9c68-6b9d021d3309" width="24%" />
<img src="https://github.com/user-attachments/assets/6a951a31-559c-42c3-b04e-73137538b988" width="24%" />
<img src="https://github.com/user-attachments/assets/ee24c85c-e22a-4ca7-8d91-6314b1ffe7de" width="24%" />

<br/>

# Projects

### Create

To create a new project either use the [editor](./editor) or the [command line interface (CLI)](./cli) like so:
```sh
cargo install plottery_cli
plottery new /path/to/folder awesome_project
```

Projects in **Plottery** are a wrapper around a regular cargo project. They support a couple of commands when run directly:

```sh
cargo run svg # run project and open the result as .svg
cargo run -- --help # get some info
```

The [editor](./editor) can also be used to run a project and tweak its input parameters.

### Parameters

Projects expose parameters as a struct:
```rs
#[derive(PlotteryParamsDefinition)]
pub struct Params {
    num_points: i32,
    points_distance: f32,
}
```

### Generate

`generate.rs` defines the function used to generate the art. It receives an instance of `Params` as an argument and returns a [`Layer`](./lib/src/composition/layer.rs):

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
