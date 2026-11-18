# AGENTS.md - Plottery

Generative art engine for pen-plotters written in Rust.

## Workspace Structure

| Folder | Package | Purpose |
|--------|---------|---------|
| `lib/` | `plottery_lib` | Core geometry: shapes, vectors, layers, transformations, text |
| `fonts/` | `plottery_fonts` | Bundled plotter fonts with their licences, loaded via `Font::bundled` |
| `runtime/` | `plottery_runtime` | What projects compile against: param types, `.plotp` format, `run_project` entry point |
| `runtime/runtime_macros/` | `plottery_runtime_macros` | `PlotteryParams` and `PlotteryEnum` derive macros |
| `project/` | `plottery_project` | Host side: project creation and management, template, building and running project binaries, cache, previews |
| `cli/` | `plottery_cli` | Command-line tool for creating and running projects, binary `plottery` |
| `editor/` | `plottery_editor` | Desktop GUI (Dioxus) for previewing, editing params and plotting |
| `server/` | `plottery_server` | Pen-plotter controller on a Raspberry Pi (Rocket HTTP, GPIO) |
| `server_lib/` | `plottery_server_lib` | Types shared by editor and server |

## Plottery vs Plottery Projects

**Plottery** is this workspace - the engine, editor, and tools.

A **Plottery Project** is a user-created artwork generator: a standalone Cargo package that depends on `plottery_lib` and `plottery_runtime`. Projects define a `Params` struct and a `generate(params) -> Layer` function. The editor/CLI compile and run these projects through `plottery_project`, which projects never depend on: keep `plottery_runtime`'s dependencies light, every project compiles them.

## Where to Find Things

- **Lib API overview:** `lib/README.md`
- **Shapes & geometry:** `lib/src/shapes/`, `lib/src/geometry/`
- **Splines (Bézier, Hobby):** `lib/src/shapes/spline/`
- **Layers, paper, frame, grids, colors:** `lib/src/composition/`
- **Transformation traits:** `lib/src/traits/`
- **Math:** `lib/src/maths/` (angles, noise, random, easing, ranges)
- **Generators (Voronoi, Poisson disk, marching squares):** `lib/src/generate/`
- **Image sampling:** `lib/src/sampling/`
- **Effects (blur, hatch):** `lib/src/effects/`
- **Text & fonts:** `lib/src/text/`, font data in `fonts/fonts/`, credits in `fonts/LICENSES/CREDITS.md`
- **Project management:** `project/src/project.rs`
- **Param types:** `runtime/src/project_params/`, the `.plotp` format in `params_file_format.rs`
- **Project binary:** entry point and subcommands in `runtime/src/project_main.rs`, `PROTOCOL_VERSION` in `runtime/src/protocol.rs`, run stream in `runtime/src/run_stream.rs` (project side) and `project/src/run_stream.rs` (host side)
- **Project template:** `project/cargo_project_template/`, its agent doc is `project/resources/AGENTS.md`
- **Example projects:** `examples/`, standalone projects with path dependencies on this checkout, shown in the README
- **Editor:** pages in `editor/src/router/`, param widgets in `editor/src/router/editor/params_editor/`, shared widgets in `editor/src/components/`
- **Hardware control:** `server/src/gpio_executor.rs`, `server/src/task_handler.rs`
- **New project skill:** `.claude/skills/new-plottery-project/`

## Code Style

- Idiomatic Rust: iterators, builder types, functional style.
- Prefer one iterator chain with `collect()` to build `Path`s and `Layer`s where that reads naturally. `new()` with `push()`/`extend()` is fine when it is clearer.
- `///` docstrings are one short line, and only where the signature alone is unclear.
- Transformations come in pairs: `rotate` returns a new value, `rotate_mut` changes it in place.
- Error handling: `anyhow::Result`.

## File Conventions

