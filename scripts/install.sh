#!/usr/bin/env bash
# ==============================================================================
# DevFlow Full Universal Installer (CLI + Desktop GUI Companion)
# Usage: curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install.sh | bash
#   or:  ./scripts/install.sh  (inside cloned repository)
# ==============================================================================
set -e

BOLD="\033[1m"
GREEN="\033[0;32m"
CYAN="\033[0;36m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
DIM="\033[2m"
RESET="\033[0m"

echo -e "${CYAN}${BOLD}⚡ DevFlow Full Installer (CLI + Desktop GUI)${RESET}"
echo -e "${DIM}Universal development runner, multi-target TUI hub & MCP server${RESET}\n"

INSTALL_DIR="${DEVFLOW_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"
REPO_ROOT=""
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/../Cargo.toml" ]; then
    REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
fi

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) OS_NAME="macos" ;;
    Linux)  OS_NAME="linux" ;;
    *)
        echo -e "${RED}✗ Error: Unsupported operating system: $OS${RESET}"
        exit 1
        ;;
esac

case "$ARCH" in
    arm64|aarch64)
        if [ "$OS_NAME" = "linux" ]; then ARCH_NAME="aarch64"; else ARCH_NAME="arm64"; fi
        ;;
    x86_64|amd64)
        ARCH_NAME="x86_64"
        ;;
    *)
        echo -e "${RED}✗ Error: Unsupported architecture: $ARCH${RESET}"
        exit 1
        ;;
esac

REPO="cmdworks/devflow"
VERSION="${DEVFLOW_VERSION:-latest}"

# ==============================================================================
# MODE A: Remote / Pre-built Release Download (No source clone needed)
# ==============================================================================
if [ -z "$REPO_ROOT" ] || [ "$1" = "--release" ] || [ "$1" = "--binary" ]; then
    echo -e "📦 Mode:        ${BOLD}Pre-compiled GitHub Release${RESET}"
    echo -e "🖥️  Platform:    ${BOLD}${OS_NAME} (${ARCH_NAME})${RESET}"
    echo -e "📁 Target dir:  ${BOLD}${INSTALL_DIR}${RESET}\n"

    CLI_ASSET="devflow-cli-${OS_NAME}-${ARCH_NAME}.tar.gz"
    GUI_ASSET="devflow-gui-${OS_NAME}-${ARCH_NAME}.tar.gz"

    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TMP_DIR"' EXIT

    if [ "$VERSION" = "latest" ]; then
        CLI_URL="https://github.com/${REPO}/releases/latest/download/${CLI_ASSET}"
        GUI_URL="https://github.com/${REPO}/releases/latest/download/${GUI_ASSET}"
    else
        CLI_URL="https://github.com/${REPO}/releases/download/${VERSION}/${CLI_ASSET}"
        GUI_URL="https://github.com/${REPO}/releases/download/${VERSION}/${GUI_ASSET}"
    fi

    # 1. Download & Install CLI
    echo -e "${CYAN}▶ Downloading DevFlow CLI (${CLI_ASSET})...${RESET}"
    if curl -fSL --progress-bar "$CLI_URL" -o "$TMP_DIR/cli.tar.gz"; then
        tar -xzf "$TMP_DIR/cli.tar.gz" -C "$TMP_DIR"
        if [ -f "$TMP_DIR/devflow" ]; then
            mv "$TMP_DIR/devflow" "$INSTALL_DIR/devflow"
            chmod +x "$INSTALL_DIR/devflow"
            echo -e "${GREEN}✓ Installed CLI binary:${RESET} $INSTALL_DIR/devflow"
        fi
    else
        echo -e "${YELLOW}⚠ Could not download CLI from $CLI_URL${RESET}"
    fi

    # 2. Download & Install GUI (if supported)
    if [ "$ARCH_NAME" != "aarch64" ] || [ "$OS_NAME" = "macos" ]; then
        echo -e "\n${CYAN}▶ Downloading DevFlow Desktop GUI Companion (${GUI_ASSET})...${RESET}"
        if curl -fSL --progress-bar "$GUI_URL" -o "$TMP_DIR/gui.tar.gz"; then
            tar -xzf "$TMP_DIR/gui.tar.gz" -C "$TMP_DIR"
            if [ "$OS_NAME" = "macos" ]; then
                APP_TARGET="/Applications/DevFlow.app"
                if [ ! -w "/Applications" ]; then
                    APP_TARGET="$HOME/Applications/DevFlow.app"
                    mkdir -p "$HOME/Applications"
                fi
                if [ -d "$TMP_DIR/DevFlow.app" ]; then
                    rm -rf "$APP_TARGET"
                    cp -R "$TMP_DIR/DevFlow.app" "$APP_TARGET"
                elif [ -f "$TMP_DIR/devflow-gui" ]; then
                    mkdir -p "$APP_TARGET/Contents/MacOS" "$APP_TARGET/Contents/Resources"
                    cp "$TMP_DIR/devflow-gui" "$APP_TARGET/Contents/MacOS/devflow-gui"
                    chmod +x "$APP_TARGET/Contents/MacOS/devflow-gui"
                fi
                if command -v xattr >/dev/null 2>&1; then
                    xattr -cr "$APP_TARGET" 2>/dev/null || true
                fi
                # Create CLI wrapper for devflow-gui
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
                echo -e "${GREEN}✓ Installed native macOS app:${RESET} $APP_TARGET"
                echo -e "${GREEN}✓ Installed GUI CLI wrapper:${RESET} $INSTALL_DIR/devflow-gui"
            else
                if [ -f "$TMP_DIR/devflow-gui" ]; then
                    mv "$TMP_DIR/devflow-gui" "$INSTALL_DIR/devflow-gui"
                    chmod +x "$INSTALL_DIR/devflow-gui"
                    echo -e "${GREEN}✓ Installed Desktop GUI binary:${RESET} $INSTALL_DIR/devflow-gui"
                fi
            fi
        else
            echo -e "${YELLOW}⚠ GUI companion not available for this target or version.${RESET}"
        fi
    fi

