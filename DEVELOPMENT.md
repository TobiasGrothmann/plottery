# Development

## Pre-Release Checks

Before releasing, validate (tests, clippy, docs) with:

```sh
./scripts/pre_release.sh
```

## Release

```sh
cargo release --execute                                 # Publish crates to crates.io and build editor DMG
cd editor && ./scripts/release_editor.sh <version> true # Upload DMG to GitHub
```

The `cargo release` command automatically builds the DMG via pre-release-hook but does not upload it. You must manually run the upload script afterwards with the same version number.
