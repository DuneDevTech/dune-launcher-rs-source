# Dune Client modifications (LGPL-3.0 §5 / §4 notice)

This is a modified version of the launcher of **Horizon** (https://github.com/horizon-rust/horizon),
Copyright (c) 2026 Ludovic Marie and the Horizon contributors, licensed under the GNU LGPL
version 3 or later (code merged earlier: MIT, `LICENSES/MIT.txt`). All upstream copyright and
licence notices are kept unchanged (`LICENSE`, `LICENSES/`, file headers). Modified by Dune Dev Tech
(Dune Client) in November 2026. Base commit: `85b5bcca7c` of upstream (`git log` has every change).

Dune Client is not affiliated with, endorsed by or sponsored by the Horizon project, Mojang or
Microsoft. "Horizon" is not used as a name or logo of this product.

## What changed

New code (written for Dune Client, part of the LGPL-covered launcher):

- `launcher/src/dune.rs`: Dune Client mod pins (sha512), download of the mod and of Fabric API /
  MaLiLib / Litematica / Mod Menu (Modrinth, sha512 checked), the three play modes
  (Dune / Dune + Fabric / Vanilla), the guard that keeps the launcher from contacting the Horizon
  project's services, the licence screen text, and the dev-only offline session.
- `launcher/src/dune_update.rs`, `scripts/dune-sign-update.py`, `launcher/dune-update-key.pub`: our own
  signed update feed (Ed25519 + SHA-512) in place of GitHub releases (docs/dune/ACTUALIZACIONES.md).
- `launcher/scenery/src/picture.rs`: the Dune Client background picture behind the glass UI.
- `scripts/third-party-audit.py`, `scripts/build-linux.sh`, `scripts/build-windows.sh`,
  `scripts/make-source-pack.sh`, `docker/windows-gnu.Dockerfile`, `docs/dune/*`.

Changed upstream files (summary; the diff against upstream is the complete list):

- Java only: the Horizon-engine pages (Versions, Servers, Wardrobe, Shop), the Horizon mod source,
  the invite-code/friends field, the edition switch, the other themes, the AI/agents and data
  collection settings and the Game (engine) settings are not reachable; the default instance plays
  Dune 1.21.11; new instances are Dune / Dune + Fabric / Vanilla on 1.21.11, 1.21.4, 1.21.1 or 1.20.1
  (`instances.rs`, `new_instance.rs`, `glass_instance*.rs`, `glass_sheet.rs`, `glass_settings.rs`,
  `minecraft_versions.rs`, `main.rs`, `frame.rs`, `glass_home.rs`, `mods_page.rs`, `themes.rs`).
- `backend.rs`: `java_game` installs the Dune mods per mode and passes `-Ddune.mode=`; no GitHub
  release list; the launcher update comes from the Dune feed; no links.json fetch; requests to the
  Horizon project's hosts are refused (`dune::upstream_blocked`); data folders `dune-launcher` and
  `dune-client`; default language Spanish (English available).
- `account.rs`: Dune Client's Azure client ID, Spanish message while Mojang's approval is pending.
- `run_leases.rs`: Java commands bypass the Horizon-engine argument parser (upstream bug).
- `i18n.rs`, `lang/es.json`: UI text says "Dune Client" where upstream says "Horizon"; Spanish
  completed for the visible UI.
- `partner.rs` and `assets/sculk-*`: the sponsor advert and its pictures are removed.
- `installer.rs`, `setup_*`, `desktop.rs`, `cli.rs`, `build.rs`, `Cargo.toml`: executable
  `dune-launcher`, install folder "Dune Client", menu entry, CLI help, icon embedding when
  cross-compiling.
- `crates/horizon-theme`: Inter + Montserrat fonts (OFL), Dune icon; `crates/horizon-telemetry`: no
  built-in upstream endpoint; `crates/horizon-addresses`: about URL.
- `packaging/`: Dune icons and AppImage / desktop entry; `brand/`: Dune Client logos (not LGPL:
  `brand/LICENSE-BRAND.txt`).
- `launcher/Cargo.toml`: the `dev-offline` feature was removed (0.5.3); no account-free play exists. `handoff` re-verifies ownership with the Minecraft services (`account::verify_handed_off`).

## Antes de distribuir (obligaciones LGPL-3.0, resumen)

1. Entregar a los destinatarios el **código fuente completo correspondiente** del commit exacto del
   binario bajo LGPL-3.0, con instrucciones de compilación (`scripts/make-source-pack.sh` genera el
   paquete y el texto de la oferta; **no se publica nada sin la aprobación del propietario**).
2. Incluir `LICENSE`, `LICENSES/`, `THIRD_PARTY_NOTICES.md` y este fichero en el paquete (hecho: el
   launcher los muestra en Ajustes → Acerca de → Licencias).
3. No usar el nombre/logos "Horizon" como marca ni sugerir que Dune Client es de Horizon.
4. DuneClientMod, tienda y plugins son programas separados (HTTP/descarga): no se publican.
