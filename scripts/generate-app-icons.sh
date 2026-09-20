#!/usr/bin/env bash
# ==============================================================================
# DevFlow: Tauri App Bundle Icon Generator
# Generates rounded-corner dark squircle macOS (.icns) and multi-res PNG icons
# ==============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SVG_SRC="${REPO_ROOT}/assets/devflow-app-icon.svg"
ICONS_DIR="${REPO_ROOT}/clients/tauri_app/src-tauri/icons"
TMP_DIR=$(mktemp -d)
ICONSET_DIR="${TMP_DIR}/devflow.iconset"

echo "🎨 Generating Tauri App Icons with rounded corners and dark background..."

mkdir -p "$ICONS_DIR"
mkdir -p "$ICONSET_DIR"

# 1. Render SVG to 1024x1024 master PNG
qlmanage -t -s 1024 -o "$TMP_DIR" "$SVG_SRC" >/dev/null 2>&1
MASTER_PNG="${TMP_DIR}/devflow-app-icon.svg.png"

if [ ! -f "$MASTER_PNG" ]; then
    echo "❌ Failed to render master PNG from ${SVG_SRC}"
    rm -rf "$TMP_DIR"
    exit 1
fi

# 2. Generate macOS iconset resolutions
sips -z 16 16     "$MASTER_PNG" --out "${ICONSET_DIR}/icon_16x16.png" >/dev/null
sips -z 32 32     "$MASTER_PNG" --out "${ICONSET_DIR}/icon_16x16@2x.png" >/dev/null
sips -z 32 32     "$MASTER_PNG" --out "${ICONSET_DIR}/icon_32x32.png" >/dev/null
sips -z 64 64     "$MASTER_PNG" --out "${ICONSET_DIR}/icon_32x32@2x.png" >/dev/null
sips -z 128 128   "$MASTER_PNG" --out "${ICONSET_DIR}/icon_128x128.png" >/dev/null
sips -z 256 256   "$MASTER_PNG" --out "${ICONSET_DIR}/icon_128x128@2x.png" >/dev/null
sips -z 256 256   "$MASTER_PNG" --out "${ICONSET_DIR}/icon_256x256.png" >/dev/null
sips -z 512 512   "$MASTER_PNG" --out "${ICONSET_DIR}/icon_256x256@2x.png" >/dev/null
sips -z 512 512   "$MASTER_PNG" --out "${ICONSET_DIR}/icon_512x512.png" >/dev/null
sips -z 1024 1024 "$MASTER_PNG" --out "${ICONSET_DIR}/icon_512x512@2x.png" >/dev/null

# 3. Compile macOS .icns
iconutil -c icns "$ICONSET_DIR" -o "${ICONS_DIR}/icon.icns"

# 4. Generate standard resolution Tauri PNG icons
sips -z 512 512   "$MASTER_PNG" --out "${ICONS_DIR}/icon.png" >/dev/null
sips -z 32 32     "$MASTER_PNG" --out "${ICONS_DIR}/32x32.png" >/dev/null
sips -z 64 64     "$MASTER_PNG" --out "${ICONS_DIR}/64x64.png" >/dev/null
sips -z 128 128   "$MASTER_PNG" --out "${ICONS_DIR}/128x128.png" >/dev/null
sips -z 256 256   "$MASTER_PNG" --out "${ICONS_DIR}/128x128@2x.png" >/dev/null

# 5. Windows / Store square icons
sips -z 30 30     "$MASTER_PNG" --out "${ICONS_DIR}/Square30x30Logo.png" >/dev/null
sips -z 44 44     "$MASTER_PNG" --out "${ICONS_DIR}/Square44x44Logo.png" >/dev/null
sips -z 71 71     "$MASTER_PNG" --out "${ICONS_DIR}/Square71x71Logo.png" >/dev/null
sips -z 89 89     "$MASTER_PNG" --out "${ICONS_DIR}/Square89x89Logo.png" >/dev/null
sips -z 107 107   "$MASTER_PNG" --out "${ICONS_DIR}/Square107x107Logo.png" >/dev/null
sips -z 142 142   "$MASTER_PNG" --out "${ICONS_DIR}/Square142x142Logo.png" >/dev/null
sips -z 150 150   "$MASTER_PNG" --out "${ICONS_DIR}/Square150x150Logo.png" >/dev/null
sips -z 284 284   "$MASTER_PNG" --out "${ICONS_DIR}/Square284x284Logo.png" >/dev/null
sips -z 310 310   "$MASTER_PNG" --out "${ICONS_DIR}/Square310x310Logo.png" >/dev/null
sips -z 50 50     "$MASTER_PNG" --out "${ICONS_DIR}/StoreLogo.png" >/dev/null

# Clean up
rm -rf "$TMP_DIR"

echo "✅ All Tauri Desktop icons successfully generated in ${ICONS_DIR}."
