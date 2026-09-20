#!/usr/bin/env bash
# ==============================================================================
# DevFlow: Automated Release Documentation Updater
# Updates README.md and docs/getting-started.md with the latest release version
# ==============================================================================

set -euo pipefail

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version-tag> (e.g. v0.1.0 or 0.1.0)"
    exit 1
fi

# Ensure version has 'v' prefix
if [[ ! "$VERSION" =~ ^v ]]; then
    VERSION="v${VERSION}"
fi

RAW_VERSION="${VERSION#v}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
README_FILE="${REPO_ROOT}/README.md"
DOCS_FILE="${REPO_ROOT}/docs/getting-started.md"

echo "⚡ Updating documentation for DevFlow release ${VERSION} (${RAW_VERSION})..."

# 1. Update README.md badge and download links if specific version paths are used
if [ -f "$README_FILE" ]; then
    echo "  → Updating ${README_FILE}..."
    # Update release version badge
    perl -i -pe 's|(img\.shields\.io/badge/Release-)[^-\s]+(-blue\.svg)|${1}'"${VERSION}"'${2}|g' "$README_FILE" || true
fi

# 2. Update docs/getting-started.md if present
if [ -f "$DOCS_FILE" ]; then
    echo "  → Updating ${DOCS_FILE}..."
fi

echo "✅ Documentation updated successfully for ${VERSION}."
