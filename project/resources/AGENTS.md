# {{project-name}} – Plottery Project

A generative art generator for pen-plotters. `generate(params: Params) -> Layer` is called whenever the
parameters change in the Plottery editor; the returned `Layer` is the drawing.

**Layout:** `{{project-name}}/src/generate.rs` (the artwork), `resources/` (editor files: params, previews),
`{{project-name}}.plottery` (project config).

```rust
#[derive(PlotteryParams)]
pub struct Params {
    pub paper: Paper,       // use `params.paper.frame()` for the drawable area
    pub mode: Mode,

    #[section("Shapes")]    // starts a collapsible editor section for this and all following fields
    /// edge length in cm   // `///` doc comments show as tooltips in the editor
    #[value(10.0)]          // default (optional)
    #[range(0.0, 100.0)]    // slider min/max (optional)
    pub size: f32,
}

#[derive(PlotteryEnum, Clone, Copy, PartialEq)]
pub enum Mode { #[default] Lines, Dots }
```

**Field types:** `f32`, `i32`, `bool`, `String`, `Paper`, `Range<f32|i32>`, `Angle`, `V2`, `V2i`, `ColorRgb`, `Seed`, `ImagePath`, `SvgPath`, `NoiseSettings`, `Easing`, `Curve2D`, `Curve2DNorm`, `PlotteryEnum` enums, nested `PlotteryParams` structs, `Option<T>`, `Vec<T>`.

Group many params with `#[section("..")]` (the struct stays flat) instead of `// ---` comments, and describe params with `///` instead of `//`.
Nested structs and `Option<Struct>` show as their own section; use `Option<Struct>` for "on/off switch + its params".

With a `Seed` param, call `params.seed.apply()` first so randomness is reproducible.

`ImagePath::load()` returns a raster `Image`, `SvgPath::load()` returns the svg paths as a `Layer`.

Render without the editor: `cargo run -- svg` from inside `{{project-name}}/`.

Prefer idiomatic rust: iterators, builders. API docs: https://docs.rs/plottery_lib
