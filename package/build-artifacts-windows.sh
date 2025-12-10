#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE=${1:?Target triple must be provided}

echo "--- Preparing to build artifacts for Windows target: $TARGET_TRIPLE ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

TMP_DIR=$(mktemp -d -p "$ARTIFACTS_DIR")

trap 'rm -rf -- "$TMP_DIR"' EXIT

echo "--- Building backend ---"
if [[ "$TARGET_TRIPLE" == *"-msvc"* ]]; then
  (cd backend && STATIC_DIR=./static cargo build --release --target "$TARGET_TRIPLE")
else
  (cd backend && STATIC_DIR=./static cargo zigbuild --release --target "$TARGET_TRIPLE")
fi

echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)
# find build/ -type f -name "*.gz" -delete

echo "--- Assembling artifacts in $TMP_DIR ---"

mkdir "$TMP_DIR/llumen"

mv frontend/build "$TMP_DIR/llumen/static"

touch "$TMP_DIR/llumen/.env"

mv "backend/target/$TARGET_TRIPLE/release/backend.exe" "$TMP_DIR/llumen/llumen.exe"

echo "--- Creating zip archive ---"

ZIP_NAME="$TARGET_TRIPLE.zip"
ZIP_PATH="$ARTIFACTS_DIR/$ZIP_NAME"

(cd "$TMP_DIR" && 7z a -tzip "../$ZIP_NAME" ./)

echo "--- Artifact created successfully: $ZIP_PATH ---"
