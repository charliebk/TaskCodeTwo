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
   gana él. `tc2 desinstalar --maquina` (abajo) quita la del directorio por defecto
   y, con `TC2_INSTALL_DIR` apuntando a otro, también la de ese y su binario.
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
  en los dos casos. Con `--version` de una anterior a la instalada añade
  `cambio=anterior`, para que no se lea como una versión nueva. Es el aviso
  de versión nueva y es una orden explícita: nunca se consulta la red por sí
  solo. La llamará `doctor` cuando exista (F3-8) y, si se quiere, un hook de
  inicio de sesión.
- **Cómo sustituye**: baja el binario junto al actual
  (`.tc2-<versión>-<pid>.descarga`), calcula su sha256 (lo calcula el `tc2`
  viejo, nunca el binario nuevo sobre sí mismo) y lo compara con
  `SHA256SUMS`; si hay `gh` con sesión, comprueba la attestation de
  procedencia (abajo); le pide `version --json` (como mucho 30 s) y comprueba
  versión, sistema y hash; y solo entonces lo pone en su sitio. En Windows lo hace por renombrados
  (el actual pasa a `tc2.exe.old`, el nuevo a `tc2.exe`) con vuelta atrás si
  el segundo falla; el `tc2.exe.old` lo borra la siguiente ejecución de `tc2`.
  Sale `actualizar ok anterior=X version=Y sha256=... attestation=...`.
  Si la descarga del binario se corta a mitad, se repite una vez; los demás
  fallos de red no se reintentan.
- **Si algo falla, no se cambia nada** y no quedan ficheros a medias:
  - sin red: `actualizar sinred url=...` (código 4);
  - el sha256 no casa: `rechazada motivo=sha256-no-casa` (3);
  - sin permiso de escritura en la carpeta del binario: `rechazada
    motivo=sin-permiso ruta=...` (3), antes de bajar nada;
  - otro fallo al escribir la descarga (disco lleno): `error
    motivo=escritura ruta=...` (1);
  - `gh` rechaza la attestation: `rechazada motivo=attestation-no-valida` (3);
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
- **Attestation**, como en los instaladores: el hash solo prueba que el
  fichero llegó entero. Si hay `gh` con sesión, `tc2 actualizar --cli` ejecuta
  `gh attestation verify <descarga> --repo charliebk/TaskCodeTwo
  --signer-workflow charliebk/TaskCodeTwo/.github/workflows/release.yml`
  antes de ejecutar nada del binario nuevo, y si no la da por buena no
  cambia nada. La línea dice cómo fue: `attestation=comprobada`,
  `attestation=sin-gh` (no hay `gh` o no tiene sesión: solo el sha256, como
  el instalador) o `attestation=ensayo` (con `TC2_BASE_URL`, cuyos binarios
  no tienen attestation). `TC2_GH` fija el `gh` que se usa; en un ensayo, solo
  se comprueba si `TC2_GH` está definida. Sin `gh` ni Authenticode (D4a),
  comprometer la release compromete binario y `SHA256SUMS` a la vez (ver
  [releases.md](releases.md)).

## Desinstalar

Lo construyó TASK-078 (F2-14, RR-16). Orden recomendado: `--proyecto` en
cada repo, luego `--maquina` y, al final, el plugin con la orden que imprime.

```bash
tc2 desinstalar --proyecto --simular   # el plan, sin tocar nada
tc2 desinstalar --proyecto             # worktrees, .git/taskcodetwo/, exclude
tc2 desinstalar --proyecto --config --borrar-tareas taskcodetwo/tareas
tc2 desinstalar --maquina --simular
tc2 desinstalar --maquina
```

- **Salida**: una línea por acción, `accion ruta=... clase=... dueno=...
  decision=borra|conserva|persona|pendiente [motivo=...] [orden=...]`, y al
  final `desinstalar <resultado> ambito=... borra=N conserva=N persona=N
  pendiente=N`. Con `--simular` la última lleva `simular=si` y no se toca
  nada. Una segunda ejecución da `sin-cambios`. Las órdenes que tiene que
  ejecutar la persona salen también, legibles, por stderr.
- **Qué quita `--maquina`** (solo lo que reconoce por su ruta, sus marcas o
  su `command` exactos): el binario de `TC2_INSTALL_DIR` y del directorio por
  defecto (si es este `tc2`, está en la carpeta propia de Windows o responde
  a `tc2 version --json`; otro `tc2` queda `dueno=ajeno`), su `tc2.exe.old` y los temporales `.tc2-*.descarga` y
  `.tc2.nuevo.*`; el bloque entre marcas de `~/.profile`, `~/.bashrc`,
  `~/.zshrc` y `~/.bash_profile` (el resto del perfil queda igual byte a
  byte, salvo el salto de línea final que `install.sh` añadió a un perfil que
  no lo tenía; si la línea entre las marcas no es exactamente la del
  instalador, no se toca y queda para la persona); en Windows, la entrada del PATH de usuario (la inversa de lo que
  añadió `install.ps1`; las demás entradas, intactas) y la carpeta
  `%LOCALAPPDATA%\taskcodetwo` si queda vacía; la caché del lanzador
  (`taskcodetwo/cli/`), y la `statusLine` de `~/.claude/settings.json` solo si
  su `command` es exactamente `tc2 estado --stdin`, sin contar los espacios
  de los extremos (sin tocar ningún otro byte del fichero). Los ficheros se
  reescriben de una vez (temporal y renombrado).
