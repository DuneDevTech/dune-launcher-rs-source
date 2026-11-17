#!/usr/bin/env bash
# Windows x64 build of the Dune Client launcher, cross-compiled in Docker (MinGW).
# Output: dist/windows/dune-launcher.exe  (+ DuneClient-<ver>-windows-x64.zip, DuneClient-Setup-<ver>.exe)
# Heavy: run one build at a time (docker --cpus=2, nice).
set -euo pipefail
cd "$(dirname "$0")/.."
VER=$(sed -n 's/^version = "\(.*\)"/\1/p' launcher/Cargo.toml | head -1)
# The shader step needs Microsoft's d3dcompiler_47.dll (a build tool, not shipped, not in the repo): any
# copy, e.g. the one in an Electron win32 zip, via $D3DCOMPILER_DLL (see docker/fxc-wine/d3dc.c).
: "${D3DCOMPILER_DLL:=$HOME/claritymc-build/dune-horizon/tools/d3dcompiler_47.dll}"
[ -f "$D3DCOMPILER_DLL" ] || { echo "set D3DCOMPILER_DLL to a d3dcompiler_47.dll (x64)" >&2; exit 1; }
IMAGE=dune-launcher-win-fxc
docker image inspect $IMAGE >/dev/null 2>&1 || docker build -f docker/windows-gnu.Dockerfile -t $IMAGE docker
mkdir -p dist/windows
docker volume inspect dune-launcher-win-cargo >/dev/null 2>&1 || docker volume create dune-launcher-win-cargo >/dev/null
nice -n 10 docker run --rm --cpus=2 --memory=12g \
  -v "$D3DCOMPILER_DLL":/usr/local/lib/d3dcompiler_47.dll:ro \
  -v "$PWD":/src -v dune-launcher-win-cargo:/opt/cargo/registry \
  -e CARGO_TARGET_DIR=/src/launcher/target-win -e CARGO_BUILD_JOBS=2 \
  -e HORIZON_LAUNCHER_VERSION="$VER" \
  -w /src/launcher $IMAGE \
  cargo build --release --locked --target x86_64-pc-windows-gnu --bin dune-launcher
EXE=launcher/target-win/x86_64-pc-windows-gnu/release/dune-launcher.exe
x86_64-w64-mingw32-strip "$EXE" -o dist/windows/dune-launcher.exe 2>/dev/null || cp "$EXE" dist/windows/dune-launcher.exe
# Portable zip (what the launcher updates itself from) and the per-user installer: the same binary
# under the setup's name (launcher/src/installer.rs: a file name starting with dune-launcher-setup).
cp dist/windows/dune-launcher.exe "dist/windows/dune-launcher-setup.exe"
rm -f "dist/DuneClient-$VER-windows-x64.zip"
python3 - "$VER" <<'PY'
import sys, zipfile, os
ver = sys.argv[1]
with zipfile.ZipFile(f"dist/DuneClient-{ver}-windows-x64.zip", "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    z.write("dist/windows/dune-launcher.exe", "dune-launcher.exe")
    for f in ("LICENSE", "NOTICE-DUNE.md", "THIRD_PARTY_NOTICES.md"):
        if os.path.exists(f):
            z.write(f, f)
PY
cp dist/windows/dune-launcher-setup.exe "dist/DuneClient-Setup-$VER.exe"
ls -l dist
