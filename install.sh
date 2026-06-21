#!/bin/sh
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

printf "${BOLD}=== tantex installer ===${NC}\n\n"

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

if [ "$ARCH" = "aarch64" ]; then
    ARCH="arm64"
fi

case "$OS" in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="tantex-linux-x86_64"
        else
            printf "${RED}✗ Unsupported architecture: $ARCH on Linux (supported: x86_64)${NC}\n"
            exit 1
        fi
        ;;
    Darwin)
        if [ "$ARCH" = "x86_64" ]; then
            BINARY_NAME="tantex-macos-x86_64"
        elif [ "$ARCH" = "arm64" ]; then
            BINARY_NAME="tantex-macos-arm64"
        else
            printf "${RED}✗ Unsupported architecture: $ARCH on macOS (supported: x86_64, arm64)${NC}\n"
            exit 1
        fi
        ;;
    *)
        printf "${RED}✗ Unsupported OS: $OS (supported: Linux, macOS)${NC}\n"
        exit 1
        ;;
esac

printf "Detected: $OS $ARCH\n\n"

# Get latest version from GitHub redirect
printf "Fetching latest version...\n"
VERSION=$(curl -fsSLI -o /dev/null -w "%{url_effective}" "https://github.com/tna5/tantex/releases/latest" | sed 's|.*/tag/||' | tr -d '[:space:]')

if [ -z "$VERSION" ]; then
    printf "${RED}✗ Could not determine latest version${NC}\n"
    exit 1
fi

printf "Latest version: ${YELLOW}$VERSION${NC}\n\n"

INSTALL_DIR="tantex-$VERSION"
DOWNLOAD_URL="https://github.com/tna5/tantex/releases/latest/download/$BINARY_NAME"

# Download binary
printf "Downloading $BINARY_NAME...\n"
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

if command -v curl > /dev/null 2>&1; then
    curl -fsSL -o "$TEMP_DIR/tantex" "$DOWNLOAD_URL"
elif command -v wget > /dev/null 2>&1; then
    wget -q -O "$TEMP_DIR/tantex" "$DOWNLOAD_URL"
else
    printf "${RED}✗ Neither curl nor wget found${NC}\n"
    exit 1
fi

chmod +x "$TEMP_DIR/tantex"
printf "${GREEN}✓ Downloaded${NC}\n\n"

# Create directory structure
mkdir -p "$INSTALL_DIR/data"

mv "$TEMP_DIR/tantex" "$INSTALL_DIR/tantex"

# Write default config file
cat > "$INSTALL_DIR/data/tantex.config.json" << 'EOF'
{
  "socket_path": "/tmp/tantex.sock",
  "http_port": 7200,
  "data_dir": "./data",
  "writer_heap_size": 4000000000,
  "shm_buffer_size": 268435456,
  "num_indexing_threads": 8,
  "index_threads_pct": 63,
  "auto_commit_doc_count": 10000000,
  "auto_commit_interval_secs": 30,
  "hard_commit_multiplier": 4,
  "merge_target_docs": 20000000,
  "max_merge_factor": 10,
  "min_num_segments": 2
}
EOF

# Print result
printf "${GREEN}✓ Installed to ${BOLD}./$INSTALL_DIR${NC}\n\n"
printf "  $INSTALL_DIR\n"
printf "  ├── data/\n"
printf "  │   └── tantex.config.json\n"
printf "  └── tantex\n"
printf "\n"
printf "Start the server:\n"
printf "  ${YELLOW}cd $INSTALL_DIR && ./tantex${NC}\n\n"
printf "Dashboard: http://localhost:7200\n"
printf "Docs:      https://tna5.github.io/tantex/\n"
