#!/usr/bin/env bash
# ==============================================================================
# DevFlow CLI — 1-Line Remote Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install-cli.sh | bash
# ==============================================================================
set -e

BOLD="\033[1m"
GREEN="\033[0;32m"
CYAN="\033[0;36m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
DIM="\033[2m"
RESET="\033[0m"

echo -e "${CYAN}${BOLD}⚡ DevFlow CLI Installer${RESET}"
echo -e "${DIM}Universal development runner, multi-target TUI hub & MCP server${RESET}\n"

# 1. Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        OS_NAME="macos"
        ;;
    Linux)
        OS_NAME="linux"
        ;;
    *)
        echo -e "${RED}✗ Error: Unsupported operating system: $OS${RESET}"
        exit 1
        ;;
esac

case "$ARCH" in
    arm64|aarch64)
        if [ "$OS_NAME" = "linux" ]; then
            ARCH_NAME="aarch64"
        else
            ARCH_NAME="arm64"
        fi
        ;;
    x86_64|amd64)
        ARCH_NAME="x86_64"
        ;;
    *)
        echo -e "${RED}✗ Error: Unsupported architecture: $ARCH${RESET}"
        exit 1
        ;;
esac

ASSET_NAME="devflow-cli-${OS_NAME}-${ARCH_NAME}.tar.gz"
REPO="cmdworks/devflow"
INSTALL_DIR="${DEVFLOW_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${DEVFLOW_VERSION:-latest}"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"
fi

echo -e "🖥️  Platform:   ${BOLD}${OS_NAME} (${ARCH_NAME})${RESET}"
echo -e "📁 Target dir: ${BOLD}${INSTALL_DIR}${RESET}"
echo -e "⬇️  Downloading: ${CYAN}${DOWNLOAD_URL}${RESET}\n"

mkdir -p "$INSTALL_DIR"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# 2. Download release archive
if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TMP_DIR/devflow.tar.gz"; then
    echo -e "${RED}✗ Error: Failed to download release archive from GitHub.${RESET}"
    echo -e "  Please verify your internet connection or check releases at: https://github.com/${REPO}/releases"
    exit 1
fi

# 3. Extract and install binary
tar -xzf "$TMP_DIR/devflow.tar.gz" -C "$TMP_DIR"

if [ -f "$TMP_DIR/devflow" ]; then
    mv "$TMP_DIR/devflow" "$INSTALL_DIR/devflow"
    chmod +x "$INSTALL_DIR/devflow"
    echo -e "${GREEN}✓ Installed CLI binary:${RESET} $INSTALL_DIR/devflow"
else
    echo -e "${RED}✗ Error: Binary 'devflow' not found in downloaded archive.${RESET}"
    exit 1
fi

# 4. macOS Gatekeeper Quarantine Clearance (xattr -cr)
if [ "$OS_NAME" = "macos" ]; then
    if command -v xattr >/dev/null 2>&1; then
        xattr -cr "$INSTALL_DIR/devflow" 2>/dev/null || true
        echo -e "${GREEN}✓ macOS Gatekeeper quarantine flags cleared.${RESET}"
    fi
fi

# 5. Verify PATH
echo ""
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠ Notice:${RESET} '$INSTALL_DIR' is not in your current PATH."
    echo -e "  To run 'devflow' from anywhere, add this line to your ${CYAN}~/.zshrc${RESET} or ${CYAN}~/.bashrc${RESET}:"
    echo -e "\n    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n"
else
    echo -e "${GREEN}✓ '$INSTALL_DIR' is active in your PATH.${RESET}"
fi

echo -e "${GREEN}${BOLD}🎉 DevFlow CLI successfully installed!${RESET}\n"
echo -e "Get started:"
echo -e "  • ${CYAN}devflow${RESET}             Open interactive TUI Hub"
echo -e "  • ${CYAN}devflow doctor${RESET}      Verify toolchain and SDK health"
echo -e "  • ${CYAN}devflow mcp connect all${RESET} Auto-connect Claude, Cursor, Antigravity, VS Code"
echo ""
