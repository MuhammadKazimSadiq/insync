#!/usr/bin/env bash
set -e

# Ensure cargo and node tools are in PATH
export PATH="$HOME/.cargo/bin:/Users/mkazim/Library/Application Support/Herd/config/nvm/versions/node/v22.22.1/bin:$PATH"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"

if [ "$1" == "--build" ] || [ ! -f "./target/release/vault-publisher-desktop" ]; then
    echo "Building UI frontend..."
    npm --prefix ui run build
    echo "Compiling native desktop binary..."
    cargo build -p vault-publisher-desktop --release
fi

echo "Launching InSync desktop app..."
./target/release/vault-publisher-desktop
