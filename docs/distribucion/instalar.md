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

     Instalar otra vez no repite el bloque, y reinstalar en otro directorio lo
     sustituye por uno con el directorio nuevo. `tc2 desinstalar` (F2-14) lo
     quita por sus marcas. Con fish, el instalador dice qué ejecutar. Como el
     directorio se escribe en el perfil, se rechaza si lleva comillas, `$`,
     `` ` ``, `\`, `:` o un salto de línea.

   En Windows, reinstalar en otro directorio añade el nuevo al final del PATH
   del usuario, pero no quita el anterior: si el `tc2.exe` viejo sigue ahí,
   gana él. Hasta `tc2 desinstalar` (F2-14), quita a mano la entrada antigua.
   Un directorio con `;` (el separador del PATH) se rechaza.

Volver a ejecutar la línea reinstala (o cambia de versión con `TC2_VERSION`).
Para actualizar sin reinstalar está `tc2 actualizar --cli` (abajo). No toca
`taskctl` ni nada de TaskCode 0.16.

## Actualizar y comprobar la versión

Lo construyó TASK-027 (F2-12; ADR F1-14, puntos 4 y 5).

```bash
tc2 actualizar --cli --comprobar   # ¿hay una versión nueva? No toca nada
tc2 actualizar --cli               # sustituye el tc2 instalado por la última estable
tc2 actualizar --cli --version 2.1.0-rc.1   # una versión concreta (también hacia atrás)
```

- **De dónde sale la versión disponible**: del `SHA256SUMS` de la última
  release estable (`releases/latest/download/SHA256SUMS`), o de la de
  `--version`. Una sola petición, la misma fuente que los instaladores.
- **`--comprobar`** escribe `actualizar ok actual=X disponible=Y` si hay una
  más nueva y `actualizar sin-cambios actual=X disponible=Y` si no; código 0
  en los dos casos. Es el aviso de versión nueva: lo llaman `doctor` y, si se
  quiere, un hook de inicio de sesión. Nunca se consulta la red por sí solo.
- **Cómo sustituye**: baja el binario junto al actual
  (`.tc2-<versión>-<pid>.descarga`), calcula su sha256 (lo calcula el `tc2`
  viejo, nunca el binario nuevo sobre sí mismo) y lo compara con
  `SHA256SUMS`; le pide `version --json` y comprueba versión, sistema y hash;
  y solo entonces lo pone en su sitio. En Windows lo hace por renombrados
  (el actual pasa a `tc2.exe.old`, el nuevo a `tc2.exe`) con vuelta atrás si
  el segundo falla; el `tc2.exe.old` lo borra la siguiente ejecución de `tc2`.
  Sale `actualizar ok anterior=X version=Y sha256=...`.
- **Si algo falla, no se cambia nada** y no quedan ficheros a medias:
  - sin red: `actualizar sinred url=...` (código 4);
  - el sha256 no casa: `rechazada motivo=sha256-no-casa` (3);
  - sin permiso de escritura en la carpeta del binario: `rechazada
    motivo=sin-permiso ruta=...` (3), antes de bajar nada;
  - el binario nuevo no arranca o no dice lo esperado: `rechazada
    motivo=binario-no-arranca` (3);
  - no se pudo renombrar (otro `tc2` abierto, un antivirus):
    `error motivo=no-se-sustituye` (1), con el binario anterior en su sitio.
- **Lo que no actualiza**: el `tc2` de la caché del plugin
  (`.../taskcodetwo/cli/<versión>/`), que gestiona el plugin con su
  `cli.lock` (`rechazada motivo=gestionado-por-plugin`: se actualiza el
  plugin), ni uno instalado con un gestor de paquetes (npm, Homebrew, winget,
  Scoop, cargo: `rechazada motivo=gestionado-por-gestor`).
- Sin `--cli`, `tc2 actualizar` da `uso`: actualizar un proyecto es de F9-2.
- `TC2_BASE_URL` vale igual que en los instaladores (solo ensayos): la carpeta
  con los ficheros de una release; aquí, https o http a `127.0.0.1`.
- Como en los instaladores, el hash prueba que el fichero llegó entero, no
  quién lo construyó: sin Authenticode (D4a), comprometer la release
  compromete binario y `SHA256SUMS` a la vez. La attestation se comprueba con
  `gh attestation verify` (ver [releases.md](releases.md)).

## Compatibilidad con el plugin

El plugin pregunta al `tc2` que encuentra (ADR F1-14, punto 5):

```bash
tc2 version --json
# {"version":"2.0.0","protocolo":1,"target":"win-x64","sha256":"<64 hex>"}
tc2 compatible --protocolo 1-1
# compatible ok protocolo=1 rango=1-1
```

Si el protocolo no cae en el rango, `compatible` sale con `rechazada` (3),
`motivo=protocolo-incompatible` y el arreglo `tc2 actualizar --cli`. El
`cli.lock` del plugin lo genera `cargo xtask cli-lock <SHA256SUMS> [--tag
<tag>]` desde el `SHA256SUMS` de la release, nunca desde un build local.

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
  `tc2` y escribe `tc2 <version>`, en menos de 60 s. En Windows, el primer
  arranque de `powershell.exe` en un runner recién creado (unos 12 s en
  `windows-11-arm`) queda fuera: la persona escribe la línea en una ventana
  ya abierta. Se mide aparte y queda en el log. Sí entra en la medida lo que
  tarda PowerShell en un proceso nuevo en encontrar `irm`: en
  `windows-11-arm` son 42-50 s (en `windows-latest`, 0-1 s), con lo que la
  instalación allí mide unos 44 s, aunque el instalador en sí tarda menos de
  1 s. El log lo deja aparte para verlo;
- con un `SHA256SUMS` alterado, sale con error y no queda `tc2`;
- dos instalaciones dejan una sola entrada en el PATH, y
  `TC2_NO_MODIFY_PATH=1` no lo toca.

Al publicar, `verificar-instaladores` ejecuta la línea real contra la release
en los mismos sistemas, con la misma medida, y comprueba la attestation donde
hay `gh`.

`tc2 actualizar --cli` se prueba en dos sitios:

- `cargo test -p tc2` (`crates/tc2/tests/actualizar_cli.rs`): una copia del
  `tc2` compilado se actualiza a un doble de la versión 9.9.9
  (`examples/tc2_falso.rs`) servido en `127.0.0.1`; el hash alterado, sin red,
  `--comprobar` y la caché del plugin. El fallo del segundo renombrado se
  prueba con un fallo inyectado.
- el job `actualizar-ensayo` de `release.yml`
  (`.github/scripts/probar-actualizar.sh`): en los 6 runners nativos, instala
  el binario del build con la línea de instalación y lo actualiza a otro
  compilado con la versión siguiente, con `--comprobar` y el hash alterado.
  Hasta el primer push con el job, **no verificado** en CI (TASK-027); su
  paso en Windows x64 se ensayó en local.

Los runners no son máquinas limpias (traen curl, gh y Node). La prueba en
máquinas limpias de verdad es de F9-9a (TASK-071).
