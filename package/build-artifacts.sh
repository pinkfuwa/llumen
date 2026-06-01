#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE=${1:-$(rustc -vV | grep 'host:' | awk '{print $2}')}
CARGO_FEATURES=${2:-}

echo "--- Preparing to build artifacts for target: $TARGET_TRIPLE ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)

echo "--- Building backend ---"
CARGO_FEATURES_FLAG=""
if [ -n "$CARGO_FEATURES" ]; then
  CARGO_FEATURES_FLAG="--features $CARGO_FEATURES"
fi

if [[ "$TARGET_TRIPLE" == *"-musl"* ]] || [[ "$TARGET_TRIPLE" == "aarch64-unknown-linux-gnu" ]]; then
  (cd backend && cargo zigbuild --release --target "$TARGET_TRIPLE" $CARGO_FEATURES_FLAG)
else
  (cd backend && cargo build --release --target "$TARGET_TRIPLE" $CARGO_FEATURES_FLAG)
fi

echo "--- Copying binary to artifacts directory ---"

cp "backend/target/$TARGET_TRIPLE/release/backend" "$ARTIFACTS_DIR/llumen-$TARGET_TRIPLE"

echo "--- Artifact created successfully: $ARTIFACTS_DIR/llumen-$TARGET_TRIPLE ---"
