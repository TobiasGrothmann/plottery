#!/usr/bin/env bash
# Builds every project in examples/ (clippy), renders it with the checkout's CLI and compares the
# render with the committed examples/<name>/preview.png. Fails if any output changed.
# With --images, overwrites the previews instead, to commit after an intended change.

set -euo pipefail

cd "$(dirname "$0")/.."

update=false
if [[ "${1:-}" == "--images" ]]; then
    update=true
fi

changed_dir=target/examples/changed
rm -rf "$changed_dir"
mkdir -p "$changed_dir"
changed=()

for project in examples/*/; do
    project=${project%/}
    name=$(basename "$project")
    echo "== $name"

    cargo clippy -q --manifest-path "$project/$name/Cargo.toml" --target-dir target/examples -- -D warnings

    rendered="$changed_dir/$name.png"
    cargo run -q -p plottery_cli -- render -p "$project" "$rendered" --defaults --size 1200

    if $update; then
        mv "$rendered" "$project/preview.png"
    elif cmp -s "$rendered" "$project/preview.png"; then
        rm "$rendered"
    else
        changed+=("$name")
    fi
done

if ((${#changed[@]})); then
    echo ""
    echo "Output changed: ${changed[*]}"
    echo "New renders are in $changed_dir/. If the change is intended, run"
    echo "scripts/check_examples.sh --images and commit the previews, otherwise fix the code."
    exit 1
fi
