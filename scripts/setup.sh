#!/bin/bash
# Setup script for Linux/Mac
# Creates symlink to make the Bobbin addon available in the Godot test project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

ADDON_SRC="$PROJECT_ROOT/bindings/godot/addons/bobbin"
ADDON_DEST="$PROJECT_ROOT/test-projects/godot/bobbin-test-project/addons/bobbin"

# Create addons directory if it doesn't exist
mkdir -p "$(dirname "$ADDON_DEST")"

# Remove existing link/directory if present
if [ -L "$ADDON_DEST" ] || [ -d "$ADDON_DEST" ]; then
    rm -rf "$ADDON_DEST"
fi

if [ "$CI" = "true" ]; then
    # CI: copy (simple, universal)
    cp -r "$ADDON_SRC" "$ADDON_DEST"
    echo "Copied addon to test project (CI mode)"
else
    # Local dev: symlink (live editing)
    ln -s "../../../../../bindings/godot/addons/bobbin" "$ADDON_DEST"
    echo "Symlinked addon to test project (dev mode)"
fi

echo "Done! Addon is now available at: $ADDON_DEST"

# Build Docker image for containerized builds
echo ""
echo "Building Docker image for containerized builds..."
(cd "$PROJECT_ROOT" && docker compose build)
echo "Docker image 'bobbin-build' ready!"