# Instalar `tc2` con una línea

Lo construyó TASK-026 (F2-11). Aplica lo decidido en
[ADR F1-14](../adr/F1-14-distribucion-e-instalacion.md) (D4a y punto 6). Cómo
se publican y verifican las releases: [releases.md](releases.md).

## Las dos líneas

Windows (PowerShell 5.1 o 7, sin permisos de administrador):

```powershell
irm https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.ps1 | iex
```

macOS y Linux (cualquier `sh`, con `curl` o `wget`):

```bash
curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.sh | sh
```

`latest` es la última release **estable**: mientras no haya ninguna (antes de
la 2.0.0), el instalador lo dice y pide fijar la versión. Una versión concreta,
o una prerelease, se instala con el instalador de esa release y `TC2_VERSION`:

```powershell
$env:TC2_VERSION = '0.0.0-prueba.1'; irm https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-prueba.1/install.ps1 | iex
```

```bash
curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-prueba.1/install.sh | TC2_VERSION=0.0.0-prueba.1 sh
```

## Qué hace

1. Detecta el sistema y la arquitectura: uno de los 6 alias de P12. En macOS
   bajo Rosetta, y en una PowerShell x64 emulada en Windows ARM64, elige el
   binario nativo arm64.
2. Baja `SHA256SUMS` y el binario `tc2-<version>-<alias>[.exe]` de la release.
3. Exige que `SHA256SUMS` tenga exactamente una línea para ese fichero y que el
   sha256 case. Si no casa, sale con error y no instala nada.
4. Si hay `gh` con sesión, comprueba también la attestation de procedencia
   (`gh attestation verify --signer-workflow .../release.yml`). Sin `gh`, lo
   avisa en una línea y sigue: el hash solo prueba que el fichero llegó entero;
   que lo construyó este repositorio lo prueba la attestation.
5. Comprueba que el binario arranca (`tc2 --version`) antes de dejarlo en su
   sitio: nunca queda en el destino un binario sin comprobar.
6. Lo deja en el PATH:
   - **Windows**: `%LOCALAPPDATA%\taskcodetwo\bin\tc2.exe` y ese directorio al
     final del PATH del usuario (registro, sin tocar las `%VARIABLES%` que ya
     tenga), además de en la sesión actual.
   - **macOS y Linux**: `~/.local/bin/tc2`. Si ese directorio no está en el
     PATH, añade este bloque a `~/.profile` y, según `$SHELL`, a `~/.zshrc` o
     a `~/.bashrc` (y a `~/.bash_profile` si existe):

     ```sh
     # >>> tc2 (TaskCodeTwo) >>>
     case ":$PATH:" in *":/home/ana/.local/bin:"*) ;; *) export PATH="/home/ana/.local/bin:$PATH" ;; esac
     # <<< tc2 (TaskCodeTwo) <<<
     ```

     Instalar otra vez no repite el bloque. `tc2 desinstalar` (F2-14) lo quita
     por sus marcas. Con fish, el instalador dice qué ejecutar.

Volver a ejecutar la línea reinstala (o cambia de versión con `TC2_VERSION`).
Actualizar con `tc2 actualizar --cli` es de F2-12. No toca `taskctl` ni nada
de TaskCode 0.16.

## Variables

| variable | qué hace |
|---|---|
| `TC2_VERSION` | versión que se instala (`2.0.0`, `2.0.0-rc.1`...); por defecto, la última estable |
| `TC2_INSTALL_DIR` | directorio del binario |
| `TC2_NO_MODIFY_PATH=1` | no toca el PATH del usuario ni los perfiles del shell |
| `TC2_BASE_URL` | solo para ensayos: URL con los ficheros de la release (https, o http a `127.0.0.1` o `localhost`); exige `TC2_VERSION` y no comprueba la attestation |

Código de salida: 0 instalado; distinto de 0 con una línea `ERROR` y otra
`arreglo`. El script de PowerShell nunca usa `exit` (con `irm | iex` cerraría
la ventana): lanza el error.

## Cómo se prueba

En cada cambio, el job `instaladores-ensayo` de `release.yml` sirve en
`127.0.0.1` los binarios recién compilados y ejecuta
`.github/scripts/probar-instalador.sh local` en los 6 runners nativos
(Windows con `powershell.exe` 5.1) y en alpine (sin curl ni bash):

- instalación medida: desde la línea hasta que una terminal nueva encuentra
  `tc2` y escribe `tc2 <version>`, en menos de 60 s;
- con un `SHA256SUMS` alterado, sale con error y no queda `tc2`;
- dos instalaciones dejan una sola entrada en el PATH, y
  `TC2_NO_MODIFY_PATH=1` no lo toca.

Al publicar, `verificar-instaladores` ejecuta la línea real contra la release
en los mismos sistemas, con la misma medida, y comprueba la attestation donde
hay `gh`.

Los runners no son máquinas limpias (traen curl, gh y Node). La prueba en
máquinas limpias de verdad es de F9-9a (TASK-071).
