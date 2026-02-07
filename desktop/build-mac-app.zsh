#!/usr/bin/env zsh
set -euo pipefail

ROOT="/Users/louis/subroot/coding/streaming"
APP_DIR="$ROOT/desktop/src-tauri"

cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but not found. Install Rust first." >&2
  exit 1
fi

if command -v rustup >/dev/null 2>&1; then
  echo "Updating Rust toolchain..."
  rustup update stable
fi

echo "Building backend (release)..."
cargo build -p streaming-app --release

BIN="$ROOT/target/release/ruststream"
if [ ! -f "$BIN" ]; then
  echo "Backend binary not found at $BIN" >&2
  exit 1
fi

mkdir -p "$APP_DIR/bin"
cp "$BIN" "$APP_DIR/bin/ruststream"

cd "$APP_DIR"

echo "Building desktop shell (release)..."
cargo build -p ruststream-desktop --release

DESKTOP_BIN="$ROOT/target/release/ruststream-desktop"
if [ ! -f "$DESKTOP_BIN" ]; then
  echo "Desktop binary not found at $DESKTOP_BIN" >&2
  exit 1
fi

APP_OUT="$HOME/Desktop/RustStream.app"
CONTENTS="$APP_OUT/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RESOURCES_DIR="$CONTENTS/Resources"

echo "Packaging .app..."
rm -rf "$APP_OUT"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR/bin"

cp "$DESKTOP_BIN" "$MACOS_DIR/RustStream"
chmod +x "$MACOS_DIR/RustStream"

cp "$BIN" "$RESOURCES_DIR/bin/ruststream"

cat > "$CONTENTS/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>RustStream</string>
  <key>CFBundleDisplayName</key>
  <string>RustStream</string>
  <key>CFBundleIdentifier</key>
  <string>com.biggorilla121.ruststream</string>
  <key>CFBundleExecutable</key>
  <string>RustStream</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleVersion</key>
  <string>1.0.1</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0.1</string>
</dict>
</plist>
PLIST

echo "Done. RustStream.app is on your Desktop."
