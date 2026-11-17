# Evaluación de Horizon como base de un launcher nativo Rust para Dune Client

Fecha: 2026-11-16. Upstream: https://github.com/horizon-rust/horizon (clon en `upstream/`, commit `85b5bcca7c`, 173 commits, 11+ colaboradores, decenas de merges por semana). Fork de trabajo: `dune-launcher-rs/` → repo **privado** `DuneDevTech/dune-launcher-rs`.

## 1. Qué es Horizon (verificado leyendo el código)

- El producto principal es **una reimplementación de Minecraft Java 26.3 en Rust** (motor propio, "sin Java"). **No sirve para Dune Client**: nuestro mod es un mod Fabric de Java; necesita la JVM y el juego vanilla.
- Pero el repo incluye `launcher/` (crate `horizon-launcher` 0.2.0, ~140 000 líneas, ventana GPUI/wgpu, **LGPL-3.0-or-later**) que además de lanzar el motor Horizon sabe **gestionar instancias de Minecraft: Java Edition de Mojang**. Lo comprobé en el código y en ejecución:
  - `src/mojang.rs`, `minecraft_versions.rs`: manifiesto de versiones, client jar, librerías, assets y logging de Mojang (piston-meta, piston-data, libraries.minecraft.net, resources.download.minecraft.net) con tamaño + sha1.
  - `src/java.rs`: descarga el runtime Java de Mojang (8/16/17/21/25) o usa uno instalado.
  - `src/loaders.rs`: **Fabric**, Quilt, Forge y NeoForge (perfil de meta.fabricmc.net, librerías con sha1). `src/java_launch.rs`: arma el comando JVM como el launcher oficial (reglas, natives, args).
  - `src/account.rs` + `account/session.rs`: login Microsoft por **device code** → Xbox Live → XSTS → Minecraft Services, comprueba que la cuenta posee el juego. El **client id Azure es configuración** (`HORIZON_MSA_CLIENT_ID` en ejecución o en compilación), así que se puede usar el nuestro `87f38142-0d4d-41cb-a3b2-11091a0bc497`.
  - CLI: `horizon-launcher instance create NOMBRE --minecraft 1.21.11 --loader fabric` y `play --instance …`.
- Mods: pestaña Modrinth (Fabric/NeoForge/Forge se "portan" al motor Horizon; en instancias Java se instalan como jars), logs, capturas, perfiles, Java por instancia, memoria, args JVM, wrapper/gamemode.
- Gobierno: GOVERNANCE.md (consejo, temas, agentes de IA que triagean/revisan). Es proyecto activo (cientos de PR) pero joven y con mucho churn: el repo se mueve rápido y el fundador cede el mando según el documento.

## 2. Qué es reutilizable y qué no

| Parte | Veredicto |
|---|---|
| Login MS/Xbox/Minecraft, comprobación de propiedad, refresh, almacén cifrado de secretos | Reutilizable tal cual (solo client id propio) |
| Descarga Mojang + Java runtime + instalación Fabric + comando de lanzamiento | Reutilizable tal cual; funciona (smoke test §6) |
| Instancias, ajustes Java/memoria, logs, capturas, UI GPUI (wgpu, Wayland/X11/Windows/macOS) | Reutilizable; la UI está construida alrededor del motor Horizon |
| Motor del juego Rust (`crates/horizon-*` salvo `horizon-theme`, `addresses`, `telemetry`, `platform`…) | **No lo usamos**, pero el launcher lo enlaza como dependencia (path crates) → compilación larga, repo grande (271 MB) |
| Páginas de versiones Horizon, mods nativos, servidor dedicado, amigos/invitaciones, import de modpacks, MCP/agents, benchmark | Sobran para Dune; hay que ocultarlas (fase 2) |
| Servicios del upstream (updater GitHub `horizon-rust`, `telemetry.playhorizon.dev`, registro de mods, servidor oficial `play.playhorizon.dev`, **banner publicitario de Sculk** metido en el binario, `discord.gg/playhorizon`) | **Peligrosos**: hay que cortarlos antes de distribuir (ver §5) |

Coste/riesgo técnico: primera compilación release ≈ 35–40 min a `-j2` en este host (≈2 min las siguientes del crate launcher), binario 92 MB sin strip, 1.1 GB de target de deps. Requiere Rust ≥ 1.95 (instalado stable 1.100 vía rustup en `~/.cargo`), y en Linux libs de ventana: aquí faltaban los `-dev` (sin sudo) → `RUST_FONTCONFIG_DLOPEN=on` y `LIBRARY_PATH` con symlinks `libxkbcommon.so`/`libxkbcommon-x11.so`. Windows: gpui apunta a `x86_64-pc-windows-msvc` (DirectX); **no hay toolchain de cross-compile en este host** (sin cargo-xwin/clang-cl/mingw) → no compilado; lo razonable es compilar en un runner Windows (GitHub Actions `windows-latest`) o con `cargo-xwin` en Docker.

