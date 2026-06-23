#!/usr/bin/env bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "========================================="
echo -e "${BLUE}🔍 Plottery Pre-Release Check${NC}"
echo "========================================="
echo ""

echo -e "${YELLOW}Running tests...${NC}"
cargo test --workspace
echo -e "${GREEN}✓ Tests passed${NC}"
echo ""

echo -e "${YELLOW}Running clippy...${NC}"
cargo clippy --workspace -- -D warnings
echo -e "${GREEN}✓ Clippy passed${NC}"
echo ""

echo -e "${YELLOW}Checking documentation...${NC}"
RUSTDOCFLAGS='-D warnings -D rustdoc::broken_intra_doc_links' cargo doc --workspace --no-deps
echo -e "${GREEN}✓ Documentation valid${NC}"
echo ""

echo "========================================="
echo -e "${GREEN}✓ All checks passed!${NC}"
echo "========================================="
echo ""
echo "Ready to release 🚀"
