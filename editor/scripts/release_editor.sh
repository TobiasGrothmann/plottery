#!/bin/bash
set -e

VERSION=$1

echo "Building .dmg for version $VERSION using the Dioxus CLI (dx)"

# Clean up any existing DMG files first
DMG_PATH="../target/dx/PlotteryEditor/bundle/macos/bundle/dmg"
if [ -d "$DMG_PATH" ]; then
  echo "Cleaning up existing DMG files from $DMG_PATH"
  find "$DMG_PATH" -name "*.dmg" -type f -delete
  echo "Cleanup complete."
fi

# Build the .dmg
dx bundle --release --platform desktop --package-types dmg

# Check if DMG file exists before creating a release
echo "Checking for DMG files in $DMG_PATH"
ls -la "$DMG_PATH"

# Use find to locate the DMG file
DMG_FILE=$(find "$DMG_PATH" -name "*.dmg" -type f | head -n 1)

if [ -z "$DMG_FILE" ]; then
  echo "ERROR: No DMG file found! Release will not be created."
  exit 1
fi

echo "Found DMG file: $DMG_FILE"

# Rename the DMG file to include version number
RENAMED_DMG_NAME="PlotteryEditor_v$VERSION.dmg"
RENAMED_DMG="$DMG_PATH/$RENAMED_DMG_NAME"
echo "Renaming DMG file to $RENAMED_DMG"
cp "$DMG_FILE" "$RENAMED_DMG"

# Create GitHub release
echo "Creating GitHub release v$VERSION"
NOTES="This dmg is not codesigned.

To remove macOS's *\"This app is damaged\"* hogwash use:
\`\`\`
xattr -d com.apple.quarantine $RENAMED_DMG_NAME
\`\`\`"

gh release create "v$VERSION" \
  --title "Plottery Editor v$VERSION" \
  --notes "$NOTES" \
  --generate-notes

# Upload to GitHub release
echo "Uploading $RENAMED_DMG to GitHub release v$VERSION"
gh release upload "v$VERSION" "$RENAMED_DMG" --clobber