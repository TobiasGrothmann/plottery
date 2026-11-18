#!/usr/bin/env bash
# Builds every project in examples/ (clippy), renders it with the checkout's CLI and compares the
# svg with the committed examples/<name>/reference.svg. Fails if any drawing changed.
# With --update, overwrites reference.svg and the README image preview.png instead, to commit
# after an intended change.

set -euo pipefail

cd "$(dirname "$0")/.."

update=false
if [[ "${1:-}" == "--update" ]]; then
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

    rendered="$changed_dir/$name"
    cargo run -q -p plottery_cli -- render -p "$project" "$rendered.svg" "$rendered.png" --defaults --size 1200

    if $update; then
        mv "$rendered.svg" "$project/reference.svg"
        mv "$rendered.png" "$project/preview.png"
    elif cmp -s "$rendered.svg" "$project/reference.svg"; then
        rm "$rendered.svg" "$rendered.png"
    else
        changed+=("$name")
    fi
done

if ((${#changed[@]})); then
    echo ""
    echo "Drawing changed: ${changed[*]}"
    echo "New renders are in $changed_dir/. If the change is intended, run"
    echo "scripts/check_examples.sh --update and commit the files, otherwise fix the code."
    exit 1
fi
