#!/usr/bin/env bash
# usage-radar — one-click developer launcher (macOS / Linux)

set -e

# Run from the project root
cd "$(dirname "$0")/.."

# Fix common PATH gotcha when Rust is freshly installed
if [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo ""
echo "==> usage-radar launcher"
echo ""

check_tool() {
    local cmd="$1"
    local hint="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        local ver
        ver=$("$cmd" --version 2>&1 | head -n1)
        printf "  [OK]      %-6s %s\n" "$cmd" "$ver"
        return 0
    else
        printf "  [MISSING] %-6s (install: %s)\n" "$cmd" "$hint"
        return 1
    fi
}

echo "Checking prerequisites..."
cargo_ok=0; bun_ok=0
check_tool cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" && cargo_ok=1
check_tool bun   "curl -fsSL https://bun.sh/install | bash"                       && bun_ok=1

if [ $cargo_ok -ne 1 ] || [ $bun_ok -ne 1 ]; then
    echo ""
    echo "Install the missing tools, open a NEW terminal, then re-run this script."
    exit 1
fi

echo ""

if [ ! -d node_modules ]; then
    echo "==> Installing frontend dependencies (bun install)..."
    bun install
else
    echo "  node_modules present, skipping install"
fi

echo ""
echo "==> Launching usage-radar (Ctrl+C to stop)"
echo "    First run compiles Rust crates — may take 5-10 minutes."
echo ""

bun run tauri dev
