#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/debug"
DIST_DIR="$ROOT_DIR/dist"
PKG_DIR="$DIST_DIR/pkg"
WASM_TOOLCHAIN="${WASM_TOOLCHAIN:-nightly-2025-05-01}"

cd "$ROOT_DIR"

if ! rustup toolchain list | grep -q "^${WASM_TOOLCHAIN}"; then
  rustup toolchain install "$WASM_TOOLCHAIN" --component rust-src --target wasm32-unknown-unknown
else
  if ! rustup component list --toolchain "$WASM_TOOLCHAIN" --installed | grep -qx "rust-src"; then
    rustup component add rust-src --toolchain "$WASM_TOOLCHAIN"
  fi

  if ! rustup target list --toolchain "$WASM_TOOLCHAIN" --installed | grep -qx "wasm32-unknown-unknown"; then
    rustup target add wasm32-unknown-unknown --toolchain "$WASM_TOOLCHAIN"
  fi
fi

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory' \
  cargo +"$WASM_TOOLCHAIN" build \
  -Z build-std=panic_abort,std \
  --features wasm_threads \
  --target wasm32-unknown-unknown

rm -rf "$DIST_DIR"
mkdir -p "$PKG_DIR"

wasm-bindgen \
  --target web \
  --out-dir "$PKG_DIR" \
  "$TARGET_DIR/sdf.wasm"

if [ -d "$PKG_DIR/snippets" ]; then
  find "$PKG_DIR/snippets" -name "workerHelpers.no-bundler.js" -exec perl -0pi -e \
    's/await pkg\.default\(data\.module, data\.memory\);/await pkg.default({ module_or_path: data.module, memory: data.memory });/g' \
    {} +
fi

cp "$ROOT_DIR/index.html" "$DIST_DIR/index.html"
cp "$ROOT_DIR/favicon.svg" "$DIST_DIR/favicon.svg"
cp -R "$ROOT_DIR/assets" "$DIST_DIR/assets"