- **En Windows, el `tc2.exe` en marcha** no se puede borrar: lo borra un
  proceso auxiliar en cuanto `tc2` sale (como mucho 10 s; `motivo=al-salir`).
  Si otro proceso está ejecutando ese binario, su grupo (binario, `.old`,
  carpeta y entrada del PATH) queda `pendiente motivo=en-uso` sin tocar, el
  resto se quita y la orden sale `rechazada` (3).
- **Qué no toca**: los hooks de un `settings.json` (si uno nombra `tc2`, lo
  avisa); el plugin y el marketplace, que son de Claude Code (imprime
  `claude plugin uninstall 'taskcodetwo@<marketplace>' --scope <scope>`); un
  `tc2` de la caché del plugin o de un gestor de paquetes (imprime la orden
  del gestor: `npm uninstall -g taskcodetwo`, `brew uninstall taskcodetwo`,
  `winget uninstall taskcodetwo`, `scoop uninstall taskcodetwo` o `cargo
  uninstall tc2`); y nada de la 0.16 (`taskctl`, `taskcode-plugin`, su status
  line, `.git/taskcode/`), que desinstala F9-10.
- **Qué quita `--proyecto`**: el worktree de la rama de estado (`tareas/`) y
  los de tarea (`.taskcodetwo/worktrees/<id>`), con sus registros;
  `.git/taskcodetwo/` (cerrojo, caché de identidad, índices temporales) y las
  líneas `/.taskcodetwo/worktrees/` y `/tareas/` de `.git/info/exclude`. Si un
  worktree tiene cambios sin guardar o commits sin publicar, está bloqueado
  (`git worktree lock`) o contiene la carpeta desde la que se lanza, sale
  `rechazada` sin tocar nada. Nunca usa la red ni toca el remoto.
- **Con confirmación**: `.taskcode/config.yml` (solo la de formato 2, la que
  lee `tc2`) y `.taskcode/personas.yml` solo con `--config`; con la config de
  la 0.16, las dos se quedan, porque son compartidas; la rama local
  `taskcodetwo/tareas` y las refs `refs/taskcodetwo/*` solo con
  `--borrar-tareas taskcodetwo/tareas` (el nombre exacto). La rama remota
  nunca: imprime `git push origin --delete taskcodetwo/tareas` para que lo
  decida la persona. Si la rama local tiene commits que el remoto no tiene,
  la línea lleva `motivo=adelantada-del-remoto sha=<sha>` y stderr dice cómo
  recuperarlos (`git branch taskcodetwo/tareas <sha>`).
- **Uso**: `--config` y `--borrar-tareas` sin `--proyecto` dan `uso` (2), y
  otro nombre de rama en `--borrar-tareas`, también. Con `--maquina
  --proyecto` fuera de un repo sale `rechazada motivo=no-es-repo` sin hacer
  nada, tampoco en la máquina.

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
  `--comprobar` y la caché del plugin; la attestation con un `gh` simulado
  (`examples/gh_falso.rs`: ausente, válida y rechazada), y en Windows que la
  ejecución siguiente borra el `.old`. El fallo del segundo renombrado se
  prueba con un fallo inyectado.
- el job `actualizar-ensayo` de `release.yml`
  (`.github/scripts/probar-actualizar.sh`): en los 6 runners nativos, instala
  el binario del build con la línea de instalación y lo actualiza a otro
  compilado con la versión siguiente, con `--comprobar` y el hash alterado.
  Hasta el primer push con el job, **no verificado** en CI (TASK-027); su
  paso en Windows x64 se ensayó en local.

`tc2 desinstalar` se prueba en `cargo test -p tc2`
(`crates/tc2/tests/desinstalar.rs`) con un HOME falso y el PATH de usuario,
los procesos y el autoborrado en memoria (nunca los de la máquina), y en
`probar-instalador.sh local`: tras instalar con la línea, `desinstalar
--maquina` deja el perfil como antes y, en Windows, quita la entrada del PATH
de usuario y el `tc2.exe` en marcha. Hasta el primer push con ese paso, **no
verificado** en CI (TASK-078).

Los runners no son máquinas limpias (traen curl, gh y Node). La prueba en
máquinas limpias de verdad es de F9-9a (TASK-071).
