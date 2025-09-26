#!/usr/bin/env bash

set -euo pipefail

# Hardcode the target triple for Windows builds.
TARGET_TRIPLE="x86_64-pc-windows-gnu"

echo "--- Preparing to build artifacts for Windows target: $TARGET_TRIPLE ---"

# Ensure the artifacts directory exists.
ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

# Create a temporary directory to stage the files.
TMP_DIR=$(mktemp -d -p "$ARTIFACTS_DIR")

# Ensure the temporary directory is removed when the script exits.
trap 'rm -rf -- "$TMP_DIR"' EXIT

echo "--- Building backend ---"
# Build the backend binary for Windows.
(cd backend && STATIC_DIR=./static cargo zigbuild --release --target "$TARGET_TRIPLE")

echo "--- Building frontend ---"
# Build the frontend assets.
(cd frontend && NOMAP=T pnpm build)

echo "--- Assembling artifacts in $TMP_DIR ---"

# Move the frontend build output to the 'static' directory.
mv frontend/build "$TMP_DIR/static"

# Move the backend Windows binary (with .exe extension) into our staging area.
mv "backend/target/$TARGET_TRIPLE/release/llumen.exe" "$TMP_DIR/llumen.exe"

echo "--- Creating zip archive ---"

# Create a zip archive of the staged files.
ZIP_NAME="$TARGET_TRIPLE.zip"
ZIP_PATH="$ARTIFACTS_DIR/$ZIP_NAME"

# From within the temp directory, create the zip file in the artifacts directory.
(cd "$TMP_DIR" && zip -r "../$ZIP_NAME" .)

echo "--- Artifact created successfully: $ZIP_PATH ---'
