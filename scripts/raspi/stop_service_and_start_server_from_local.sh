#!/usr/bin/env bash
set -euo pipefail

SERVICE_NAME="plottery.service"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

if [ "$(id -u)" -eq 0 ]; then
  echo "Error: Do not run this script as root." >&2
  echo "Run it as the user that owns the local Rust toolchain." >&2
  exit 1
fi

CURRENT_USER="$(id -un)"
CURRENT_HOME="${HOME:-}"

if [ -z "$CURRENT_HOME" ]; then
  CURRENT_HOME="$(getent passwd "$CURRENT_USER" | cut -d: -f6 || true)"
fi

if [ -z "$CURRENT_HOME" ]; then
  echo "Error: Could not determine home directory for user '$CURRENT_USER'." >&2
  exit 1
fi

CARGO_BIN="${CARGO_BIN:-$CURRENT_HOME/.cargo/bin/cargo}"

if [ ! -x "$CARGO_BIN" ]; then
  echo "Error: cargo not found at $CARGO_BIN" >&2
  echo "Install rust for user '$CURRENT_USER' or set CARGO_BIN=/path/to/cargo." >&2
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
