# Informe de revision — TASK-046 (ronda 1)

- Commit revisado: e33879a25a4399d5755559aa147ff96497ca231a
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido | crates/tc2/src/confianza/cierre.rs | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido | crates/tc2/src/confianza/cierre.rs, crates/tc2/src/config/validar.rs | implementacion |
| MENOR-1 | MENOR | corregido | crates/tc2/tests/verificar.rs | implementacion |
| MENOR-2 | MENOR | corregido | crates/tc2/tests/verificar.rs | implementacion |
| MENOR-3 | MENOR | aceptado | crates/tc2/src/verificacion/ejecutar.rs | implementacion |
| MENOR-4 | MENOR | aceptado | docs/ci/verificacion-reutilizable.md | plan |

Detalle del estado:
- IMPORTANTE-1: corregido (c6ffbc1)
- IMPORTANTE-2: corregido (c6ffbc1, f7a9be6)
- MENOR-1: corregido (c6ffbc1)
- MENOR-2: corregido (c6ffbc1)
- MENOR-3: aceptado: Claude Code cierra stdin tras escribir el JSON; documentado en la guia (8fd2d67)
- MENOR-4: aceptado: limites de diseno (D2a, no versionados, TOCTOU, enlaces y submodulos) documentados en la guia (8fd2d67)

### IMPORTANTE-1 — El cierre no sigue scripts con espacios o caracteres no ASCII en el nombre (ni nada fuera de `[A-Za-z0-9_.%~\-\\/]`): se ejecutan sin confianza

- Donde: `crates/tc2/src/confianza/cierre.rs`, `es_de_token` y `candidatos` (el texto se parte en tokens por todo lo que no sea ASCII alfanumerico o `_ . % ~ - \ /`).
- Reproduccion (ejecutada, Windows, shell `cmd`, `LOCALAPPDATA` propio): repo con `scripts/verificar.bat` que hace `call "%~dp0lib\mi paso.bat"` (y otro con `verificación.bat`), ambos versionados. `tc2 confiar verificacion.comando` da `ficheros=1` (solo el principal). Se reescribe `scripts/lib/mi paso.bat` con contenido distinto y `tc2 verificar --perfil completa` sigue en `confianza=confiado hasheados=0`, ejecuta el script cambiado (la salida muestra `MALO-espacio`) y sale verde.
- Impacto: rompe el criterio «nada del repo sin confianza» para un script versionado y alcanzable por la orden, sin aviso alguno. Un nombre con espacio es habitual en Windows. La PoC A1 tiene la misma limitacion, pero la tarea la presenta como cierre fiable.
- Sugerencia: tokenizar respetando comillas (`"..."`, `'...'`) y aceptar cualquier caracter alfanumerico Unicode; o, como minimo, que `tc2 confiar` avise de los tokens entrecomillados con forma de ruta que no resuelven a un versionado, y documentar la limitacion en el ADR y en la ayuda.

### IMPORTANTE-2 — Un glob de `confianza_incluye` que no casa o que no compila se ignora en silencio, y el validador de config lo da por bueno

- Donde: `cierre.rs::calcular` (`if let Ok(g) = GlobBuilder...build()` descarta el glob invalido; `set.is_match(r)` es sensible a mayusculas y no quita `./`) frente a `config/validar.rs::todo_el_arbol` (que si trata `./` como forma admitida).
- Reproduccion (ejecutada): con `cfg/a.cfg` versionado y `confiar` tras cada config:
  - `'cfg/*.cfg'` -> `ficheros=3` (correcto);
  - `'./cfg/*.cfg'` -> `ficheros=1` (no casa nada). Despues de `confiar`, se cambia `cfg/a.cfg` y `tc2 verificar` responde `confianza=confiado hasheados=0` y ejecuta;
  - `'CFG/*.cfg'` en Windows (sistema de ficheros insensible, y `clave_busqueda` ya lo es para los candidatos) -> `ficheros=1`;
  - `'cfg/['` (glob invalido) -> `ficheros=1`, sin error en `confiar` ni en `verificar`.
