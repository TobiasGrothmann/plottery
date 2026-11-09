# Plottery Project

Part of [**Plottery**](https://github.com/TobiasGrothmann/plottery), a generative art engine for pen-plotters.

Creates, builds and runs **Plottery** projects, and defines their parameters. A project is a regular cargo package:

<img alt="folder structure" src="https://github.com/user-attachments/assets/d0066b16-5b75-4563-b625-88cc7d206a77" />

Create one with the [editor](https://github.com/TobiasGrothmann/plottery/tree/main/editor) or `plottery new` from the [CLI](https://crates.io/crates/plottery_cli).

Complete example projects with rendered previews are in [`examples/`](https://github.com/TobiasGrothmann/plottery/tree/main/examples).

## Generate

`src/generate.rs` turns a `Params` struct into a `Layer`:

```rust
use plottery_lib::*;
use plottery_project::*;

#[derive(PlotteryParams)]
pub struct Params {
    pub paper: Paper,
}

pub fn generate(params: Params) -> Layer {
    let frame = params.paper.frame();
    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.with_name("root").optimize_recursive()
}
```

`src/main.rs` hands it to `run_project`, which the editor and CLI call. Run directly, `cargo run svg` opens the result as `.svg` and `cargo run -- --help` lists the other commands.

```rust,ignore
fn main() -> Result<(), Box<dyn std::error::Error>> {
    plottery_project::run_project(generate)
}
```

## Parameters

Every field of `Params` shows up in the editor:

```rust
use plottery_lib::*;
use plottery_project::*;

#[derive(PlotteryEnum)]
pub enum Style {
    #[default]
    Circles,
    Squares,
}

#[derive(PlotteryParams)]
pub struct Params {
    paper: Paper,
    style: Style,

    #[section("Points")]
    num_points: i32,
    /// distance between neighbouring points in cm
    #[value(2.0)]
    #[range(0.5, 10.0)]
    points_distance: f32,
}
```

- `#[value(..)]` sets the default.
- `#[range(min, max)]` shows a slider, or sets the bounds of a `Range`.
- `#[section("Name")]` starts a collapsible section. It holds this field and all following ones until the next `#[section]`. The struct stays flat.
- `///` doc comments are shown as tooltips on the param labels.

Supported field types:

- `f32`, `i32`, `bool`, `String`
- `Paper`, `Range<f32>`, `Range<i32>`, `Angle`, `V2`, `V2i`, `Seed`, `Easing`, `ColorRgb`, `NoiseSettings`, `BundledFont`
- `ImagePath`, `SvgPath`, `Curve2D`, `Curve2DNorm`
- enums deriving `PlotteryEnum`, with `#[default]` on one variant
- nested structs deriving `PlotteryParams`. They render as their own section.
- `Option<T>` and `Vec<T>` of the above. `Option<Struct>` gets an on/off switch in its section header.
