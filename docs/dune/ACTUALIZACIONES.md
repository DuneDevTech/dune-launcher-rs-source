# Actualizaciones del launcher (feed propio firmado)

El launcher nativo **no** consulta nunca GitHub ni los servicios de Horizon. Su única fuente de
actualizaciones es:

    https://duneclient.com/downloads/launcher-rs/latest.json      (DUNE_UPDATE_FEED lo sustituye, solo QA)

## Formato

```json
{
  "version": "0.3.0",
  "platforms": {
    "linux-x86_64":     { "url": "https://duneclient.com/downloads/launcher-rs/DuneClient-0.3.0-x86_64.AppImage", "sha512": "<hex 128>", "sig": "<base64>" },
    "linux-x86_64-tar": { "url": ".../DuneClient-0.3.0-linux-x86_64.tar.gz", "sha512": "...", "sig": "..." },
    "windows-x86_64":   { "url": ".../DuneClient-0.3.0-windows-x64.zip",     "sha512": "...", "sig": "..." }
  }
}
```

- `linux-x86_64` es el AppImage (lo lee un launcher que corre desde AppImage); `linux-x86_64-tar` es
  un `.tar.gz` que contiene el binario `dune-launcher` (instalación suelta); `windows-x86_64` es un
  `.zip` que contiene `dune-launcher.exe`. Las URL deben ser `https://`.
- `sig` = Ed25519 (base64) sobre el mensaje exacto
  `"dune-launcher-rs update\n<version>\n<plataforma>\n<sha512 hex>\n<url>"`
  (`launcher/src/dune_update.rs::message`). Firmar versión, plataforma, hash y URL impide que un feed
  manipulado cambie el fichero, sirva otra plataforma o reutilice una firma antigua.
- La clave **pública** está compilada en el launcher (`launcher/dune-update-key.pub`). La **privada**
  vive solo en la máquina de publicación: `~/.config/dune/launcher-rs-signing.key` (PEM PKCS#8, modo
  0600, nunca en el repositorio ni en logs). Si se pierde o se filtra hay que generar otra, publicar un
  launcher con la clave nueva por un canal manual y cambiar `dune-update-key.pub` (el fichero admite
  varias claves, una por línea, para una rotación sin corte).

## Publicar una versión (manual; no se publica nada solo)

```bash
scripts/build-linux.sh          # AppImage + tar.gz
scripts/build-windows.sh        # zip portable + DuneClient-Setup
scripts/dune-sign-update.py 0.3.0 \
  linux-x86_64=https://duneclient.com/downloads/launcher-rs/DuneClient-0.3.0-x86_64.AppImage=dist/DuneClient-0.3.0-x86_64.AppImage \
  linux-x86_64-tar=https://duneclient.com/downloads/launcher-rs/DuneClient-0.3.0-linux-x86_64.tar.gz=dist/DuneClient-0.3.0-linux-x86_64.tar.gz \
  windows-x86_64=https://duneclient.com/downloads/launcher-rs/DuneClient-0.3.0-windows-x64.zip=dist/DuneClient-0.3.0-windows-x64.zip \
  > latest.json
```

Subir primero los binarios y **al final** `latest.json` (como `latest.yml` del launcher Electron).

## Cómo lo usa el launcher

1. Lee el feed (HTTPS), comprueba la firma de la entrada de su plataforma con la clave compilada y
   que `version` sea mayor que la propia. Un feed ilegible, sin firma válida o sin su plataforma no
   ofrece nada (no hay errores visibles).
2. Descarga a una carpeta temporal y solo conserva el fichero si su SHA-512 es el firmado; guarda el
   SHA-256 para revisarlo otra vez justo antes de instalar (`place_checked`).
3. Sustituye el AppImage / binario (en Windows, mediante el relanzador de `update_handoff`).

Se desactiva con `DUNE_NO_SELF_UPDATE=1`; las compilaciones de desarrollo (debug) nunca se
autoactualizan.
