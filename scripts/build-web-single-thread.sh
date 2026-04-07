#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
DIST_DIR="$ROOT_DIR/dist"
PKG_DIR="$DIST_DIR/pkg"

cd "$ROOT_DIR"

cargo build --release --target wasm32-unknown-unknown --no-default-features

rm -rf "$DIST_DIR"
mkdir -p "$PKG_DIR"

wasm-bindgen \
  --target web \
  --out-dir "$PKG_DIR" \
  "$TARGET_DIR/sdf.wasm"

cp "$ROOT_DIR/index-single-thread.html" "$DIST_DIR/index.html"
