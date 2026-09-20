#!/usr/bin/env bash
# ==============================================================================
# DevFlow GUI Desktop Companion — 1-Line Remote Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install-gui.sh | bash
# ==============================================================================
set -e

BOLD="\033[1m"
GREEN="\033[0;32m"
CYAN="\033[0;36m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
DIM="\033[2m"
RESET="\033[0m"

echo -e "${CYAN}${BOLD}🖥️ DevFlow Desktop Companion App Installer${RESET}"
echo -e "${DIM}Native GUI dashboard, split terminal logs & MCP inspector${RESET}\n"

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
        echo -e "${RED}✗ Error: Desktop GUI companion is only supported on macOS and Linux.${RESET}"
        exit 1
        ;;
esac

case "$ARCH" in
    arm64|aarch64)
        ARCH_NAME="arm64"
        ;;
    x86_64|amd64)
        ARCH_NAME="x86_64"
        ;;
    *)
        echo -e "${RED}✗ Error: Unsupported architecture: $ARCH${RESET}"
        exit 1
        ;;
esac

ASSET_NAME="devflow-gui-${OS_NAME}-${ARCH_NAME}.tar.gz"
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
if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TMP_DIR/devflow-gui.tar.gz"; then
    echo -e "${RED}✗ Error: Failed to download GUI release archive from GitHub.${RESET}"
    echo -e "  Please verify your internet connection or check releases at: https://github.com/${REPO}/releases"
    exit 1
fi

# 3. Extract and install
tar -xzf "$TMP_DIR/devflow-gui.tar.gz" -C "$TMP_DIR"

if [ "$OS_NAME" = "macos" ]; then
    # macOS: Install DevFlow.app to /Applications or ~/Applications
    APP_TARGET="/Applications/DevFlow.app"
    if [ ! -w "/Applications" ]; then
        APP_TARGET="$HOME/Applications/DevFlow.app"
        mkdir -p "$HOME/Applications"
    fi

    if [ -d "$TMP_DIR/DevFlow.app" ]; then
        rm -rf "$APP_TARGET"
        cp -R "$TMP_DIR/DevFlow.app" "$APP_TARGET"
        echo -e "${GREEN}✓ Installed native macOS app:${RESET} $APP_TARGET"
    elif [ -f "$TMP_DIR/devflow-gui" ]; then
        # Fallback for raw binary archives: wrap into DevFlow.app
        mkdir -p "$APP_TARGET/Contents/MacOS" "$APP_TARGET/Contents/Resources"
        cp "$TMP_DIR/devflow-gui" "$APP_TARGET/Contents/MacOS/devflow-gui"
        chmod +x "$APP_TARGET/Contents/MacOS/devflow-gui"
        echo -e "${GREEN}✓ Installed native macOS app:${RESET} $APP_TARGET"
    fi

    # Clear macOS Gatekeeper Quarantine
    if command -v xattr >/dev/null 2>&1; then
        xattr -cr "$APP_TARGET" 2>/dev/null || true
        echo -e "${GREEN}✓ macOS Gatekeeper quarantine flags cleared.${RESET}"
    fi

    # Create CLI companion wrappers in INSTALL_DIR
    cat << EOF > "$INSTALL_DIR/devflow-gui"
#!/usr/bin/env bash
TARGET_DIR="\${1:-\$PWD}"
if [ "\$#" -eq 0 ] || [ "\$#" -eq 1 -a -d "\$1" ]; then
    if [ -d "/Applications/DevFlow.app" ]; then
        open -a "/Applications/DevFlow.app" --args "\$TARGET_DIR"
    elif [ -d "\$HOME/Applications/DevFlow.app" ]; then
        open -a "\$HOME/Applications/DevFlow.app" --args "\$TARGET_DIR"
    else
        "$APP_TARGET/Contents/MacOS/devflow-gui" "\$@"
    fi
else
    "$APP_TARGET/Contents/MacOS/devflow-gui" "\$@"
fi
EOF
    chmod +x "$INSTALL_DIR/devflow-gui"

    cat << EOF > "$INSTALL_DIR/devflow"
#!/usr/bin/env bash
if [ "\$#" -eq 0 ]; then
    exec "$INSTALL_DIR/devflow-gui"
fi
exec "$APP_TARGET/Contents/MacOS/devflow-gui" "\$@"
EOF
    chmod +x "$INSTALL_DIR/devflow"

    echo -e "${GREEN}✓ Installed CLI wrappers:${RESET} $INSTALL_DIR/devflow, $INSTALL_DIR/devflow-gui"

else
    # Linux: Install binary, icon, and desktop entry
    if [ -f "$TMP_DIR/devflow-gui" ]; then
        mv "$TMP_DIR/devflow-gui" "$INSTALL_DIR/devflow-gui"
        chmod +x "$INSTALL_DIR/devflow-gui"
        ln -sf "$INSTALL_DIR/devflow-gui" "$INSTALL_DIR/devflow"
        echo -e "${GREEN}✓ Installed Desktop GUI binary:${RESET} $INSTALL_DIR/devflow-gui"
        echo -e "${GREEN}✓ Created CLI symlink:${RESET} $INSTALL_DIR/devflow"
    fi

    # Install Linux Desktop Entry & Icon
    ICON_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
    APPS_DIR="$HOME/.local/share/applications"
    mkdir -p "$ICON_DIR" "$APPS_DIR"

    if [ -f "$TMP_DIR/devflow.png" ]; then
        cp "$TMP_DIR/devflow.png" "$ICON_DIR/devflow.png"
    fi

    cat << EOF > "$APPS_DIR/devflow.desktop"
[Desktop Entry]
Name=DevFlow
Comment=Universal Framework-Aware Development Platform & MCP Hub
Exec=$INSTALL_DIR/devflow-gui %U
Icon=devflow
Terminal=false
Type=Application
Categories=Development;IDE;
StartupNotify=true
EOF
    chmod +x "$APPS_DIR/devflow.desktop"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APPS_DIR" 2>/dev/null || true
    fi
    echo -e "${GREEN}✓ Registered Linux desktop application:${RESET} $APPS_DIR/devflow.desktop"
fi

# 5. Verify PATH
echo ""
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠ Notice:${RESET} '$INSTALL_DIR' is not in your current PATH."
    echo -e "  Add this line to your ${CYAN}~/.zshrc${RESET} or ${CYAN}~/.bashrc${RESET}:"
    echo -e "\n    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n"
else
    echo -e "${GREEN}✓ '$INSTALL_DIR' is active in your PATH.${RESET}"
fi

echo -e "${GREEN}${BOLD}🎉 DevFlow Desktop successfully installed!${RESET}\n"
echo -e "Usage:"
echo -e "  • ${CYAN}DevFlow${RESET}              Launch via Spotlight / Launchpad / Applications"
echo -e "  • ${CYAN}devflow-gui .${RESET}        Launch desktop GUI for current directory (detached)"
echo -e "  • ${CYAN}devflow --help${RESET}       Run CLI commands & agent tools"
echo ""