- Impacto: la persona declara que un fichero entra en la confianza y no entra; el fallo es silencioso y abre la puerta justo donde la persona cree haberla cerrado. Es la situacion contraria a «falla cerrado».
- Sugerencia: normalizar `./` igual que el validador, usar `case_insensitive` donde `clave_busqueda` lo es, y devolver error (o rechazo) si un glob no compila; valorar avisar con `ficheros` extra cuando un glob no casa nada.

### MENOR-1 — Test que sobrevive a la mutacion: mayusculas en rutas del cierre
- Mutacion ejecutada: `clave_busqueda` con `cfg!(any(windows, target_os = "macos"))` reemplazado por `false` -> `cargo test -p tc2 --test verificar` sigue en 7 verdes. Ningun test ejerce la divergencia 4 (`Scripts\V.bat` ejecuta `scripts/v.bat`). Yo si la comprobe a mano (`call "%~dp0LIB\P.BAT"` hacia `scripts/lib/p.bat`: tras cambiar `p.bat` da `cambiado`), asi que el codigo funciona, pero nada lo protege.

### MENOR-2 — Test que sobrevive a la mutacion: la shell dentro de la huella
- Mutacion ejecutada: `orden_configurada` devuelve solo `v.comando` (sin `v.shell`) -> 7 verdes. Cambiar `shell` (de `sh` a otra shell o de `cmd` a `pwsh`) cambia lo que se ejecuta y la documentacion lo promete; no hay test.

### MENOR-3 — `--hook` bloquea esperando a que stdin se cierre
- Reproduccion: `(sleep 6) | tc2 verificar --perfil completa --hook` tarda 7 s (stdin abierto 6 s). `stop_hook_activo` hace `read_to_string` sobre stdin sin plazo. Claude Code cierra stdin tras el JSON, asi que no afecta al uso real, pero otro lanzador que lo deje abierto rompe el «rapido» y el presupuesto de 1 s. Un hilo con plazo corto, o leer solo si hay datos, lo evita.

### MENOR-4 — Limites aceptados o no reproducibles aqui (constan, no se corrigen necesariamente)
- Cambio que conserva `(tamano, mtime)`: reproducido (`touch -r` de un `.bat` de otro contenido y mismo tamano -> `confiado hasheados=0` y ejecuta `MALOS`). Es la D2a aceptada y esta en el plan; con `mtime` distinto da `cambiado`. Ademas `mtime_ns` devuelve 0 si el sistema no lo da: con tamano igual la cache acierta siempre. Documentarlo en el ADR como riesgo residual.
- TOCTOU entre hashear y ejecutar: por diseno no hay ventana cerrada (hash y `spawn` son pasos separados; la shell relee los scripts al ejecutarlos). No lo reproduje con un temporizador fiable; lo anoto por razonamiento.
- Enlaces simbolicos: en esta maquina no hay privilegio para `mklink`, **no reproducido**. Por lectura: `fs::metadata`/`fs::read` siguen enlaces, asi que un enlace a fichero versionado se hashea por su destino (bien); pero un enlace a **directorio** versionado (`lib -> ../otro`) deja `lib/x.sh` fuera del indice de `git ls-files`, por lo que `indice.script` no lo encuentra y el script que ejecuta la orden no entra en el cierre. Un submodulo se comporta igual (`is_file()` falso, se salta). Habria que probarlo en Linux.
- Una orden que llame a un script **no versionado** queda fuera por diseno (solo versionados). Reproducido: `scripts/local.bat` sin `git add`, cambiado tras confiar, se ejecuta con `confiado`. Coherente con el plan, pero conviene que `tc2 confiar` avise de los tokens con forma de ruta que existen en disco y no estan versionados.
- Modulos por nombre (`import ./util`, `python -m x`) y herramientas como `cargo` (Cargo.toml, build.rs): fuera, a declarar en `confianza_incluye`. Esta documentado en el modulo.
- Fichero de captura del hook `tc2-verificar-<pid>.log` en `temp_dir()`: nombre predecible y `File::create` sigue enlaces; en un `/tmp` compartido de Unix seria un truncado de ficheros ajenos. En Windows (carpeta temporal por usuario) no aplica. Usar `create_new` o un nombre aleatorio.

