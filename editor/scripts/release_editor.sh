#!/usr/bin/env bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

VERSION=$1
UPLOAD=$2

if [ -z "$VERSION" ]; then
  echo -e "${RED}✗ Error: Version number required${NC}"
  echo "Usage: $0 <version> <upload:true|false>"
  exit 1
fi

if [ -z "$UPLOAD" ]; then
  echo -e "${RED}✗ Error: Upload flag required${NC}"
  echo "Usage: $0 <version> <upload:true|false>"
  exit 1
fi

if [ "$UPLOAD" != "true" ] && [ "$UPLOAD" != "false" ]; then
  echo -e "${RED}✗ Error: Upload flag must be 'true' or 'false'${NC}"
  echo "Usage: $0 <version> <upload:true|false>"
  exit 1
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
  echo -e "${RED}✗ Error: Invalid version format: $VERSION${NC}"
  echo "Expected semver format (e.g., 0.7.0 or 1.2.3-beta.1)"
  exit 1
fi

echo "========================================="
echo -e "${BLUE}📦 Building Plottery Editor DMG${NC}"
echo -e "${BLUE}Version: $VERSION${NC}"
echo "========================================="
echo ""

DMG_PATH="../target/dx/PlotteryEditor/bundle/macos/bundle/dmg"
if [ -d "$DMG_PATH" ]; then
  echo -e "${YELLOW}Cleaning up existing DMG files...${NC}"
  find "$DMG_PATH" -name "*.dmg" -type f -delete
  echo -e "${GREEN}✓ Cleanup complete${NC}"
  echo ""
fi

echo -e "${YELLOW}Building .dmg using Dioxus CLI (dx)...${NC}"
dx bundle --release --platform desktop --package-types dmg
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

echo -e "${YELLOW}Locating DMG file...${NC}"
DMG_FILE=$(find "$DMG_PATH" -name "*.dmg" -type f | head -n 1)

if [ -z "$DMG_FILE" ]; then
  echo -e "${RED}✗ Error: No DMG file found in $DMG_PATH${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Found DMG file: $DMG_FILE${NC}"
echo ""

RENAMED_DMG_NAME="PlotteryEditor_v$VERSION.dmg"
RENAMED_DMG="$DMG_PATH/$RENAMED_DMG_NAME"
echo -e "${YELLOW}Renaming DMG file...${NC}"
cp "$DMG_FILE" "$RENAMED_DMG"
echo -e "${GREEN}✓ Renamed to: $RENAMED_DMG_NAME${NC}"
echo ""

if [ "$UPLOAD" = "true" ]; then
  echo -e "${YELLOW}Creating GitHub release v$VERSION...${NC}"
  NOTES="This dmg is not codesigned.

To remove macOS's *\"This app is damaged\"* hogwash use:
\`\`\`
xattr -d com.apple.quarantine $RENAMED_DMG_NAME
\`\`\`"

  gh release create "v$VERSION" \
    --title "Plottery Editor v$VERSION" \
    --notes "$NOTES" \
    --generate-notes

  echo -e "${GREEN}✓ GitHub release created${NC}"
  echo ""

  echo -e "${YELLOW}Uploading $RENAMED_DMG_NAME to GitHub release...${NC}"
  gh release upload "v$VERSION" "$RENAMED_DMG" --clobber
  echo -e "${GREEN}✓ Upload complete${NC}"
  echo ""

  echo "========================================="
  echo -e "${GREEN}✓ Release v$VERSION published successfully!${NC}"
  echo "========================================="
  echo ""
  echo -e "${BLUE}View release:${NC} https://github.com/TobiasGrothmann/plottery/releases/tag/v$VERSION"
else
  echo "========================================="
  echo -e "${GREEN}✓ DMG built successfully!${NC}"
  echo "========================================="
  echo ""
  echo -e "${BLUE}DMG location:${NC} $RENAMED_DMG"
  echo ""
  echo -e "${YELLOW}To publish to GitHub, run:${NC}"
  echo -e "${YELLOW}  ./scripts/release_editor.sh $VERSION true${NC}"
fi
