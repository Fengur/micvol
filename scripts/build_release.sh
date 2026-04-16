#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_DIR/dist"

echo "Building micvol release..."
cd "$PROJECT_DIR"
cargo build --release

echo "Copying artifacts to dist/..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"
cp target/release/libmicvol.a "$DIST_DIR/"
cp include/micvol.h "$DIST_DIR/"

echo ""
echo "Done. Output:"
echo "  $DIST_DIR/libmicvol.a"
echo "  $DIST_DIR/micvol.h"
echo ""
echo "Exported symbols:"
nm -g "$DIST_DIR/libmicvol.a" | grep " T _micvol_" | awk '{print "  " $3}'
