# Plottery

**Plottery** is a **genererative art engine** for **pen-plotters** written in Rust.

* [**plottery_lib**](./lib) ([crates.io](https://crates.io/crates/plottery_lib)): Core library – Contains the functionality to generate art. This includes shapes, geometry helpers, structs for composition and other tools.
* [**plottery_editor**](./editor): Plottery Editor ([releases](https://github.com/TobiasGrothmann/plottery/releases)) – The GUI Application to create, manage and run projects, preview artworks and control the plotter hardware.
* [**plottery_server**](./server): Controller for the pen-plotter hardware – The server offers a http interface to send tasks to execute on the hardware and controls the motors. This is highly customized to my personal DIY pen-plotter.
* [**plottery_cli**](./cli) ([crates.io](https://docs.rs/crate/plottery_cli/latest)): Command line interface to create and run projects.
* [**plottery_project**](./project) ([crates.io](https://docs.rs/plottery_project/latest/plottery_project/)): Library containing functionality to handle **Plottery** projects.
* [**plottery_fonts**](./fonts) ([crates.io](https://crates.io/crates/plottery_fonts)): 65 bundled single-line fonts for plotters (Hershey, EMS, Cutlings, Relief, ...) with their licences. Data only, usable without **Plottery**.

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

# Editor

The [Editor](./editor) lets you manage, preview and run **Plottery** projects.
<img src="https://github.com/user-attachments/assets/e5fdc4c5-a898-45eb-98c6-8065eeb9aa2e" />

# Projects

### Create

To create a new project either use the [editor](./editor) or the [command line interface (CLI)](./cli) like so:
```sh
cargo install plottery_cli
plottery new /path/to/folder awesome_project
```

The [editor install script](./editor/README.md#install-on-macos) also installs the CLI. From a checkout, `cargo install --path cli` installs the unreleased version.

Projects in **Plottery** are a [wrapper around a regular cargo project](./project). They support a couple of commands when run directly:

```sh
cargo run svg # run project and open the result as .svg
cargo run -- --help # get some info
```

The [editor](./editor) can also be used to run a project and tweak its input parameters.

### Parameters

Projects expose parameters as a struct:
```rust
#[derive(PlotteryParams)]
pub struct Params {
    paper: Paper,

    #[section("Points")]
    num_points: i32,
    /// distance between neighbouring points in cm
    #[value(2.0)]
    #[range(0.5, 10.0)]
    points_distance: f32,
}
```

Supported field types are `f32`, `i32`, `bool`, `String`, `Paper`, `Range<f32>`, `Range<i32>`,
`Angle`, `V2`, `V2i`, `Seed`, `ImagePath`, `SvgPath`, `NoiseSettings`, `Easing`, `ColorRgb`, `Curve2D`, `Curve2DNorm`, enums deriving
`PlotteryEnum`, nested structs deriving `PlotteryParams`, and `Option<T>` / `Vec<T>` of those.
`#[value(..)]` sets the default, `#[range(min, max)]` shows a slider (or bounds for `Range`).
`#[section("Name")]` starts a collapsible editor section that holds this and all following fields until the next `#[section]`;
the struct stays flat. `///` doc comments are shown as tooltips on the param labels.
Nested structs and `Option<Struct>` render as their own section, the latter with an on/off switch in the header.

### Generate

`generate.rs` defines the function used to generate the art. It receives an instance of `Params` as an argument and returns a [`Layer`](./lib/src/composition/layer.rs):

```rust
pub fn generate(params: Params) -> Layer {
    let mut l = Layer::new();

    /* Generate your art here! */

    l.with_name("root").optimize_recursive()
}
```

# Text & Fonts

Text is drawn with single-line fonts, so every glyph is a set of pen strokes. The [bundled fonts](./fonts) are embedded in the library and load with no setup:

```rust
let font = Font::bundled(BundledFont::HersheyScript);
let text = Text::new("Hello plotter", &font)
    .size(1.5) // em height in cm
    .in_rect(frame.inner_rect()) // wraps words at the rect's width
    .align(TextAlign::Center)
    .vertical_align(VerticalAlign::Center)
    .to_layer(SampleSettings::default())
    .expect("text");
l.push_layer(text);
```

Anchoring to a rect, wrapping, justify, fitting and rotation are shown in the [text layout samples](./lib/samples/text/README.md). `Font::load` reads SVG, TTF and OTF files, `Font::load_single_line` keeps the strokes of single-line TrueType fonts open, and `Font::system("Helvetica")` finds an installed font. All 65 bundled fonts are shown in the [fonts README](./fonts/README.md).

<img src="./fonts/samples/HersheyScript.svg" width="49%" /> <img src="./fonts/samples/EmsReadability.svg" width="49%" />
<img src="./fonts/samples/ReliefSingleLine.svg" width="49%" /> <img src="./fonts/samples/HersheyGothicEnglish.svg" width="49%" />

# Example

This project generates a spiral masked inside a rectangle:

<div align="center" >
  <img src="https://github.com/user-attachments/assets/8aa40c30-4c36-498a-87de-ca9f2cc3fdd3" width="50%" />
</div>

```rust
use plottery_lib::*;
use plottery_project::*;

// These project parameters are exposed in the Plottery Editor.
#[derive(PlotteryParams)]
pub struct Params {
    #[value(2_000)]
    num_points: i32,

    #[value(10.0)]
    rotations: f32,
}

pub fn generate(params: Params) -> Layer {
    let mut l = Layer::new();
    let size = V2::a6(); // DIN-A6 paper size
    let frame = Frame::new_xy(
        size,
        size.min_axis() * 0.1, // 10% margin of smallest side
    );

    // add outer border as a paper cutting guide
    l.push(frame.outer_rect());

    // generate spiral by collecting from iterator of points
    let path: Path = (0..params.num_points)
        .map(|i| {
            let angle =
                Angle::from_rotations(i as f32 / params.num_points as f32) * params.rotations;
            let radius = i as f32 * 0.005;
            V2::polar(angle, radius)
        })
        .collect::<Path>() // collect points into a Path
        .translate(frame.center()); // translate spiral to center of frame

    // mask path to the inner rect of the frame (outer frame without the margin)
    let masked_path = path.mask_brute_force(frame.inner_rect(), SampleSettings::default());

    // push all elements of inside the inner frame to `l` flat
    l.push_layer_flat(masked_path.inside);

    l.with_name("root").optimize_recursive()
}
