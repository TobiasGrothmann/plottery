#!/usr/bin/env bash
set -euo pipefail

SERVICE_NAME="plottery.service"
CARGO_BIN="/home/pi/.cargo/bin/cargo"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"

if [ ! -x "$CARGO_BIN" ]; then
  echo "Error: cargo not found at $CARGO_BIN" >&2
  echo "Install rust for user pi or adjust CARGO_BIN in this script." >&2
  exit 1
fi

if [ ! -f "/etc/systemd/system/$SERVICE_NAME" ]; then
  echo "Error: /etc/systemd/system/$SERVICE_NAME not found." >&2
  echo "Install the service first, for example:" >&2
  echo "  sudo cp $REPO_ROOT/server/plottery.service /etc/systemd/system/$SERVICE_NAME" >&2
  echo "  sudo systemctl daemon-reload" >&2
  echo "  sudo systemctl enable $SERVICE_NAME" >&2
  exit 1
fi

if ! systemctl list-unit-files --type=service --no-legend | grep -q "^$SERVICE_NAME"; then
  echo "Error: $SERVICE_NAME is not registered with systemd." >&2
  echo "Run: sudo systemctl daemon-reload" >&2
  exit 1
fi

sudo systemctl stop "$SERVICE_NAME"

"$CARGO_BIN" build --manifest-path "$REPO_ROOT/Cargo.toml" -p plottery_server --features raspi --release

sudo "$REPO_ROOT/target/release/plottery_server"