- Tests: `foo_test.rs` next to `foo.rs`, wrapped in a `#[cfg(test)]` module and registered with `mod foo_test;` in the parent `mod.rs`.
- Serialization formats: `.plotl` (layers), `.plotp` (params), `.plots` (plot settings), `.plottery` (project config). The three binary ones start with a header of 4 magic bytes and a `u16` format version (`lib/src/composition/file_header.rs`); bump the version when the layout after it changes.

## Testing

Write tests for real logic, algorithms and fixed bugs, not for one-line helpers or trivial delegations.

Tests are deterministic. Random test data comes from a seeded `StdRng::seed_from_u64(n)`. Tests that use the lib's global rng (`rand_*`, `V2::random_*`, unseeded noise, `seed()`) start with `let _rng = seed_for_test(n);` from `lib/src/maths/global_rng.rs`. It holds a lock so parallel tests don't interleave.

Before finishing a change:

```sh
cargo clippy --workspace -- -D warnings
cargo test -p <package>
./scripts/check_examples.sh     # after lib or project API changes
```

`scripts/pre_release_checks.sh` runs the full set, including rustdoc with warnings as errors.

## Tools

Building, local checkouts in projects, tests and releasing: see `DEVELOPMENT.md`. Scripts live in `scripts/`.

## Common Pitfalls

- **Adding shapes:** A plottable shape implements `Plottable`, which needs a variant in the `Shape` enum in `lib/src/shapes/shape.rs`. Implement only the transform traits that make sense, e.g. `Rect` has no `Rotate` and `Circle` no `Scale2D`. Shapes that are not plotted directly, like `Spline`, convert to a `Path` instead.
- **Adding param types:** every place a new `ProjectParamValue` variant is handled:
  - `runtime/src/project_params/project_param_value.rs`: the variant, appended at the end (variant order is the `.plotp` type tag, see `ParamRecord` in `params_file_format.rs`), `type_name`, `value_as_string`
  - `runtime/src/project_params/param_field.rs`: `impl ParamField`, `impl RangedParamField` if it takes `#[range]`
  - `runtime/src/project_params/param_text.rs`: `to_text`, `set_from_text`
  - `runtime/src/project_params/project_param.rs`: `value_schema_equal`
  - `runtime/src/project_params/project_params_list_wrapper.rs`: `combine_values`
  - `runtime/src/project_params/mod.rs`: the module and re-export of a new value type
  - `runtime/src/protocol.rs`: `PROTOCOL_VERSION`
  - `cli/src/params.rs`: `type_label`, `constraint`, `same_value`
  - `editor/src/router/editor/params_editor/params_editor.rs`: `render_leaf_param`, with the widget as a new `*_field.rs` registered in `mod.rs`
  - `runtime/README.md`: the widget table
  - `project/resources/AGENTS.md`: "Parameters"
  - tests: `param_field_test.rs`, `param_text_test.rs`, `project_params_list_wrapper_test.rs`
- **Project files:** Everything in a project folder is committed. Derived files (builds, previews, thumbnails) go to the cache dir, see `plottery_cache_dir` in `project/src/project.rs`.
- **Project template:** `project/resources/AGENTS.md` is copied into every new project and describes the project, not this workspace. Existing projects keep their old copy, so template changes have to be redistributed.
- **API changes:** Update `examples/` in the same change; `scripts/check_examples.sh` must pass. It fails when a render differs from the committed `preview.png`: look at `target/examples/changed/`, then fix the code or commit new previews with `--images`
- **Crate READMEs:** `lib/README.md`, `runtime/README.md` and `project/README.md` are their crates' docs.rs pages (`include_str!`), so their code blocks are doctests and their links must be absolute GitHub URLs.
- **Editor formatting:** `dx fmt` moves comments and splits `.await` chains. Format rsx by hand, or run it on one file and diff the result. `cargo fmt` is safe but skips rsx.
- **Server crate:** Hardware-specific code; `raspi` feature flag required for GPIO
- **Platforms:** Supported systems per crate are in the README's "Platforms" section; don't add code for others.
