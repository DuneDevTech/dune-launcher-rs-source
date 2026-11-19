# Compilar el launcher nativo (Dune Client)

Requisitos: Rust ≥ 1.95 (se usó 1.100 vía rustup). Host CPU limitado: **una compilación pesada a la
vez**, `nice -n 10`, `-j2`. La primera compilación release tarda 35–40 min a `-j2`; las siguientes ~3 min.

## Linux
```bash
sudo apt install libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev libfontconfig-dev libvulkan-dev libasound2-dev patchelf
cd launcher && cargo build --release --locked          # target/release/dune-launcher
scripts/build-linux.sh                                  # binario sin símbolos, .tar.gz y AppImage en dist/
```
Sin los `-dev` (sin sudo): `RUST_FONTCONFIG_DLOPEN=on` y `LIBRARY_PATH` apuntando a symlinks
`libxkbcommon.so` / `libxkbcommon-x11.so` (ver `../libshim`).

## Windows (cross-compilación en Docker, sin Windows)
```bash
scripts/build-windows.sh     # docker --cpus=2, target x86_64-pc-windows-gnu (MinGW)
```
Produce `dist/windows/dune-launcher.exe`, `dist/DuneClient-<ver>-windows-x64.zip` (portable: es lo que
usa el autoupdate) y `dist/DuneClient-Setup-<ver>.exe` (el mismo binario con nombre de instalador:
al abrirlo sin argumentos muestra el instalador por usuario de `launcher/src/installer.rs`, sin
administrador; `... setup --quiet [--dir X] [--desktop-shortcut]`, `dune-launcher.exe uninstall`).
Los shaders HLSL de `gpui-fast-windows` se compilan con `d3dcompiler_47.dll` de Microsoft bajo wine dentro de la
imagen (`docker/fxc-wine`, `GPUI_FXC_PATH`; la DLL es una herramienta de compilación, no se distribuye ni está en el
repo: `D3DCOMPILER_DLL=...`, por defecto `dune-horizon/tools/`). Upstream solo compila shaders en un host Windows, por eso
`third_party/gpui-fast-windows` parchea únicamente su `build.rs`. Prueba en wine (headless): con una `icuuc.dll` mínima
(wine no trae la de Windows 10+) `dune-launcher.exe --version` imprime `dune-launcher 0.3.0`; el render DirectX no se ha
visto en una GPU Windows real.
Sin firmar: Windows SmartScreen mostrará "Editor desconocido" (ver el launcher Electron:
`docs/TIENDA-Y-FIRMA.md`). No se pudo ejecutar en Windows/Wine aquí: ver `BUILD-REPORT.md`.

## Pruebas
`cd launcher && cargo test -j2` (lib + bin + integración). Dos pruebas de upstream dependen del
entorno (`data_location::…stale_worker…` falla a veces en paralelo por reutilización de PID; pasa sola).

## QA con pantalla (sin tocar ningún Xvfb/Chrome de otros)
`dune-horizon/shoot.sh NOMBRE [PAGINA] [SEGUNDOS]` abre un sway headless propio
(`XDG_RUNTIME_DIR=/tmp/dune-xdg`), arranca el launcher con `HORIZON_LAUNCHER_PAGE` y captura con grim.
Variables de QA: `HORIZON_LAUNCHER_HOME`, `HORIZON_HOME`, `HORIZON_NO_KEYCHAIN=1` (`DUNE_DEV_OFFLINE` y `dev-offline` ya no existen).
