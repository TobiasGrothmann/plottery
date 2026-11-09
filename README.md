# Plottery

[![crates.io](https://img.shields.io/crates/v/plottery_lib.svg)](https://crates.io/crates/plottery_lib)
[![docs.rs](https://img.shields.io/docsrs/plottery_lib)](https://docs.rs/plottery_lib)
[![license](https://img.shields.io/badge/license-GPL--3.0-blue)](./LICENSE)
[![editor release](https://img.shields.io/github/v/release/TobiasGrothmann/plottery?label=editor)](https://github.com/TobiasGrothmann/plottery/releases/latest)

**Plottery** is a **generative art engine** for **pen-plotters** written in Rust.

* [**plottery_lib**](./lib) ([crates.io](https://crates.io/crates/plottery_lib), [docs.rs](https://docs.rs/plottery_lib)): Core library with shapes, geometry helpers, composition and other tools to generate art.
* [**plottery_editor**](./editor) ([crates.io](https://crates.io/crates/plottery_editor), [releases](https://github.com/TobiasGrothmann/plottery/releases)): The macOS app to create, manage and run projects, preview artworks and control the plotter.
* [**plottery_cli**](./cli) ([crates.io](https://crates.io/crates/plottery_cli)): Command line interface to create, run and render projects.
* [**plottery_project**](./project) ([crates.io](https://crates.io/crates/plottery_project), [docs.rs](https://docs.rs/plottery_project)): Project handling and the `Params` derive macros.
* [**plottery_fonts**](./fonts) ([crates.io](https://crates.io/crates/plottery_fonts), [docs.rs](https://docs.rs/plottery_fonts)): 65 bundled single-line fonts for plotters (Hershey, EMS, Cutlings, Relief, ...) with their licences. Data only, usable without **Plottery**.
* [**plottery_server**](./server): The controller of my DIY pen-plotter. It serves an HTTP interface for plot tasks and drives the motors.

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

# Install

On macOS, this installs the newest editor and, if `cargo` is installed, the matching [`plottery` CLI](./cli):

```sh
curl -fsSL https://raw.githubusercontent.com/TobiasGrothmann/plottery/main/scripts/install_macos.sh | bash
```

CLI only: `cargo install plottery_cli`.

# Editor

The [editor](./editor) lets you manage, preview and run **Plottery** projects.

<img width="100%" src="./editor/screenshots/editor.png" />

# Projects

A project is a [regular cargo package](./project) that depends on **Plottery**. Create one with the [editor](./editor) or the [CLI](./cli):

```sh
plottery new /path/to/folder awesome_project
```

Its `src/generate.rs` turns a `Params` struct into a `Layer`:

```rust
use plottery_lib::*;
use plottery_project::*;

#[derive(PlotteryParams)]
pub struct Params {
    pub paper: Paper,
    #[value(12)]
    #[range(1, 50)]
    pub circles: i32,
}

pub fn generate(params: Params) -> Layer {
    let frame = params.paper.frame();
    let step = frame.inner_rect().width() * 0.5 / params.circles as f32;
    let circles = (1..=params.circles).map(|i| Circle::new(frame.center(), step * i as f32));

    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.push_many(circles);
    l.with_name("root").optimize_recursive()
}
```

Run it in the [editor](./editor), with `plottery run`, or with `cargo run svg` to open the result as `.svg`.

### Examples

<a href="./examples/spiral"><img src="https://github.com/user-attachments/assets/8aa40c30-4c36-498a-87de-ca9f2cc3fdd3" width="38%" /></a>
<a href="./examples/scribbles"><img src="./examples/scribbles/preview.png" width="19%" /></a>
<a href="./examples/pebbles"><img src="./examples/pebbles/preview.png" width="19%" /></a>
<a href="./examples/poster"><img src="./examples/poster/preview.png" width="19%" /></a>

* [**spiral**](./examples/spiral/spiral/src/generate.rs): a path collected from polar points, masked to the margin, in named and colored sublayers.
* [**scribbles**](./examples/scribbles/scribbles/src/generate.rs): squares that dissolve into scribbles, with a seed, a noise param, an easing curve and random ranges.
* [**pebbles**](./examples/pebbles/pebbles/src/generate.rs): closed Hobby splines through random points, with rings inside.
* [**poster**](./examples/poster/poster/src/generate.rs): a title fitted to the page width and a wrapped paragraph, fonts and text set in the editor.

Every example is a complete project that builds against this repository: in a clone, open its folder in the editor or run `plottery run examples/spiral`.

### Parameters

Every field of `Params` shows up in the editor. `#[value(..)]` sets the default, `#[range(min, max)]` shows a slider, `#[section("..")]` groups fields and `///` comments become tooltips. All attributes and supported types are listed in the [project README](./project/README.md#parameters), and the [examples](#examples) use many of them.

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

Anchoring to a rect, wrapping, justify, fitting and rotation are shown in the [text layout samples](./lib/samples/text/README.md). `Font::load` reads SVG, TTF and OTF files, `Font::load_single_line` keeps the strokes of single-line TrueType fonts open, and `Font::system("Helvetica")` finds an installed font. The [poster](./examples/poster/poster/src/generate.rs) example picks its fonts and text in the editor. All 65 bundled fonts are shown in the [fonts README](./fonts/README.md).

<img src="./fonts/samples/HersheyScript.svg" width="49%" /> <img src="./fonts/samples/EmsReadability.svg" width="49%" />
<img src="./fonts/samples/ReliefSingleLine.svg" width="49%" /> <img src="./fonts/samples/HersheyGothicEnglish.svg" width="49%" />

# Development

Building from source, tests and releasing are described in [DEVELOPMENT.md](./DEVELOPMENT.md).

# License

[GPL-3.0](./LICENSE). The bundled fonts have their own licences, see [plottery_fonts](./fonts/README.md#licences).
