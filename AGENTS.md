# AGENTS.md - Plottery

Generative art engine for pen-plotters written in Rust.

## Workspace Structure

| Folder | Package | Purpose |
|--------|---------|---------|
| `lib/` | `plottery_lib` | Core geometry: shapes, vectors, layers, transformations, text |
| `fonts/` | `plottery_fonts` | Bundled plotter fonts with their licences, loaded via `Font::bundled` |
| `project/` | `plottery_project` | Project creation and management, param types, project runner |
| `project/project_macros/` | `plottery_project_macros` | `PlotteryParams` derive macro |
| `cli/` | `plottery_cli` | Command-line tool for creating and running projects, binary `plottery` |
| `editor/` | `plottery_editor` | Desktop GUI (Dioxus) for previewing, editing params and plotting |
| `server/` | `plottery_server` | Pen-plotter controller on a Raspberry Pi (Rocket HTTP, GPIO) |
| `server_lib/` | `plottery_server_lib` | Types shared by editor and server |

## Plottery vs Plottery Projects

**Plottery** is this workspace - the engine, editor, and tools.

A **Plottery Project** is a user-created artwork generator: a standalone Cargo package that depends on `plottery_lib` and `plottery_project`. Projects define a `Params` struct and a `generate(params) -> Layer` function. The editor/CLI compile and run these projects to produce artwork.

## Where to Find Things

- **Lib API overview:** `lib/README.md`
- **Shapes & geometry:** `lib/src/shapes/`, `lib/src/geometry/`
- **Splines (Bézier, Hobby):** `lib/src/shapes/spline/`
- **Layers, paper, frame, grids, colors:** `lib/src/composition/`
- **Transformation traits:** `lib/src/traits/`
- **Math:** `lib/src/maths/` (angles, noise, random, easing, ranges)
- **Generators (Voronoi, marching squares):** `lib/src/generate/`
- **Image sampling:** `lib/src/sampling/`
- **Effects (blur, hatch):** `lib/src/effects/`
- **Text & fonts:** `lib/src/text/`, font data in `fonts/fonts/`, credits in `fonts/LICENSES/CREDITS.md`
- **Project management:** `project/src/project.rs`
- **Param types:** `project/src/project_params/`
- **Project template:** `project/cargo_project_template/`, its agent doc is `project/resources/AGENTS.md`
- **Example projects:** `examples/`, standalone projects with path dependencies on this checkout, shown in the README
- **Editor:** pages in `editor/src/router/`, param widgets in `editor/src/router/editor/params_editor/`, shared widgets in `editor/src/components/`
- **Hardware control:** `server/src/gpio_executor.rs`, `server/src/task_handler.rs`
- **New project skill:** `.claude/skills/new-plottery-project/`

## Code Style

- Idiomatic Rust: iterators, builder types, functional style.
- Build `Path`s and `Layer`s with one iterator chain and `collect()`, or preallocate with `with_capacity`. Avoid `new()` followed by `push()`/`extend()`.
- `///` docstrings are one short line, and only where the signature alone is unclear.
- Transformations come in pairs: `rotate` returns a new value, `rotate_mut` changes it in place.
- Error handling: `anyhow::Result`.

## File Conventions

- Tests: `foo_test.rs` next to `foo.rs`, wrapped in a `#[cfg(test)]` module and registered with `mod foo_test;` in the parent `mod.rs`.
- Serialization formats: `.plotl` (layers), `.plotp` (params), `.plottery` (project config)

## Testing

Write tests for real logic, algorithms and fixed bugs, not for one-line helpers or trivial delegations.

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
- **Adding param types:** Add a `ProjectParamValue` variant, a `ParamField` impl in `project/src/project_params/param_field.rs` and a widget in the editor's params editor. List the new type in `project/resources/AGENTS.md`.
- **Project template:** `project/resources/AGENTS.md` is copied into every new project and describes the project, not this workspace. Existing projects keep their old copy, so template changes have to be redistributed.
- **API changes:** Update `examples/` in the same change; `scripts/check_examples.sh` must pass. It fails when a render differs from the committed `preview.png`: look at `target/examples/changed/`, then fix the code or commit new previews with `--images`
- **Crate READMEs:** `lib/README.md` and `project/README.md` are their crates' docs.rs pages (`include_str!`), so their code blocks are doctests and their links must be absolute GitHub URLs.
- **Editor formatting:** `dx fmt` moves comments and splits `.await` chains. Format rsx by hand, or run it on one file and diff the result. `cargo fmt` is safe but skips rsx.
- **Server crate:** Hardware-specific code; `raspi` feature flag required for GPIO
