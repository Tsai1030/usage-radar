#!/usr/bin/env bash
# usage-radar — produce a release installer (macOS / Linux)

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

CONF="src-tauri/tauri.conf.json"
RESTORE_BUNDLE=0
if command -v node >/dev/null 2>&1; then
    ACTIVE=$(node -e "console.log(require('./$CONF').bundle.active)")
    if [ "$ACTIVE" != "true" ]; then
        echo "==> Temporarily enabling bundle.active in tauri.conf.json"
        node -e "
            const fs = require('fs');
            const c = require('./$CONF');
            c.bundle.active = true;
            fs.writeFileSync('$CONF', JSON.stringify(c, null, 2) + '\n');
        "
        RESTORE_BUNDLE=1
    fi
fi

restore() {
    if [ "$RESTORE_BUNDLE" = "1" ]; then
        echo "==> Restoring bundle.active = false"
        node -e "
            const fs = require('fs');
            const c = require('./$CONF');
            c.bundle.active = false;
            fs.writeFileSync('$CONF', JSON.stringify(c, null, 2) + '\n');
        "
    fi
}
trap restore EXIT

echo "==> Building release (this can take several minutes)..."
bun run tauri build

echo ""
echo "==> Output files:"
find src-tauri/target/release/bundle -type f \( -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" -o -name "*.rpm" -o -name "*.msi" -o -name "*.exe" \) 2>/dev/null | while read -r f; do
    echo "   $f"
done
