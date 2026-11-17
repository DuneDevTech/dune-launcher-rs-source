#!/usr/bin/env bash
# Linux x86_64 build of the Dune Client launcher: stripped binary, .tar.gz (what `linux-x86_64-tar`
# updates from) and AppImage. Never built with the `dev-offline` QA feature.
# Heavy: one build at a time, nice, -j2.
set -euo pipefail
cd "$(dirname "$0")/.."
VER=$(sed -n 's/^version = "\(.*\)"/\1/p' launcher/Cargo.toml | head -1)
export HORIZON_LAUNCHER_VERSION="$VER"
: "${RUST_FONTCONFIG_DLOPEN:=on}"; export RUST_FONTCONFIG_DLOPEN
(cd launcher && nice -n 10 cargo build --release --locked -j2 --bin dune-launcher)
mkdir -p dist/linux
strip -o dist/linux/dune-launcher launcher/target/release/dune-launcher
chmod 755 dist/linux/dune-launcher
tar -C dist/linux -czf "dist/DuneClient-$VER-linux-x86_64.tar.gz" dune-launcher
packaging/linux/appimage.sh dist/linux/dune-launcher "$VER" dist
sha256sum dist/DuneClient-"$VER"-* > dist/sha256sums-linux.txt
ls -l dist
