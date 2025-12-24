#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE=${1:?Target triple must be provided}

echo "--- Preparing to build artifacts for Windows target: $TARGET_TRIPLE ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)

echo "--- Building backend ---"
if [[ "$TARGET_TRIPLE" == *"-msvc"* ]]; then
  (cd backend && cargo build --release --target "$TARGET_TRIPLE")
else
  (cd backend && cargo zigbuild --release --target "$TARGET_TRIPLE")
fi

echo "--- Copying binary to artifacts directory ---"

cp "backend/target/$TARGET_TRIPLE/release/backend.exe" "$ARTIFACTS_DIR/llumen-$TARGET_TRIPLE.exe"

echo "--- Artifact created successfully: $ARTIFACTS_DIR/llumen-$TARGET_TRIPLE.exe ---"