## 3. Licencia: obligaciones en lenguaje llano

Horizon: **LGPL-3.0-or-later** para el trabajo original (`LICENSE`), código fusionado antes del cambio sigue disponible bajo **MIT** (`LICENSES/MIT.txt`); dependencias con sus licencias (`THIRD_PARTY_NOTICES.md`; p. ej. GPUI/gpui-fast Apache-2.0, cap-std MIT/Apache). Nota: el propio proyecto retiró código por conflictos de licencia (Iris LGPL, OpenJDK GPL) en el motor — no afecta al launcher, pero indica que conviene auditar con `cargo license`/`cargo deny` antes de distribuir.

**Podemos**: usar, modificar y **redistribuir, también comercialmente**, el launcher y nuestro fork.

**Debemos** (si distribuimos binarios a terceros):
1. Dar a cada receptor el **código fuente completo correspondiente** de la parte LGPL que distribuimos (nuestro fork modificado de Horizon: este repo, commit exacto del binario) bajo LGPL-3.0 — publicarlo o ofrecerlo por escrito/enlace con las instrucciones de compilación.
2. Permitir **relinkar/reemplazar** (con las fuentes y cómo compilar basta; no hay cifrado/firmado que lo impida mientras el usuario pueda ejecutar su build; ojo si algún día se meten DRM/anti-tamper).
3. Incluir el texto de licencia (`LICENSE`, `LICENSES/`) y avisos (`THIRD_PARTY_NOTICES.md`) en el paquete y en una pantalla "Acerca de".
4. **Indicar los cambios** (`NOTICE-DUNE.md`, ya creado) y mantener todos los avisos de copyright upstream.
5. No restringir los derechos que da la LGPL (no EULA que prohíba reversing de la parte LGPL ni redistribución).

**No nos conceden**: marca/nombre/logos "Horizon" (re-marca obligatoria: hecho en texto, icono y paleta), ni los logos de Microsoft/Fabric/Minecraft más allá de su propia licencia (el botón "Sign in with Microsoft" sigue las pautas de Microsoft, ver `THIRD_PARTY_NOTICES.md`).

**Se mantiene privado**: DuneClientMod, tienda, plugins, licensing server → son programas separados que se comunican por HTTP/descarga de jar/propiedad JVM (`-Ddune.mode=dune`), no enlazados al launcher; no son "obra derivada" y no heredan la LGPL. Cuidado: no copiar código LGPL del launcher dentro del mod o del servidor.

**Mientras el repo sea privado no hay obligación** (no hay distribución). La obligación nace con el **primer binario entregado a un tercero** (incluidos beta testers): antes hay que publicar/ofrecer el código fuente (p. ej. repo público `DuneDevTech/dune-launcher-rs` o tarball con el tag). Eso no obliga a publicar nada más de Dune.

(Esto es una explicación práctica, no asesoría legal; si hay dinero de por medio conviene que lo revise un abogado.)

## 4. Resultado: ¿se puede? **Sí, con adaptación razonable** (hecho)

Fork creado y funcionando: ver `dune-launcher-rs/` (remote `origin` = DuneDevTech/dune-launcher-rs privado; remote `upstream` = horizon-rust/horizon para traer parches; `upstream-local` = el clon de evaluación).

Cambios hechos (detalle en `NOTICE-DUNE.md`):
- `launcher/src/dune.rs` (nuevo): tabla de builds de DuneClientMod (1.21.11, 1.21.4, 1.21.1, 1.20.1, mismos sha512 que `versions.ts`; descarga de `https://duneclient.com/downloads/client/DuneClientMod-1.2.0-mc<v>.jar` con verificación), Fabric API + MaLiLib + Litematica desde Modrinth con sha512 (igual que `THIRD_PARTY_MODS` de DuneClient), sesión offline solo-QA.
- `instance create … --dune` + `-Ddune.mode=dune`; Fabric Loader 0.20.1 (el mismo pin).
- Corrección de un bug del upstream con instancias Java (`run_leases`: pasaba la línea de comandos de `java` por el parser del motor Horizon → "missing value after 720").
- Marca: texto "Horizon"→"Dune Client" en la UI (en `i18n`, sin tocar las claves de traducción para poder fusionar upstream), título/app id, acento Oro Dune #E3A857 y fondo Noche #15161A, iconos Dune.
- Seguridad/privacidad: sin auto‑update desde `horizon-rust`, sin endpoint de telemetría por defecto, banner de patrocinador (Sculk) oculto siempre.
- Client id propio: `HORIZON_MSA_CLIENT_ID=87f38142-… cargo build` (o variable en ejecución). Aún pendiente la aprobación de Mojang, igual que en Electron.

