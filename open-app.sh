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

APP_BUNDLE="InSync.app"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
cp "./target/release/vault-publisher-desktop" "$APP_BUNDLE/Contents/MacOS/InSync"
cp "src-tauri/icons/icon.icns" "$APP_BUNDLE/Contents/Resources/icon.icns"

cat << 'PLIST' > "$APP_BUNDLE/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>InSync</string>
    <key>CFBundleIdentifier</key>
    <string>com.insync.app</string>
    <key>CFBundleName</key>
    <string>InSync</string>
    <key>CFBundleDisplayName</key>
    <string>InSync</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

touch "$APP_BUNDLE"

echo "Launching InSync desktop app..."
if [ "$1" == "--fg" ]; then
    ./"$APP_BUNDLE"/Contents/MacOS/InSync
else
    open "$APP_BUNDLE"
fi
