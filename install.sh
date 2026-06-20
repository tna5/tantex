#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== tantex installer ===${NC}\n"

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

# Map architecture names (normalize aarch64 to arm64 for display)
if [ "$ARCH" = "aarch64" ]; then
    ARCH="arm64"
fi

# Determine binary name based on OS and architecture
case "$OS" in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="tantex-linux-x86_64"
        else
            echo -e "${RED}✗ Unsupported architecture: $ARCH on Linux${NC}"
            echo "Supported: x86_64"
            exit 1
        fi
        ;;
    Darwin)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="tantex-macos-x86_64"
        elif [ "$ARCH" = "arm64" ]; then
            BINARY_NAME="tantex-macos-arm64"
        else
            echo -e "${RED}✗ Unsupported architecture: $ARCH on macOS${NC}"
            echo "Supported: x86_64, arm64"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}✗ Unsupported OS: $OS${NC}"
        echo "Supported: Linux, macOS"
        exit 1
        ;;
esac

echo "Detected: $OS $ARCH"
echo "Binary: $BINARY_NAME"
echo ""

# Download URL
DOWNLOAD_URL="https://github.com/tna5/tantex/releases/latest/download/$BINARY_NAME"

# Create temporary directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Downloading from GitHub releases..."
TEMP_BINARY="$TEMP_DIR/$BINARY_NAME"

# Try curl first, then wget
if command -v curl &> /dev/null; then
    if ! curl -fsSL -o "$TEMP_BINARY" "$DOWNLOAD_URL"; then
        echo -e "${RED}✗ Failed to download from $DOWNLOAD_URL${NC}"
        exit 1
    fi
elif command -v wget &> /dev/null; then
    if ! wget -q -O "$TEMP_BINARY" "$DOWNLOAD_URL"; then
        echo -e "${RED}✗ Failed to download from $DOWNLOAD_URL${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ Neither curl nor wget found. Please install one of them.${NC}"
    exit 1
fi

# Make executable
chmod +x "$TEMP_BINARY"
echo -e "${GREEN}✓ Downloaded successfully${NC}"

# Installation options
echo ""
echo "Installation paths:"
echo "  1. Install to /usr/local/bin (requires sudo)"
echo "  2. Copy to current directory"
echo "  3. Just verify the binary"
read -p "Choose [1-3]: " choice

case "$choice" in
    1)
        echo ""
        echo "Installing to /usr/local/bin..."
        if sudo mv "$TEMP_BINARY" /usr/local/bin/tantex; then
            echo -e "${GREEN}✓ Installed to /usr/local/bin/tantex${NC}"
            echo ""
            echo "You can now run: ${YELLOW}tantex${NC}"
        else
            echo -e "${RED}✗ Installation failed${NC}"
            exit 1
        fi
        ;;
    2)
        cp "$TEMP_BINARY" ./tantex
        echo -e "${GREEN}✓ Copied to ./tantex${NC}"
        echo ""
        echo "You can now run: ${YELLOW}./tantex${NC}"
        ;;
    3)
        cp "$TEMP_BINARY" ./tantex
        echo -e "${GREEN}✓ Binary copied to ./tantex (not in PATH)${NC}"
        echo ""
        echo "You can run: ${YELLOW}./tantex${NC}"
        ;;
    *)
        echo -e "${RED}✗ Invalid choice${NC}"
        exit 1
        ;;
esac

# Verify by checking version output
echo ""
echo "Verifying installation..."
if [ "$choice" = "1" ]; then
    BINARY_PATH="tantex"
else
    BINARY_PATH="./tantex"
fi

if $BINARY_PATH --help &> /dev/null || $BINARY_PATH 2>&1 | grep -q "tantex"; then
    echo -e "${GREEN}✓ tantex is ready!${NC}"
    echo ""
    echo "Next steps:"
    echo "  Start the server:  $BINARY_PATH"
    echo "  View docs:         https://tna5.github.io/tantex/"
else
    echo -e "${YELLOW}⚠ Could not verify installation${NC}"
fi
