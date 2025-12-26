#!/bin/bash
# Build the imgplay Swift CLI sidecar for macOS
# This script builds the Swift binary for the current architecture

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SWIFT_PROJECT="$SCRIPT_DIR/../swift/imgplay"
BIN_DIR="$SCRIPT_DIR/../binaries"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building imgplay Swift sidecar...${NC}"

# Check we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo -e "${RED}Error: This script only runs on macOS${NC}"
    exit 1
fi

# Check Swift is available
if ! command -v swift &> /dev/null; then
    echo -e "${RED}Error: Swift is not installed${NC}"
    exit 1
fi

# Create binaries directory if it doesn't exist
mkdir -p "$BIN_DIR"

# Get current architecture
ARCH=$(uname -m)

if [ "$ARCH" = "arm64" ]; then
    TARGET_TRIPLE="aarch64-apple-darwin"
elif [ "$ARCH" = "x86_64" ]; then
    TARGET_TRIPLE="x86_64-apple-darwin"
else
    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
    exit 1
fi

echo "Target architecture: $TARGET_TRIPLE"

cd "$SWIFT_PROJECT"

# Check Swift version
SWIFT_VERSION=$(swift --version | head -n1)
echo "Using: $SWIFT_VERSION"

# Build release binary
echo "Building release binary..."
swift build -c release 2>&1

if [ $? -ne 0 ]; then
    echo -e "${RED}Swift build failed${NC}"
    exit 1
fi

# Copy to binaries with target triple suffix (Tauri convention)
SOURCE_BIN=".build/release/imgplay"
DEST_BIN="$BIN_DIR/imgplay-$TARGET_TRIPLE"

if [ -f "$SOURCE_BIN" ]; then
    cp "$SOURCE_BIN" "$DEST_BIN"
    chmod +x "$DEST_BIN"
    echo -e "${GREEN}Built: $DEST_BIN${NC}"

    # Show binary info
    echo ""
    echo "Binary info:"
    file "$DEST_BIN"
    ls -lh "$DEST_BIN"
else
    echo -e "${RED}Error: Built binary not found at $SOURCE_BIN${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Build complete!${NC}"
