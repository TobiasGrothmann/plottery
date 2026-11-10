# {{project-name}} – Plottery Project

A generative artwork for pen plotters. `generate(params: Params) -> Layer` returns the drawing.

## How the user works with this project

The user mostly works in the Plottery editor, not in the code:

- The project browser shows all projects as tiles with their latest drawing.
- An open project shows the drawing, a Parameters panel, a Layers panel and a Console.
- The Parameters panel has one widget per `Params` field: sliders, checkboxes, dropdowns, curve editors,
  file pickers. `#[section]`s fold, `///` comments show as tooltips. A `V2` can also be set by clicking
  the drawing.
- Every param change re-runs `generate`, so keep it fast.
- With Hot reload on, saving a file in `src/` rebuilds and redraws. The Run button does it once.
- The Layers panel shows the layer tree with names from `with_name` and colors. The user can hide
  sublayers there.
- The Console shows build output and anything the project prints with `println!` or `eprintln!`.
- From the editor the user sends the visible layers to the pen plotter.

So `Params` are the user's controls. Whatever they would want to tune while looking at the drawing
should be a param with a fitting widget, range and tooltip, not a constant they have to ask you to change.

The plotter ignores color and pen width; they only affect the preview and svg. For several pens, put
each pen into its own named sublayer. The user plots them one after another by hiding the others.

## Layout

- `{{project-name}}/src/generate.rs`: `Params` and `generate`. Start here.
- `{{project-name}}/Cargo.toml`: depends on `plottery_lib` (geometry) and `plottery_project` (params, runner).
- `{{project-name}}.plottery`: project config. The `plottery` CLI finds it from any folder inside the project.
- `resources/images/`, `resources/svgs/`: files the user picked for `ImagePath` and `SvgPath` params,
  copied in by the editor. The params hold paths relative to the project root.
