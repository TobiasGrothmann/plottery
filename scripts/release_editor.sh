#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
EDITOR_DIR="$REPO_ROOT/editor"

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

BUNDLE_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}/dx/PlotteryEditor/bundle/macos"
if [ -d "$BUNDLE_DIR" ]; then
  echo -e "${YELLOW}Cleaning up existing DMG files...${NC}"
  find "$BUNDLE_DIR" -name "*.dmg" -type f -delete
  echo -e "${GREEN}✓ Cleanup complete${NC}"
  echo ""
fi

echo -e "${YELLOW}Cleaning .DS_Store files from assets...${NC}"
find "$EDITOR_DIR/public" -name ".DS_Store" -type f -delete

echo -e "${YELLOW}Building .app using Dioxus CLI (dx)...${NC}"
(
  cd "$EDITOR_DIR"
  dx bundle --release --platform desktop --package-types macos
)
APP="$BUNDLE_DIR/macos/PlotteryEditor.app"
if [ ! -d "$APP" ]; then
  echo -e "${RED}✗ Error: No app bundle at $APP${NC}"
  exit 1
fi
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# dx 0.7.9 can't add document types to the Info.plist it generates, so merge them in afterwards
echo -e "${YELLOW}Registering .plottery files in Info.plist...${NC}"
/usr/libexec/PlistBuddy -c "Merge $EDITOR_DIR/document_types.plist" "$APP/Contents/Info.plist"
echo -e "${GREEN}✓ Info.plist updated${NC}"
echo ""

RENAMED_DMG_NAME="PlotteryEditor_v$VERSION.dmg"
RENAMED_DMG="$BUNDLE_DIR/$RENAMED_DMG_NAME"
echo -e "${YELLOW}Creating $RENAMED_DMG_NAME...${NC}"
STAGING=$(mktemp -d)
cp -R "$APP" "$STAGING/"
ln -s /Applications "$STAGING/Applications"
hdiutil create -volname PlotteryEditor -srcfolder "$STAGING" -ov -format UDZO "$RENAMED_DMG"
rm -rf "$STAGING"
echo -e "${GREEN}✓ DMG created${NC}"
echo ""

if [ "$UPLOAD" = "true" ]; then
  echo -e "${YELLOW}Creating GitHub release v$VERSION...${NC}"
  NOTES="Install or update on macOS with one command, it downloads the newest release, puts the app into /Applications and installs the matching plottery CLI if cargo is available:
\`\`\`
curl -fsSL https://raw.githubusercontent.com/TobiasGrothmann/plottery/main/scripts/install_macos.sh | bash
\`\`\`

The dmg is not codesigned. If you download it in a browser instead, remove macOS's *\"This app is damaged\"* hogwash with:
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
