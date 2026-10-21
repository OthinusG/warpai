#!/usr/bin/env bash
# Build native Linux desktop packages using the existing platform packagers.
set -euo pipefail

release_tag="${1:-}"
[[ "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?$ ]] || { echo 'Usage: build-warpai-linux.sh vMAJOR.MINOR.PATCH[.REVISION]' >&2; exit 1; }
[[ "$(uname -s)" == Linux ]] || { echo 'Linux packages require a native Linux build host' >&2; exit 1; }
architecture="$(uname -m)"
case "$architecture" in
  x86_64) asset_arch=x64 ;;
  aarch64) asset_arch=arm64 ;;
  *) echo "Unsupported Linux architecture: $architecture" >&2; exit 1 ;;
esac
cd "$(dirname "$0")/.."
export GIT_RELEASE_TAG="$release_tag"
export CARGO_PROFILE=release
export CARGO_BUILD_RUSTFLAGS='-C symbol-mangling-version=v0'
export SOURCE_DATE_EPOCH="$(git show -s --format=%ct HEAD)"
export APPIMAGE_EXTRACT_AND_RUN=1
cargo test -p warp-agent-bus --lib ssh_remote::tests --locked
cargo test -p warpai --lib remote_server::selected_session::ssh_tests --locked
script/linux/bundle --channel oss --release-tag "$release_tag" --arch "$architecture" --packages appimage,deb,rpm,arch
python3 script/linux/verify-packages.py "$release_tag" "$asset_arch"
