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
- `#[range(min, max)]` turns a number into a slider and bounds a `Range`, also inside an `Option`.
- `#[section("Name")]` starts a collapsible section. It holds this field and all following ones until the next `#[section]`. The struct stays flat.
- `///` doc comments are shown as tooltips on the param labels.
- Field names show lowercase with spaces (`circle_size` → "circle size"). Enum variants split CamelCase (`SomeVariant` → "some variant").

Field types and their widgets:

| Field type | Editor widget |
|---|---|
| `f32`, `i32` | number box; slider with `#[range]` |
| `bool` | switch |
| `String` | multi-line text box |
| `Paper` | DIN A/B or custom size, orientation, margin in cm or as a fraction |
| `Range<f32>`, `Range<i32>` | min and max boxes, clamped to `#[range]` |
| `Angle` | number with a deg/rot/rad unit switch |
| `V2`, `V2i` | x and y boxes; a `V2` can also be picked by clicking the drawing |
| `Seed` | number, dice button, recent seeds |
| `Easing` | power, bias or s-curve, slider, curve preview |
| `ColorRgb` | color picker and hex code |
| `NoiseSettings` | perlin/simplex/worley, frequency per cm or per paper, octaves |
| `BundledFont` | font picker with previews |
| `ImagePath`, `SvgPath` | file picker; copies the file into `resources/` or links the original |
| `Curve2DNorm` | curve editor on 0..1, double-click adds or removes points |
| `Curve2D` | same curve plus the x and y range it maps to |
| enum deriving `PlotteryEnum` (`#[default]` on one variant) | buttons up to four variants, dropdown beyond |
| struct deriving `PlotteryParams` | its own collapsible section |
| `Option<T>` | switch beside the widget, or in the section header for a struct; off hides it |
| `Vec<T>` | list with add/remove; new items start at `T`'s default (a struct's `#[value]`s) |

`Option<Option<T>>`, `Vec<Option<T>>` and `Vec<Vec<T>>` don't compile. `Option<Vec<T>>` works.