## Alcance
- Ficheros revisados: los 14 de la peticion (`confianza/cierre.rs`, `confianza/mod.rs`, `config/mod.rs`, `desinstalar/maquina.rs`, `git/puerto.rs`, catalogos i18n, `lib.rs`, `verificacion/ejecutar.rs`, `verificacion/mod.rs`, `tests/desinstalar.rs`, `tests/verificar.rs`).
- Contexto adicional pedido: `tarea.md` (`## Divergencias`), `planificacion/plan-final.md`, decision 5 de `F1-8` (grep de mtime/enlaces) y `config/validar.rs` (`todo_el_arbol`, `glob_malo`) para comparar la validacion de globs. Tambien la salida de `git ls-files` por razonamiento, sin leerla del repo.
- Limite JS-WASM: no aplica (sin `#[wasm_bindgen]` en el diff).

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-046-r1-rust`, checkout de e33879a; repos de prueba en `...\task-046-r1-rust-work` con `LOCALAPPDATA` apuntando a una carpeta propia.
- Build: `cargo test -p tc2 --test verificar` (debug) compila y pasa. No se ejecuto `cargo build --target wasm32` (no hay wasm en el diff).
- Formato y lints: no repetidos; la verificacion completa de e33879a (2026-11-21) se da por buena segun la peticion.
- Suite: `cargo test -p tc2 --test verificar` -> 7 passed, 1 ignored, 0 failed (11,5 s). El pesado en release no termino: la compilacion `--release` de dependencias no acabo en mas de 30 min en una maquina cargada y se cancelo. **No he medido la cifra de 1 s del pesado.** Medida parcial en debug: `tc2 verificar --perfil completa` en caliente sobre un repo trivial (incluye lanzar `cmd` y un script de una linea) tarda 357-367 ms; no equivale al cierre tipo ToolsaWeb.
- Casos construidos: ver cada hallazgo. Ademas, `--hook` sin confianza: sale 0 con una linea en stderr que explica que hacer y no ejecuta nada; con `{"stop_hook_active":true}` sale 0 sin tocar el repo. Mayusculas en la propia orden (`SCRIPTS\V.BAT`) -> `cambiado` (la orden entra en la huella tal cual), correcto.
- Mutacion (cada una con `cargo test -p tc2 --test verificar`):
  - cache sin mtime -> 2 rojos (muere);
  - `Cambiado` tratado como `Confiado` -> 1 rojo (muere);
  - sin recursion de candidatos -> 2 rojos (muere);
  - sin globs de `confianza_incluye` -> 1 rojo (muere);
  - hook bloquea sin confianza -> 1 rojo (muere);
  - id del almacen por raiz en lugar de dir comun -> 1 rojo (muere);
  - `clave_busqueda` sin insensibilidad -> verde (sobrevive, MENOR-1);
  - shell fuera de la huella -> verde (sobrevive, MENOR-2).
  Todas revertidas con `git checkout`.
- Medidas: ver Suite.

## Revisado sin hallazgos
- Clave del almacen y ubicacion: `sha256(canonicalize(dir_comun))` en `%LOCALAPPDATA%\taskcodetwo\confianza\<sha>.json`, fuera del arbol, compartido entre worktrees (cubierto por test y mutacion M7); si el fichero se borra o su `version` no coincide, todo vuelve a `nunca`.
- Errores sobre datos externos: sin `unwrap`/`expect`/`panic` en lo revisado; lectura del almacen, config y stdin devuelven valores neutros (falla cerrado o sigue); el rebanado de `lineas` usa `saturating_sub`.
- Adaptador `--hook`: no bloquea sin confianza, sin perfil, con config invalida ni con error de lanzado (sale 0 y explica); bloquea solo en rojo/timeout (2).
- `desinstalar --maquina` borra el almacen de confianza.
- Escritura del almacen: fichero temporal por pid y `rename`; la carrera solo puede restaurar una confianza ya existente (falla cerrado).
