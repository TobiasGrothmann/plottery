#!/usr/bin/env bash
# Installs the newest Plottery Editor release into /Applications (or the directory given as argument),
# and the plottery CLI of the same version when cargo is available.
set -euo pipefail

target="${1:-/Applications}"
release=$(curl -fsSL https://api.github.com/repos/TobiasGrothmann/plottery/releases/latest)
url=$(echo "$release" | grep -o 'https://[^"]*\.dmg' | head -n 1)
tag=$(echo "$release" | grep -o '"tag_name": *"[^"]*"' | head -n 1 | sed 's/.*"\([^"]*\)"$/\1/')
version="${tag#v}"
dmg="$(mktemp -d)/$(basename "$url")"

echo "Downloading $(basename "$url")..."
curl -fL --progress-bar "$url" -o "$dmg"
volume=$(hdiutil attach "$dmg" -nobrowse -readonly | grep -o '/Volumes/.*')

rm -rf "$target/PlotteryEditor.app"
cp -R "$volume/PlotteryEditor.app" "$target/"
# the dmg is not codesigned; a browser-downloaded copy would be quarantined and reported as damaged
xattr -dr com.apple.quarantine "$target/PlotteryEditor.app" 2>/dev/null || true

hdiutil detach "$volume" -quiet
rm -rf "$(dirname "$dmg")"
echo "Installed $target/PlotteryEditor.app"

cargo=$(command -v cargo || echo "$HOME/.cargo/bin/cargo")
if [ -x "$cargo" ]; then
  echo "Installing the plottery CLI $version..."
  "$cargo" install plottery_cli --version "$version"
else
  echo "cargo not found, skipped the plottery CLI. Install Rust from https://rustup.rs, then run:"
  echo "  cargo install plottery_cli --version $version"
fi
