#!/usr/bin/env bash

set -euo pipefail

echo "--- Building artifacts for Docker nightly ---"

ARTIFACTS_DIR="$(pwd)/artifacts"
mkdir -p "$ARTIFACTS_DIR"

# Build frontend once (architecture independent)
echo "--- Building frontend ---"
(cd frontend && NOMAP=T pnpm build)
find frontend/build -type f -name "*.gz" -delete

# Build backend for both architectures
echo "--- Building backend for x86_64-unknown-linux-musl ---"
(cd backend && STATIC_DIR=./static cargo zigbuild --release --target x86_64-unknown-linux-musl)

echo "--- Building backend for aarch64-unknown-linux-musl ---"
(cd backend && STATIC_DIR=./static cargo zigbuild --release --target aarch64-unknown-linux-musl)

# Organize artifacts for Docker
echo "--- Organizing artifacts for Docker ---"
mkdir -p "$ARTIFACTS_DIR/linux/amd64"
mkdir -p "$ARTIFACTS_DIR/linux/arm64"

cp backend/target/x86_64-unknown-linux-musl/release/backend "$ARTIFACTS_DIR/linux/amd64/"
cp backend/target/aarch64-unknown-linux-musl/release/backend "$ARTIFACTS_DIR/linux/arm64/"

echo "--- Artifacts built successfully ---"
echo "Frontend build: frontend/build"
echo "Backend x86_64: $ARTIFACTS_DIR/linux/amd64/backend"
echo "Backend aarch64: $ARTIFACTS_DIR/linux/arm64/backend"
