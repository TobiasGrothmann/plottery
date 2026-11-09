#!/usr/bin/env bash
# Takes the README screenshots of the installed PlotteryEditor.app at a fixed window size.
# Usage: scripts/editor_screenshots.sh [PROJECT]   (WAIT=<seconds> to change the load delay)
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$REPO_ROOT/editor/screenshots"
STATE_DIR="$HOME/Library/Application Support/de.tobiasgrothmann.plottery"
APP_NAME="PlotteryEditor"

X=100
Y=100
WIDTH=1400
HEIGHT=900
TITLE_BAR=28
WAIT="${WAIT:-8}"

if [ "$(uname)" != "Darwin" ]; then
  echo "Error: macOS only." >&2
  exit 1
fi
command -v plottery >/dev/null || { echo "Error: plottery CLI not found (cargo install --path cli)." >&2; exit 1; }
if pgrep -xq "$APP_NAME"; then
  echo "Error: $APP_NAME is running. Quit it first, its state would be overwritten." >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
BACKUP_DIR="$TMP_DIR/state_backup"
mkdir -p "$BACKUP_DIR" "$OUT_DIR"

backup() {
  for f in app_state.json window_state.json; do
    if [ -f "$STATE_DIR/$f" ]; then
      cp -p "$STATE_DIR/$f" "$BACKUP_DIR/$f"
    fi
  done
}

restore() {
  quit_app
  for f in app_state.json window_state.json; do
    if [ -f "$BACKUP_DIR/$f" ]; then
      cp -p "$BACKUP_DIR/$f" "$STATE_DIR/$f"
    else
      rm -f "$STATE_DIR/$f"
    fi
  done
  rm -rf "$TMP_DIR"
  echo "Editor state restored."
}

quit_app() {
  if pgrep -xq "$APP_NAME"; then
    osascript -e "quit app \"$APP_NAME\"" >/dev/null 2>&1 || true
    for _ in $(seq 1 50); do
      pgrep -xq "$APP_NAME" || return 0
      sleep 0.2
    done
    pkill -x "$APP_NAME" || true
  fi
}

# Pins the window to the fixed frame and start route for the next launch.
write_window_state() {
  mkdir -p "$STATE_DIR"
  cat > "$STATE_DIR/window_state.json" <<EOF
{
  "x": $X.0,
  "y": $Y.0,
  "width": $WIDTH.0,
  "height": $HEIGHT.0,
  "route": "/"
}
EOF
}

capture() {
  local out="$1"
  sleep "$WAIT"
  osascript -e "tell application \"$APP_NAME\" to activate" >/dev/null 2>&1 || true
  sleep 1
  if ! screencapture -x -R "$X,$Y,$WIDTH,$((HEIGHT + TITLE_BAR))" "$out" || [ ! -s "$out" ]; then
    echo "Error: capture failed. Give your terminal the Screen Recording permission in" >&2
    echo "System Settings > Privacy & Security > Screen Recording, then run again." >&2
    exit 1
  fi
  echo "Wrote $out"
  quit_app
}

backup
trap restore EXIT

PROJECT="${1:-}"
if [ -z "$PROJECT" ]; then
  echo "Creating a template project..."
  plottery new "$TMP_DIR" demo
  PROJECT="$TMP_DIR/demo"
  plottery run "$PROJECT"
fi

echo "Browser screenshot..."
write_window_state
open -a "$APP_NAME"
capture "$OUT_DIR/browser.png"

echo "Editor screenshot..."
write_window_state
plottery open "$PROJECT"
capture "$OUT_DIR/editor.png"
