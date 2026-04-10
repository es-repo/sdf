#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
DIST_DIR="$ROOT_DIR/dist"
PKG_DIR="$DIST_DIR/pkg"
WASM_TOOLCHAIN="${WASM_TOOLCHAIN:-nightly-2025-05-01}"

cd "$ROOT_DIR"

rustup toolchain install "$WASM_TOOLCHAIN" --component rust-src --target wasm32-unknown-unknown

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory' \
  cargo +"$WASM_TOOLCHAIN" build \
  -Z build-std=panic_abort,std \
  --features wasm_threads \
  --release \
  --target wasm32-unknown-unknown

rm -rf "$DIST_DIR"
mkdir -p "$PKG_DIR"

wasm-bindgen \
  --target web \
  --out-dir "$PKG_DIR" \
  "$TARGET_DIR/sdf.wasm"

find "$PKG_DIR/snippets" -name "workerHelpers.no-bundler.js" -exec perl -0pi -e \
  's/await pkg\.default\(data\.module, data\.memory\);/await pkg.default({ module_or_path: data.module, memory: data.memory });/g' \
  {} +

cp "$ROOT_DIR/index.html" "$DIST_DIR/index.html"
cp "$ROOT_DIR/favicon.svg" "$DIST_DIR/favicon.svg"