# ==============================================================================
# MODE B: Local Source Build (Inside cloned repository)
# ==============================================================================
else
    echo -e "📦 Mode:        ${BOLD}Local Source Build${RESET}"
    echo -e "📁 Repository:  ${BOLD}${REPO_ROOT}${RESET}"
    echo -e "📁 Target dir:  ${BOLD}${INSTALL_DIR}${RESET}\n"

    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}✗ Error: Rust toolchain ('cargo') is required to build from source.${RESET}"
        exit 1
    fi

    # 1. Build CLI
    echo -e "${CYAN}▶ Building DevFlow CLI (release mode)...${RESET}"
    cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" -p devflow_cli
    CLI_BIN="$REPO_ROOT/target/release/devflow"
    ln -sf "$CLI_BIN" "$INSTALL_DIR/devflow"
    echo -e "${GREEN}✓ Linked CLI:${RESET} $INSTALL_DIR/devflow -> $CLI_BIN"

    # 2. Build GUI
    if command -v npm >/dev/null 2>&1; then
        echo -e "\n${CYAN}▶ Building DevFlow Desktop GUI Companion...${RESET}"
        (cd "$REPO_ROOT/clients/tauri_app" && npm run build) >/dev/null 2>&1 || (cd "$REPO_ROOT/clients/tauri_app" && npm run build)
        cargo build --release --manifest-path "$REPO_ROOT/clients/tauri_app/src-tauri/Cargo.toml"
        GUI_BIN="$REPO_ROOT/clients/tauri_app/src-tauri/target/release/devflow-desktop"
        if [ -f "$GUI_BIN" ]; then
            if [ "$OS_NAME" = "macos" ]; then
                APP_TARGET="/Applications/DevFlow.app"
                if [ ! -w "/Applications" ]; then
                    APP_TARGET="$HOME/Applications/DevFlow.app"
                    mkdir -p "$HOME/Applications"
                fi
                mkdir -p "$APP_TARGET/Contents/MacOS" "$APP_TARGET/Contents/Resources"
                cp "$GUI_BIN" "$APP_TARGET/Contents/MacOS/devflow-gui"
                chmod +x "$APP_TARGET/Contents/MacOS/devflow-gui"
                cp "$REPO_ROOT/clients/tauri_app/src-tauri/icons/icon.icns" "$APP_TARGET/Contents/Resources/icon.icns" 2>/dev/null || true
                cp "$REPO_ROOT/clients/tauri_app/src-tauri/icons/icon.png" "$APP_TARGET/Contents/Resources/icon.png" 2>/dev/null || true
                cat << EOF > "$INSTALL_DIR/devflow-gui"
#!/usr/bin/env bash
TARGET_DIR="\${1:-\$PWD}"
if [ "\$#" -eq 0 ] || [ "\$#" -eq 1 -a -d "\$1" ]; then
    open -a "$APP_TARGET" --args "\$TARGET_DIR"
else
    "$APP_TARGET/Contents/MacOS/devflow-gui" "\$@"
fi
EOF
                chmod +x "$INSTALL_DIR/devflow-gui"
                echo -e "${GREEN}✓ Installed native macOS app:${RESET} $APP_TARGET"
            else
                ln -sf "$GUI_BIN" "$INSTALL_DIR/devflow-gui"
            fi
            echo -e "${GREEN}✓ Linked GUI:${RESET} $INSTALL_DIR/devflow-gui -> $GUI_BIN"
        fi
    fi
fi

# ==============================================================================
# macOS Gatekeeper Quarantine Bypass (xattr -cr)
# ==============================================================================
if [ "$OS_NAME" = "macos" ]; then
    echo -e "\n${CYAN}▶ Applying macOS Gatekeeper Quarantine Bypass (xattr -cr)...${RESET}"
    if command -v xattr >/dev/null 2>&1; then
        xattr -cr "$INSTALL_DIR/devflow" 2>/dev/null || true
        xattr -cr "$INSTALL_DIR/devflow-gui" 2>/dev/null || true
        [ -d "/Applications/DevFlow.app" ] && xattr -cr "/Applications/DevFlow.app" 2>/dev/null || true
        [ -d "$HOME/Applications/DevFlow.app" ] && xattr -cr "$HOME/Applications/DevFlow.app" 2>/dev/null || true
        echo -e "${GREEN}✓ Quarantine attributes cleared successfully.${RESET}"
    fi
fi

# ==============================================================================
# Verify PATH & Display Success Banner
# ==============================================================================
echo ""
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠ Notice:${RESET} '$INSTALL_DIR' is not in your current PATH."
    echo -e "  Add this line to your ${CYAN}~/.zshrc${RESET} or ${CYAN}~/.bashrc${RESET}:"
    echo -e "\n    ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n"
else
    echo -e "${GREEN}✓ '$INSTALL_DIR' is active in your PATH.${RESET}"
fi

echo -e "${GREEN}${BOLD}🎉 DevFlow successfully installed!${RESET}\n"
echo -e "Quick Start Commands:"
echo -e "  • ${CYAN}devflow${RESET}             Launch interactive TUI Hub"
echo -e "  • ${CYAN}devflow doctor${RESET}      Run toolchain and SDK diagnostics"
echo -e "  • ${CYAN}devflow-gui${RESET}         Launch Desktop Companion App"
echo -e "  • ${CYAN}devflow mcp connect all${RESET} Auto-configure Claude, Cursor, Antigravity, VS Code"
echo ""
