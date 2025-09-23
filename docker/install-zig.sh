#!/bin/sh
set -ex

echo "Detecting architecture..."
arch=$(uname -m)
echo "Architecture: $arch"

case $arch in
    "x86_64")
        zig_arch="x86_64"
        ;;
    "aarch64")
        zig_arch="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $arch"
        exit 1
        ;;
esac

zig_version="0.13.0"
zig_tarball="zig-linux-${zig_arch}-${zig_version}.tar.xz"
zig_url="https://ziglang.org/download/${zig_version}/${zig_tarball}"

echo "Downloading Zig from $zig_url..."
curl -L -o "${zig_tarball}" "${zig_url}"

echo "Extracting Zig..."
tar -xf "${zig_tarball}"

echo "Installing Zig..."
mkdir -p /opt/zig
mv "zig-linux-${zig_arch}-${zig_version}/zig" /opt/zig/
rm -rf "zig-linux-${zig_arch}-${zig_version}" "${zig_tarball}"

echo "Zig installation complete."
