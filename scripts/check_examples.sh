#!/usr/bin/env bash
# Runs clippy on every project in examples/ and renders it with the checkout's CLI.
# With --images, writes examples/<name>/preview.png for the README instead of a temp file.

set -euo pipefail

cd "$(dirname "$0")/.."

images=false
if [[ "${1:-}" == "--images" ]]; then
    images=true
fi

# the README shows a hosted image for these
hosted_images=(spiral)

out_dir=$(mktemp -d)
trap 'rm -rf "$out_dir"' EXIT

for project in examples/*/; do
    project=${project%/}
    name=$(basename "$project")
    echo "== $name"

    cargo clippy -q --manifest-path "$project/$name/Cargo.toml" --target-dir target/examples -- -D warnings

    out="$out_dir/$name.png"
    if $images && [[ ! " ${hosted_images[*]} " =~ " $name " ]]; then
        out="$project/preview.png"
    fi
    cargo run -q -p plottery_cli -- render -p "$project" "$out" --defaults --size 1200
done