- `resources/` otherwise: editor state (the user's param values, plot settings, preview, thumbnail),
  mostly binary. Don't edit it by hand.
- The project root is its own git repository. Commit there, not in Plottery.

## Build and render with the `plottery` CLI

Run it anywhere inside the project, it finds `{{project-name}}.plottery` upwards (or pass `-p <path>`):

```sh
plottery build                                    # compile, shows warnings
plottery params                                   # the user's param values
plottery render /tmp/{{project-name}}.png         # render to look at, prints stats
plottery render /tmp/a.png /tmp/a.svg --set size=12 --set glow.radius=2   # try other values
plottery run                                      # like the editor's Run button
```

`plottery --help` lists all commands. If `plottery` is missing or lacks a command, install the version
matching `plottery_project`: `cargo install plottery_cli`, or `cargo install --path <checkout>/cli`.

The CLI works like the editor. It builds in release mode into the target directory the editor shares,
and runs with the user's param values, so a render shows what the user sees.

- `plottery params` prints one line per param: `name: type = value   (range or variants, default)   doc`.
  Each name and value is valid for `--set NAME=VALUE`; nested fields are `glow.radius`, list items
  `points.0`. Angles are `45deg`, vectors `1.5,2`, ranges `2..5`, colors `#rrggbb`, paper `A3 landscape`,
  options `on`/`off`. File paths are relative to the project root.
- `--set` and `--defaults` (the code's `#[value]`s) apply to that one command and never change the
  user's saved values. Use `params --set ...` to check an override before rendering.
- `render` writes `.png` (white, `--size` px on the long side, default 1600) and `.svg` files and prints
  shape count, draw and travel length, and the bounds against the paper frame.
- `plottery run` and `render --update-editor` write the result into `resources/` for the editor.
  Don't use them to try things out: they replace what the user sees.
- Compiler errors show paths relative to `{{project-name}}/`, e.g. `src/generate.rs` is
  `{{project-name}}/src/generate.rs`.

Cargo in `{{project-name}}/` behaves differently:

- It builds into a separate `{{project-name}}/target/` and compiles every dependency again.
- `cargo run -- svg <path>` renders the `#[value]` defaults and ignores the user's param values.

Use cargo only for what the CLI doesn't do, such as `cargo add` or `cargo clippy`.

**Done** means `plottery build` shows no warnings, you looked at a rendered png, and the bounds in
the stats are where you meant the drawing to be.

## Plottery source

Read the source instead of guessing APIs. `{{project-name}}/Cargo.toml` shows where it is:

- `path = ".../lib"`: a local Plottery checkout with `lib/`, `project/` and `project/project_macros/`.
- a version: `~/.cargo/registry/src/*/plottery_lib-<version>/`, same for `plottery_project` and `plottery_project_macros`.
- online: https://docs.rs/plottery_lib, https://docs.rs/plottery_project, https://github.com/TobiasGrothmann/plottery

## What Plottery already has

Check here before writing your own helper. Paths are in `plottery_lib/src/`; the docs there explain usage.

| Need | Where |
|------|-------|
| `Circle`, `Rect`, `Path`, arcs, rounded corners, the `Shape` enum | `shapes/` |
| hatched fills: spiral, criss-cross, towards the center | `filled_*` on `Rect` and `Circle` |
| smooth curves: Bézier, Hobby splines through points, a spline builder | `shapes/spline/` |
| intersection and containment tests | `intersects_*` and `contains_*` in `shapes/`, `geometry/line.rs` |
| per shape: clip by a mask, resample, dash, distort, reduce points | `traits/plottable.rs`, implemented by every shape |
| per layer: map, filter, distort, dash, combine shapes, reduce points, optimize, flatten, color, pen width | `composition/layer.rs` |
| loading an svg as a `Layer` | `Layer::new_from_svg` in `composition/layer.rs` |
| translate, rotate, scale, mirror, fit into a rect, bounding box, closest point | `traits/` |
| `V2`, `V2i`, `Line`, `TransformMatrix`; `.sum()` and `.average()` over points | `geometry/` |
| `Paper` (DIN A/B, custom, landscape, margin), `Frame` | `composition/paper.rs`, `composition/frame.rs` |
| grids, hex grids, colors and color names | `composition/` |
| `Angle` with constants like `golden_angle`, `Range`, easing | `maths/` |
| float helpers: `lerp`, `linlin`, `sin01`, `cos01`, `clamp01`; evenly spaced steps `fractions`, `linspace` | `maths/float.rs` |
| perlin, simplex and worley noise, scaled per cm or per paper | `maths/noise.rs` |
| random helpers and seeding | `maths/random.rs`, `maths/thread_local.rs` |
| Voronoi diagrams with relaxation | `generate/voronoi.rs` |
| contour lines of a 2D function (marching squares) | `generate/func_2d/` |
| images as brightness or darkness fields, histograms, equalizing | `sampling/` |
| text layout, bundled and system fonts | `text/`; the bundled list is `BundledFont` in `plottery_fonts` |

## Parameters

Expose what an artist wants to tweak while watching the result. Keep derived values and implementation
constants in code. Where to find out what is possible, all in `plottery_project`:

- **Field types:** `src/project_params/param_field.rs`. Every `impl ParamField for T` is a usable field
  type. Types with `impl RangedParamField` also accept `#[range(min, max)]`.
- **Attributes:** the docs on `PlotteryParams` and `PlotteryEnum` in `plottery_project_macros/src/lib.rs`.
- **Using a value:** each type's own docs and methods. Project-only types like `Seed`, the curves and
  the image and svg paths live in `src/project_params/`, the rest in `plottery_lib`.
- **Example:** `cargo_project_template/src/generate.rs`, the code every new project starts from.

Choosing well:

- Use the most specific type: `Angle` instead of an `f32` in radians, `Range` for a random min/max,
  `V2` for a position, `Seed` for randomness, `BundledFont` for a font, a `PlotteryEnum` for modes.
- Put a feature that can be switched off into `Option<SomeParamsStruct>`. It shows as a checkbox with its own section.
- Bound sliders with `#[range]` to values that give sensible results.
- Describe every param with `///` and state its unit. Group params with `#[section("..")]`, not `// ---` comments.

The user's values are kept across code changes while a field's name, type and `#[range]` stay the same.
Changing `#[value]` doesn't touch a kept value, so the user won't see a new default. Renaming a field or
changing its type or range resets it to the default. Moving it to another section keeps it.

## Conventions

- Units are centimetres. The origin is bottom-left and y points up; the svg export flips it.
- `Paper` defaults to A4 portrait with a 10% margin. `params.paper.frame()` has two rects:
  `outer_rect()` is the paper edge, where an optional cut line goes (a `draw_frame: bool` param);
  `inner_rect()` is the drawing area, and the margin between them is the only gap. Lay the drawing
  out for the inner rect, then `fit_inside_mut(frame.inner_rect(), Alignment::Center)` the finished
  layer to catch overhangs. Don't add padding or extra border params.
- Pen plotters only draw lines. Fills are hatching, and overlapping strokes are drawn twice.
- Plot time matters. Every pen lift costs time, many tiny segments are slow, and the pen slows at
  corners. Join paths with `combine_shapes_recursive`, thin them with `reduce_points_recursive`, and
  return the root layer through `optimize_recursive()` so paths are reordered for plotting.
- With a `Seed` param, call `params.seed.apply()` first. Then use Plottery's random helpers
  (`maths/random.rs`) and `Range::sample` instead of the `rand` crate, so results are reproducible.
  `NoiseSettings::noise()` also draws its seed from there, so create noise after applying the seed.
- For text, prefer single-line fonts (`Font::is_single_line`, `is_stroke`). Outline fonts draw both
  contours of every stroke.
- Use idiomatic Rust: iterator chains, builders. `shape.rotate(a)` returns a copy, `shape.rotate_mut(a)` edits in place.
