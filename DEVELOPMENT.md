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

## Tests

```sh
cargo test --workspace
```

The `lib` and `project` READMEs are their crates' docs.rs front pages, so their code blocks run as doctests too. Links in them must be absolute GitHub URLs, docs.rs does not resolve relative ones.

## Editor screenshots

```sh
./scripts/editor_screenshots.sh [PROJECT]
```

Writes `editor/screenshots/browser.png` and `editor/screenshots/editor.png` from the installed `PlotteryEditor.app` at a fixed window size. Without `PROJECT` it shows a fresh template project. Your editor state is restored afterwards. The terminal needs the macOS Screen Recording permission.

## Pre-Release Checks

Before releasing, validate (tests, clippy, docs) with:

```sh
./scripts/pre_release_checks.sh
```

## Release

```sh
cargo release --execute                                 # Publish crates to crates.io and build editor DMG
./scripts/release_editor.sh <version> true             # Upload DMG to GitHub
```

The `cargo release` command automatically builds the DMG via pre-release-hook but does not upload it. You must manually run the upload script afterwards with the same version number.
