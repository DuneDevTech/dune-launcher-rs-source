# Pruebas y CI

## Que corre la CI (`.github/workflows/ci.yml`, runner self-hosted del host Dune)
`cargo test -j2` del launcher, clippy (no bloquea aun), cargo-deny (licencias/bans/fuentes bloquean, avisos no), gitleaks.

Se ejecuta en cada PR y en cada push a `dev`/`main`/`master`. No lleva secretos de produccion.

## En local
```sh
cd launcher
cargo test --locked --lib --bins -j2
cargo clippy --locked --all-targets -j2
```

## Politica de pruebas
- **TDD** para la logica de `launcher/src` y los crates que compila: prueba primero, ver el fallo (rojo), codigo minimo (verde), refactor.
- **Bug corregido = test de regresion** en el mismo PR (debe fallar sin el arreglo).
- **UI**: tests con captura de pantalla para las pantallas GPUI; captura de pantalla en el PR.
- Los tests no tocan servicios reales ni datos de produccion: usa BD temporales / fixtures.

## Politica de merge (convencion, sin enforcement)
El plan gratuito de la org no permite branch protection en repos privados. Se aplica por convencion:
sin push directo, PR + CI en verde + review. Ver `CONTRIBUTING.md`.
