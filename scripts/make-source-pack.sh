#!/usr/bin/env bash
# LGPL-3.0 compliance pack for a launcher binary: the complete corresponding source of the commit,
# the licence and notices, the offer text, and the build instructions. NOT published automatically:
# the owner approves distribution first. Brand art is replaced by neutral placeholders (it is not
# LGPL; see brand/LICENSE-BRAND.txt); everything needed to rebuild and relink is included.
#
#   scripts/make-source-pack.sh [OUTDIR]     (default: dist/lgpl)
set -euo pipefail
cd "$(dirname "$0")/.."
OUT=${1:-dist/lgpl}
VER=$(sed -n 's/^version = "\(.*\)"/\1/p' launcher/Cargo.toml | head -1)
COMMIT=$(git rev-parse HEAD)
NAME="dune-launcher-rs-$VER-source"
mkdir -p "$OUT"; WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT
git archive --format=tar --prefix="$NAME/" HEAD | tar -x -C "$WORK"
S="$WORK/$NAME"
# Brand art -> placeholders (a solid Noche square / flat background), same file names.
python3 - "$S" <<'PY'
import sys, glob, os, struct, zlib
S = sys.argv[1]
def png(w, h, rgb=(0x15, 0x16, 0x1a)):
    raw = b"".join(b"\x00" + bytes(rgb) * w for _ in range(h))
    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xffffffff)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")
for f in glob.glob(S + "/packaging/icons/hicolor/*/apps/dune-client.png") + [S + "/crates/horizon-theme/icons/horizon-64.png", S + "/brand/dune-logo.png"]:
    if os.path.exists(f):
        n = int(os.path.basename(os.path.dirname(os.path.dirname(f))).split("x")[0]) if "hicolor" in f else 64
        open(f, "wb").write(png(n, n))
for f in glob.glob(S + "/launcher/assets/brand/*"):
    os.remove(f)
open(S + "/launcher/assets/brand/dune-fondo-noche-1920.webp", "wb").write(b"")
PY
python3 - "$S" <<'PY'
# The webp background placeholder must decode: write a 16x9 lossless WebP through PIL if present, else leave
# the build to fall back to the procedural sunset (Picture::new() returns None on an undecodable file).
import sys
try:
    from PIL import Image
    Image.new("RGB", (16, 9), (0x15, 0x16, 0x1a)).save(sys.argv[1] + "/launcher/assets/brand/dune-fondo-noche-1920.webp", lossless=True)
except Exception:
    pass
PY
rm -f "$S"/brand/*.svg "$S"/packaging/icons/dune.ico "$S"/packaging/icons/dune.icns "$S"/packaging/icons/hicolor/scalable/apps/dune-client.svg
# keep build-time includes valid
printf '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect width="16" height="16" fill="#15161a"/></svg>\n' > "$S/packaging/icons/hicolor/scalable/apps/dune-client.svg"
cp "$S/LICENSE" "$WORK/"; cp -r "$S/LICENSES" "$WORK/LICENSES" 2>/dev/null || true
cat > "$S/OFERTA-LGPL.txt" <<OFFER
Dune Client launcher $VER  (commit $COMMIT)
===========================================================

This program is a modified version of the Horizon launcher
(https://github.com/horizon-rust/horizon, (c) 2026 Ludovic Marie and the Horizon contributors) and is
licensed under the GNU Lesser General Public License, version 3 or (at your option) any later version
(see LICENSE and LICENSES/). Other components keep their own licences (THIRD_PARTY_NOTICES.md and
third_party_licenses/). Our changes are described in NOTICE-DUNE.md.

WRITTEN OFFER. Dune Development will give any person who received a copy of this program the complete
corresponding source code of the exact version they received (this archive: $NAME.tar.gz), together with the
scripts to build and install it, for at least three years from the day of distribution and for as long as we
distribute that version, at no charge beyond the cost of delivery. Ask at https://duneclient.com (contact
page) naming the version and commit above.

You may modify the program and replace it with your own build: follow docs/dune/COMPILAR.md. Nothing in the
program or its updater prevents you from running a modified build (the update feed's signature check only
decides which updates THIS build installs; a build you compile yourself can carry your own key in
launcher/dune-update-key.pub).

The Dune Client logo, icons and background are not part of the licensed source (brand/LICENSE-BRAND.txt):
this archive contains neutral placeholders in their place. The mod (DuneClientMod), the store and the
plugins are separate programs, are not part of this program and are not covered by this offer.
OFFER
cp "$S/OFERTA-LGPL.txt" "$OUT/OFERTA-LGPL-$VER.txt"
tar -C "$WORK" -czf "$OUT/$NAME.tar.gz" "$NAME"
( cd "$OUT" && sha256sum "$NAME.tar.gz" "OFERTA-LGPL-$VER.txt" > "sha256sums-lgpl-$VER.txt" )
ls -l "$OUT"
