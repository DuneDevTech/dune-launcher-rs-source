# Inicio de sesión con Microsoft

- Flujo **device code** (sin secreto): Microsoft → Xbox Live → XSTS → servicios de Minecraft, y se
  comprueba que la cuenta posee Java Edition (`entitlements/mcstore`, Game Pass incluido).
- **Client ID Azure propio**: `2f1eb485-c280-475c-9448-83fbeb91870c` (valor público, en
  `launcher/src/account.rs::DUNE_CLIENT_ID`; `DUNE_MSA_CLIENT_ID` lo sustituye en ejecución o en
  compilación). La app Azure está en el tenant de Nebula/Dune: "Solo cuentas personales", sin URI de
  redirección, "Allow public client flows" activado.
- **Estado**: Mojang aún no ha aprobado el ID (https://aka.ms/mce-reviewappid). Hasta entonces
  `login_with_xbox` responde 403 y el launcher muestra (en español):
  «Mojang todavía no ha aprobado Dune Client para iniciar sesión con Microsoft (la solicitud está
  pendiente). Tu cuenta y tu contraseña están bien: hasta que llegue la aprobación no se puede
  iniciar sesión ni jugar con cuenta Microsoft.» Es lo esperado, no un fallo.
- Requisitos del formulario de Mojang que ya cumple el launcher: no se puede jugar sin cuenta que
  posea el juego; no existe ningún modo sin cuenta (ni offline, ni de QA, ni feature de compilación); el `handoff`
  del launcher Electron vuelve a verificar token y propiedad con Minecraft services antes de arrancar el juego.
- Las credenciales se guardan en el llavero del sistema (servicio `com.duneclient.launcher`).
