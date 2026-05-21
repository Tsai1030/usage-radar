#!/usr/bin/env bash
# usage-radar — produce a release installer (macOS / Linux)
# Uses `tauri build --bundles all` so we don't have to mutate tauri.conf.json.

set -e

cd "$(dirname "$0")/.."

if [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo ""
echo "==> usage-radar release build"
echo ""

if [ ! -d node_modules ]; then
    echo "==> Installing frontend dependencies..."
    bun install
fi

echo "==> Building release (this can take several minutes)..."
case "$(uname -s)" in
    Darwin*) BUNDLES="dmg app" ;;
    Linux*)  BUNDLES="deb appimage rpm" ;;
    *)       BUNDLES="msi nsis" ;;
esac
bun run tauri build --bundles $BUNDLES

echo ""
echo "==> Output files:"
find src-tauri/target/release/bundle -type f \( -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" -o -name "*.rpm" -o -name "*.msi" -o -name "*.exe" \) 2>/dev/null | while read -r f; do
    echo "   $f"
done
