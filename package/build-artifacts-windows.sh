#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE="x86_64-pc-windows-gnu"

echo "--- Preparing to build artifacts for Windows target: $TARGET_TRIPLE ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

TMP_DIR=$(mktemp -d -p "$ARTIFACTS_DIR")

trap 'rm -rf -- "$TMP_DIR"' EXIT

echo "--- Building backend ---"
(cd backend && STATIC_DIR=./static cargo zigbuild --release --target "$TARGET_TRIPLE")

echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)

echo "--- Assembling artifacts in $TMP_DIR ---"

mv frontend/build "$TMP_DIR/static"

touch "$TMP_DIR/.env"

mv "backend/target/$TARGET_TRIPLE/release/backend.exe" "$TMP_DIR/llumen.exe"

echo "--- Creating zip archive ---"

ZIP_NAME="$TARGET_TRIPLE.zip"
ZIP_PATH="$ARTIFACTS_DIR/$ZIP_NAME"

(cd "$TMP_DIR" && zip -r "../$ZIP_NAME" .)

echo "--- Artifact created successfully: $ZIP_PATH ---"
