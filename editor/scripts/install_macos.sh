#!/usr/bin/env bash
# Installs the newest Plottery Editor release into /Applications (or the directory given as argument).
set -euo pipefail

target="${1:-/Applications}"
url=$(curl -fsSL https://api.github.com/repos/TobiasGrothmann/plottery/releases/latest | grep -o 'https://[^"]*\.dmg' | head -n 1)
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
