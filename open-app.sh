#!/usr/bin/env bash
set -e

# Ensure cargo and tools are in PATH
export PATH="$HOME/.cargo/bin:/Users/mkazim/Library/Application Support/Herd/config/nvm/versions/node/v22.22.1/bin:$PATH"

# Move to script directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"

# If release binary exists, run directly; otherwise build first
if [ -f "./target/release/vault-publisher-desktop" ]; then
    echo "Launching Vault Publisher desktop app..."
    ./target/release/vault-publisher-desktop
else
    echo "Building and launching Vault Publisher desktop app..."
    npm --prefix ui run build
    cargo run -p vault-publisher-desktop --release
fi