## 5. Pendiente antes de distribuir (plan concreto)

Bloqueantes:
1. **Aprobación Mojang** del app id (misma puerta que DuneClient; exige cuenta MS + comprobación de propiedad, ya implementada aquí; mantener `dev-offline` fuera de las builds publicadas).
2. **Quitar el motor/UI Horizon** de la interfaz: instancia por defecto "Horizon", páginas Versions/servidores/amigos/mods nativos, texto de "Horizon" en `--help` del CLI y strings no pasadas por `tr()`, enlaces a playhorizon.dev/Discord/GitHub, "servidor oficial", pantalla de bienvenida y consentimiento de datos (hoy mencionan "I'm playing ping"; ya no envía nada porque no hay endpoint, pero el texto debe salir o apuntar a nuestra política).
3. Pantalla de inicio propia: crear al primer arranque las 4 instancias Dune (`scripts/dune-qa-instances.sh` es el esqueleto) y selector de versión tipo DuneClient; Play = JUGAR con la cuenta MS.
4. Feed propio: reemplazar `launcher_update` (hoy `None`) por un updater contra `duneclient.com/downloads/launcher-rs/` con firma (hay `horizon-release-sign` Ed25519 reutilizable); y que la tabla `BUILDS` salga de un JSON servido (`/downloads/client/versions.json`; el modelo es el `versions` de feed.json de DuneClient, hoy `feed.json` aún responde 404).
5. Empaquetado: Linux AppImage/deb (hay `packaging/linux`), Windows MSVC build en runner Windows + instalador (hay `packaging/windows`), macOS pendiente; firma de código.
6. **Cumplimiento LGPL**: publicar el fuente antes del primer binario (§3), pantalla "Licencias", auditoría `cargo deny` de dependencias.
7. Auditar upstream en cada fusión (`git fetch upstream`): el churn es alto; fusionar `launcher/` y `crates/` selectivamente. Mantener los cambios nuestros aislados (por eso `dune.rs` + parches pequeños).

Alternativas si el mantenimiento resulta caro: (a) seguir con el launcher Electron de DuneClient (ya completo) y usar este solo como "modo ligero"; (b) un launcher Rust mínimo propio (reqwest + serde + `portablemc`-style, ~3-5 kLOC) reutilizando solo las ideas de `mojang.rs/java.rs/loaders.rs/account` — evitaría 140 kLOC y el motor, pero hay que reescribir la UI.

## 6. Prueba (smoke test, evidencia en `shots/`)

Entorno aislado `qa-home/` (HORIZON_LAUNCHER_HOME/HORIZON_HOME propios), build release `--features dev-offline`, `DUNE_DEV_OFFLINE=1`:
- `horizon-launcher instance create "Dune 1.21.11" --minecraft 1.21.11 --loader fabric --dune` → instancia Java + Fabric 0.20.1.
- `play --instance dune-1-21-11`: descargó cliente/librerías/assets de Mojang (4593 ficheros) y usó el Java 21 del sistema (Corretto; la descarga del runtime de Mojang existe en `java::install` pero **no se ejercitó** porque `java::pick` encontró uno instalado), instaló Fabric, Fabric API 0.141.6, MaLiLib, Litematica y `DuneClientMod.jar` (sha512 verificado) y arrancó Minecraft 1.21.11 bajo su propio Xvfb :187 (GL llvmpipe). Ventana "Dune Client 1.21.11" (la rebrand de la mod vía `-Ddune.mode=dune`).
- `shots/02-minecraft-1.21.11-title-fabric-duneclientmod.png`: pantalla de título del mod Dune (nota: el arte/promos de esa pantalla siguen siendo "Nebula", es contenido del **mod**, no del launcher).
- `shots/01-launcher-home-datos-modal.png`: ventana del launcher Rust (render GPUI/wgpu con Vulkan lavapipe sobre un sway headless + `grim`, porque Xvfb no tiene DRI3 y la ventana salía negra) — con título "Dune Client", paleta dorada y Home de la instancia "Dune 1.21.11" detrás del modal de consentimiento de datos. No pude cerrar el modal con clics sintéticos (gpui no recibió los eventos de `swaymsg seat cursor`), por eso no hay captura más limpia.
- No tocados: Xvfb :55, Chrome CDP 9333, el árbol de NebulaLauncher/DuneClient. Procesos propios (Xvfb :187, sway, launcher, JVM) cerrados.
- No ejecutado: `cargo test` (compilación de test heavy; los tests de `dune.rs` están escritos pero sin correr), build Windows/macOS, login Microsoft real (client id sin aprobar), servidor.
