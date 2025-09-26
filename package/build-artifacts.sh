#!/usr/bin/env bash

set -euo pipefail

# Use the first argument as the target triple, otherwise default to the host triple.
TARGET_TRIPLE=${1:-$(rustc -vV | grep 'host:' | awk '{print $2}')}

echo "--- Preparing to build artifacts for target: $TARGET_TRIPLE ---"

# Ensure the artifacts directory exists.
ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

# Create a temporary directory to stage the files.
TMP_DIR=$(mktemp -d -p "$ARTIFACTS_DIR")

# Ensure the temporary directory is removed when the script exits.
trap 'rm -rf -- "$TMP_DIR"' EXIT

echo "--- Building backend ---"
# Build the backend binary. The STATIC_DIR env var is likely used at compile time
# to set the path where the backend will look for static assets at runtime.
(cd backend && STATIC_DIR=./static cargo zigbuild --release --target "$TARGET_TRIPLE")

echo "--- Building frontend ---"
# Build the frontend assets. NOMAP=T likely disables source maps.
(cd frontend && NOMAP=T pnpm build)

echo "--- Assembling artifacts in $TMP_DIR ---"

# The backend is configured to serve files from a 'static' directory.
# The frontend build output from 'pnpm build' is in 'frontend/build'.
# We move the frontend build output to the 'static' directory in our temp staging area.
mv frontend/build "$TMP_DIR/static"

# Move the backend binary into our staging area.
mv "backend/target/$TARGET_TRIPLE/release/llumen" "$TMP_DIR/llumen"

echo "--- Creating compressed tarball ---"

# Create a gzipped tarball of the staged files.
# The tarball will be named after the target triple (e.g., x86_64-unknown-linux-gnu.tar.gz).
TARBALL_PATH="$ARTIFACTS_DIR/$TARGET_TRIPLE.tar.gz"
tar -czf "$TARBALL_PATH" -C "$TMP_DIR" .

echo "--- Artifact created successfully: $TARBALL_PATH ---"
