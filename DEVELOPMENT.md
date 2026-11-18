# Development

## Build from source

```sh
cargo build                  # whole workspace
cargo install --path cli     # the plottery CLI from this checkout
```

The editor needs the [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started) (`dx`):

```sh
cd editor
dx serve                     # dev build with hot reload of the editor itself
```

The server is built on the Raspberry Pi, see [server/README.md](server/README.md).

## Projects against a local checkout

New projects use the published crates by default. To make a project depend on this checkout instead:

```sh
plottery set-home /path/to/plottery          # once, saved in Plottery's config file
plottery new sketches my_sketch -l home      # depends on that checkout's lib and runtime
```

`-l <PATH>` uses any other checkout. The config file is read by the CLI and the editor; where it is
and how the `PLOTTERY_HOME` environment variable overrides it: `project/src/plottery_config.rs`.

## Examples

`examples/` holds the small projects shown in the README. They depend on this checkout by path, so they always use the current API.

```sh
cargo run -p plottery_cli -- run examples/spiral    # run one, or open its folder in the editor
./scripts/check_examples.sh                         # clippy, render, compare with the committed drawings
./scripts/check_examples.sh --update                # rewrite reference.svg and preview.png instead
```

The check fails when an example's svg differs from its committed `reference.svg` and keeps the new renders in `target/examples/changed/`. Look at them: if the change is intended, run `--update` and commit the files, otherwise fix the code. `preview.png` is only the image for the README (which shows `spiral` from a hosted image), so a renderer update that changes pixels fails nothing.

Create a new example with `plottery new examples <name> -l ../../..`, then delete its `.git`, `.gitignore` and `AGENTS.md`, and write its first reference with `--update`.

## Tests

```sh
cargo test --workspace
```

The `lib`, `runtime` and `project` READMEs are their crates' docs.rs front pages, so their code blocks run as doctests too. Links in them must be absolute GitHub URLs, docs.rs does not resolve relative ones.

## Pre-Release Checks

Before releasing, validate (tests, clippy, examples, docs) with:

```sh
./scripts/pre_release_checks.sh
```

## Release

```sh
cargo release <patch|minor> --execute                   # Publish crates to crates.io and build editor DMG
./scripts/release_editor.sh <version> true             # Upload DMG to GitHub
```

All crates share one version and are released together. `cargo release` publishes them in dependency order, so `plottery_runtime_macros` and `plottery_runtime` go out before `plottery_project`. Use `patch` when nothing breaks for existing projects, and `minor` for breaking changes: projects depend on `^0.x`, so a minor bump leaves them on the old version until they are updated.

The `cargo release` command automatically builds the DMG via pre-release-hook but does not upload it. You must manually run the upload script afterwards with the same version number.
