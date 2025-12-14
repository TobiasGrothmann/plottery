# AGENTS.md - Plottery

Generative art engine for pen-plotters written in Rust.

## Workspace Structure

| Crate | Purpose |
|-------|---------|
| `lib/` | Core geometry library - shapes, vectors, layers, transformations |
| `project/` | Project creation/management, `PlotteryParams` derive macro |
| `cli/` | Command-line tool for creating and running projects |
| `editor/` | Desktop GUI (Dioxus) for previewing and managing projects |
| `server/` | Pen-plotter hardware controller (Rocket HTTP, Raspberry Pi GPIO) |
| `server_lib/` | Shared types for server communication |

## Plottery vs Plottery Projects

**Plottery** is this workspace - the engine, editor, and tools.

A **Plottery Project** is a user-created artwork generator: a standalone Cargo package that depends on `plottery`. Projects define a `Params` struct and a `generate(params) -> Layer` function. The editor/CLI compile and run these projects to produce artwork.

See `project/cargo_project_template/` for the generated project template.

## Where to Find Things

- **Shapes & geometry:** `lib/src/shapes/`, `lib/src/geometry/`
- **Layer composition:** `lib/src/composition/layer.rs`
- **Transformation traits:** `lib/src/traits/` (`Translate`, `Rotate`, `Scale`, `Mirror`, `Transform`, etc.)
- **Math utilities:** `lib/src/maths/` (angles, noise, random)
- **Project management:** `project/src/project.rs`
- **PlotteryParams macro:** `project/project_macros/`
- **Editor UI components:** `editor/src/components/`
- **Hardware control:** `server/src/hardware.rs`

## Code Style

Use idiomatic rust - iterators, builder types, and functional programming.

**Transformations** have immutable and mutable variants:
- `shape.rotate(angle)` returns new shape
- `shape.rotate_mut(angle)` modifies in place
- `TransformMatrix::builder()` for chained transforms

**Layers** are hierarchical shape containers:
- `push()` adds shapes, `push_layer()` adds sublayers
- `optimize_recursive()` reorders paths for efficient plotting
- Export via `write_svg()` or binary `.plotl` files

**Error handling:** Use `anyhow::Result` throughout.

## File Conventions

- Tests: `*_test.rs` adjacent to source files
- Serialization formats: `.plotl` (layers), `.plotp` (params), `.plottery` (project config)

## Tools

- Cargo workspace: `cargo build`, `cargo test`
- Dioxus CLI (`dx`) for editor builds
- Cross-compilation for Raspberry Pi server

## Testing

Add `*_test.rs` files for new functionality. Only if relevant, use `cargo` for testing.

## Publishing

`lib`, `project`, and `cli` are published to crates.io. Workspace shares a single version in root `Cargo.toml`.

## Common Pitfalls

- **Adding shapes:** Must implement all traits in `lib/src/traits/` and add variant to `Shape` enum in `lib/src/shapes/shape.rs`
- **Server crate:** Hardware-specific code; `raspi` feature flag required for GPIO
