#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE=${1:-$(rustc -vV | grep 'host:' | awk '{print $2}')}

echo "--- Preparing to build artifacts for target: $TARGET_TRIPLE ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)

echo "--- Assembling artifacts in $TMP_DIR ---"

mkdir "$TMP_DIR/llumen"

mv frontend/build "$TMP_DIR/llumen/static"

touch "$TMP_DIR/llumen/.env"

mv "backend/target/$TARGET_TRIPLE/release/backend" "$TMP_DIR/llumen/llumen"

echo "--- Copying binary to artifacts ---"

BINARY_PATH="$ARTIFACTS_DIR/llumen-$TARGET_TRIPLE"
cp "backend/target/$TARGET_TRIPLE/release/backend" "$BINARY_PATH"

echo "--- Artifact created successfully: $BINARY_PATH ---"
