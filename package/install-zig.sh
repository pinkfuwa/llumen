#!/usr/bin/env bash

set -euo pipefail

zig_version=${1:-0.15.1}
install_dir=${2:-/opt/zig}

host_os=$(uname -s)
case "$host_os" in
  Linux)
    platform=linux
    extension=tar.xz
    ;;
  MINGW*|MSYS*)
    platform=windows
    extension=zip
    ;;
  *)
    echo "Unsupported Zig host OS: $host_os" >&2
    exit 1
    ;;
esac

host_arch=$(uname -m)
case "$host_arch" in
  x86_64) arch=x86_64 ;;
  aarch64|arm64) arch=aarch64 ;;
  *)
    echo "Unsupported Zig host architecture: $host_arch" >&2
    exit 1
    ;;
esac

case "$zig_version/$platform/$arch" in
  0.13.0/linux/x86_64)
    checksum=d45312e61ebcc48032b77bc4cf7fd6915c11fa16e4aad116b66c9468211230ea
    ;;
  0.13.0/linux/aarch64)
    checksum=041ac42323837eb5624068acd8b00cd5777dac4cf91179e8dad7a7e90dd0c556
    ;;
  0.13.0/windows/x86_64)
    checksum=d859994725ef9402381e557c60bb57497215682e355204d754ee3df75ee3c158
    ;;
  0.15.1/linux/x86_64)
    checksum=c61c5da6edeea14ca51ecd5e4520c6f4189ef5250383db33d01848293bfafe05
    ;;
  0.15.1/linux/aarch64)
    checksum=bb4a8d2ad735e7fba764c497ddf4243cb129fece4148da3222a7046d3f1f19fe
    ;;
  0.15.1/windows/x86_64)
    checksum=91e69e887ca8c943ce9a515df3af013d95a66a190a3df3f89221277ebad29e34
    ;;
  *)
    echo "Unsupported Zig release: $zig_version/$platform/$arch" >&2
    exit 1
    ;;
esac

case "$zig_version" in
  0.13.0) archive_name="zig-$platform-$arch-$zig_version" ;;
  0.15.1) archive_name="zig-$arch-$platform-$zig_version" ;;
esac

install_parent=$(dirname "$install_dir")
mkdir -p "$install_parent"
download_dir=$(mktemp -d "$install_parent/.zig-install.XXXXXX")
trap 'rm -rf "$download_dir"' EXIT
archive="$download_dir/zig.$extension"

echo "Installing Zig $zig_version for $platform/$arch into $install_dir"
curl --fail --location --retry 3 "https://ziglang.org/download/$zig_version/$archive_name.$extension" --output "$archive"
printf '%s  %s\n' "$checksum" "$archive" | sha256sum --check

if [ "$platform" = windows ]; then
  7z x -y "$(cygpath -w "$archive")" "-o$(cygpath -w "$download_dir")"
else
  tar -xf "$archive" -C "$download_dir"
fi

if [ -d "$install_dir" ]; then
  cp -a "$download_dir/$archive_name/." "$install_dir/"
else
  mv "$download_dir/$archive_name" "$install_dir"
fi

if [ -n "${GITHUB_PATH:-}" ]; then
  install_dir=$(cd "$install_dir" && pwd -P)
  if [ "$platform" = windows ]; then
    install_dir=$(cygpath -w "$install_dir")
  fi
  printf '%s\n' "$install_dir" >> "$GITHUB_PATH"
fi
