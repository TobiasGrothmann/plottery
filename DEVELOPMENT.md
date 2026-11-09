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
plottery set-home /path/to/plottery          # once, sets PLOTTERY_HOME
plottery new sketches my_sketch -l home      # depends on $PLOTTERY_HOME/lib and project
```

`-l <PATH>` uses any other checkout.

## Examples

`examples/` holds the small projects shown in the README. They depend on this checkout by path, so they always use the current API.

```sh
cargo run -p plottery_cli -- run examples/spiral    # run one, or open its folder in the editor
./scripts/check_examples.sh                         # clippy, render, compare with the committed previews
./scripts/check_examples.sh --images                # rewrite examples/<name>/preview.png instead
```

The check fails when a render differs from the committed `preview.png` and keeps the new renders in `target/examples/changed/`. Look at them: if the change is intended, run `--images` and commit the previews, otherwise fix the code. The README shows `spiral` from a hosted image, its `preview.png` is only the reference.

Create a new example with `plottery new examples <name> -l ../../..`, then delete its `.git`, `.gitignore` and `AGENTS.md`, and write its first preview with `--images`.

## Tests

```sh
cargo test --workspace
```

The `lib` and `project` READMEs are their crates' docs.rs front pages, so their code blocks run as doctests too. Links in them must be absolute GitHub URLs, docs.rs does not resolve relative ones.

## Pre-Release Checks

Before releasing, validate (tests, clippy, examples, docs) with:

```sh
./scripts/pre_release_checks.sh
```

## Release

```sh
cargo release --execute                                 # Publish crates to crates.io and build editor DMG
./scripts/release_editor.sh <version> true             # Upload DMG to GitHub
```

The `cargo release` command automatically builds the DMG via pre-release-hook but does not upload it. You must manually run the upload script afterwards with the same version number.
