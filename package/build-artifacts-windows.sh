#!/usr/bin/env bash

set -euo pipefail

TARGET_TRIPLE=${1:?Target triple must be provided}
CARGO_FEATURES=${2:-cli}

echo "--- Preparing to build artifacts for Windows target: $TARGET_TRIPLE ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)

echo "--- Building backend ---"
CARGO_FEATURES_ARGS=()
if [ -n "$CARGO_FEATURES" ]; then
  CARGO_FEATURES_ARGS=(--features "$CARGO_FEATURES")
fi

if [[ "$TARGET_TRIPLE" == *"-msvc"* ]]; then
  (cd backend && cargo build --release --target "$TARGET_TRIPLE" "${CARGO_FEATURES_ARGS[@]}")
else
  (
    cd backend
    # Mimalloc uses __DATE__/__TIME__; keep this Zig flag out of host MSVC builds.
    TARGET_CFLAGS_VAR="CFLAGS_${TARGET_TRIPLE//-/_}"
    export "$TARGET_CFLAGS_VAR=${!TARGET_CFLAGS_VAR:-${CFLAGS:-}} -Wno-error=date-time"
    cargo zigbuild --release --target "$TARGET_TRIPLE" "${CARGO_FEATURES_ARGS[@]}"
  )
fi

echo "--- Copying binary to artifacts directory ---"

cp "backend/target/$TARGET_TRIPLE/release/backend.exe" "$ARTIFACTS_DIR/llumen-$TARGET_TRIPLE.exe"

echo "--- Artifact created successfully: $ARTIFACTS_DIR/llumen-$TARGET_TRIPLE.exe ---"
