# dune-launcher-rs — native Dune Client launcher (Rust)

> **PRIVATE until the first binary is distributed.** This repository is a fork of the launcher
> of [horizon-rust/horizon](https://github.com/horizon-rust/horizon) (LGPL-3.0-or-later; older
> code MIT). **Before distributing the first binary** to anyone outside Dune Dev Tech, the
> complete corresponding source of the launcher we ship (this fork) must be published or offered
> under the LGPL to the recipients (see `NOTICE-DUNE.md` → "Antes de distribuir"). Do not
> publish binaries from this repo without that step.

Dune Client's fast native launcher option, next to the Electron/TypeScript `DuneClient` launcher.
It uses Horizon's launcher code (GPUI window, Microsoft device-code login, Mojang downloads,
Java runtime download, Fabric install, instances) to play **vanilla Minecraft: Java Edition with
Fabric + the Dune Client mod** — it does *not* use Horizon's Rust reimplementation of the game.

## What Dune Client adds on top of upstream

See `NOTICE-DUNE.md` for the full change list. In short: `launcher/src/dune.rs` (pinned
DuneClientMod builds from `https://duneclient.com/downloads/client/`, sha512 verified; Fabric
API/MaLiLib/Litematica from Modrinth, sha512 verified), `instance create --dune`, the
`-Ddune.mode=dune` JVM property, Dune palette (Oro Dune accent, Noche background), no upstream
self-update, no built-in telemetry endpoint, no third-party sponsor banner, rebranded UI text.

## Build (Linux)

    rustup toolchain install stable          # rust-version >= 1.95
    cd launcher
    # Debian/Ubuntu without -dev packages: RUST_FONTCONFIG_DLOPEN=on and a LIBRARY_PATH dir
    # with libxkbcommon.so / libxkbcommon-x11.so symlinks (see EVALUACION.md).
    nice cargo build --release -j2           # add --features dev-offline for QA builds only

## Microsoft login

Set the Azure public client id with `HORIZON_MSA_CLIENT_ID` (run time) or at build time:
`HORIZON_MSA_CLIENT_ID=87f38142-0d4d-41cb-a3b2-11091a0bc497 cargo build --release`.
Mojang must approve the app id (same gate as the Electron launcher, docs/MICROSOFT-LOGIN.md there).

## Dev/QA offline session

Builds made with `--features dev-offline` (or debug builds) honour `DUNE_DEV_OFFLINE=1` (offline
name `DUNE_DEV_NAME`, default `DuneDev`, no Microsoft account). Published release builds are made
**without** the feature and ignore the variable.

## Upstream

    git remote -v        # upstream = https://github.com/horizon-rust/horizon.git
    git fetch upstream && git merge upstream/main   # or cherry-pick launcher/ and crates/ fixes

Upstream's original README is `README.upstream.md`. Licences: `LICENSE` (LGPL-3.0),
`LICENSES/` (MIT, GPL-3.0 text), `THIRD_PARTY_NOTICES.md` — all kept. "Horizon", its logos and
name are not licensed to us and are not used in the Dune Client user interface.
