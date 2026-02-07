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

if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "Installing tauri-cli v1..."
  cargo install tauri-cli --version 1.5.6 --locked
else
  if ! cargo tauri --version | grep -q '^1\\.'; then
    echo "Re-installing tauri-cli v1..."
    cargo install tauri-cli --version 1.5.6 --locked
  fi
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

echo "Building macOS app..."
cargo tauri build

OUT_APP="$ROOT/target/release/bundle/macos/RustStream.app"
if [ ! -d "$OUT_APP" ]; then
  echo "App bundle not found at $OUT_APP" >&2
  exit 1
fi

cp -R "$OUT_APP" "$HOME/Desktop/RustStream.app"

echo "Done. RustStream.app is on your Desktop."
