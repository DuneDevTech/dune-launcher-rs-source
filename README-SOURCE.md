# dune-launcher-rs-source

Corresponding source of the native Dune Client launcher engine (`dune-launcher`), a modified version of the
[Horizon](https://github.com/horizon-rust/horizon) launcher, licensed under **LGPL-3.0-or-later** (see `LICENSE`,
`LICENSES/`, `NOTICE-DUNE.md` for our changes, `OFERTA-LGPL.txt` for the written offer, `THIRD_PARTY_NOTICES.md`).

Current version: **0.3.2** (shipped inside Dune Client 1.2.0). Mirror of the release tarball:
https://duneclient.com/source/

## Build
Rust >= 1.95. Heavy: first release build takes ~40 min at `-j2`.

    cd launcher && cargo build --release --locked --bin dune-launcher     # Linux: target/release/dune-launcher
    scripts/build-windows.sh                                             # Windows x64 via Docker (needs d3dcompiler_47.dll, see docs/dune/COMPILAR.md)

Full instructions: `docs/dune/COMPILAR.md`. You may replace the binary with your own build; a self-built binary can carry
its own update key in `launcher/dune-update-key.pub` (only the public key is in this repository).

## Not included
The Dune Client logo, icons and background (brand art, not LGPL) are replaced by neutral placeholders with the same file names.
The Electron launcher, DuneClientMod, the store and plugins are separate programs and are not part of this source.
