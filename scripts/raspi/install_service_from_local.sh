#!/usr/bin/env bash
set -euo pipefail

SERVICE_NAME="plottery.service"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
SERVICE_TEMPLATE="$REPO_ROOT/server/plottery.service"
SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME"

if [ "$(id -u)" -eq 0 ]; then
  echo "Error: Do not run this script as root." >&2
  echo "Run it as the target user and let this script call sudo as needed." >&2
  exit 1
fi

CURRENT_USER="$(id -un)"

if [ ! -f "$SERVICE_TEMPLATE" ]; then
  echo "Error: service template not found at $SERVICE_TEMPLATE" >&2
  exit 1
fi

if ! grep -q "{{USER_NAME}}" "$SERVICE_TEMPLATE"; then
  echo "Error: template placeholder {{USER_NAME}} not found in $SERVICE_TEMPLATE" >&2
  exit 1
fi

RENDERED_FILE="$(mktemp)"
trap 'rm -f "$RENDERED_FILE"' EXIT

sed "s|{{USER_NAME}}|$CURRENT_USER|g" "$SERVICE_TEMPLATE" > "$RENDERED_FILE"

sudo install -m 644 "$RENDERED_FILE" "$SERVICE_FILE"
sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"
sudo systemctl restart "$SERVICE_NAME"
sudo systemctl --no-pager --full status "$SERVICE_NAME"
