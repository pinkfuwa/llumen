#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE=${1:-$(rustc -vV | grep 'host:' | awk '{print $2}')}

echo "--- Preparing to build artifacts for target: $TARGET_TRIPLE ---"

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

mv "backend/target/$TARGET_TRIPLE/release/backend" "$TMP_DIR/llumen"

echo "--- Creating compressed tarball ---"

TARBALL_PATH="$ARTIFACTS_DIR/$TARGET_TRIPLE.tar.gz"
tar -czf "$TARBALL_PATH" -C "$TMP_DIR" .

echo "--- Artifact created successfully: $TARBALL_PATH ---"
