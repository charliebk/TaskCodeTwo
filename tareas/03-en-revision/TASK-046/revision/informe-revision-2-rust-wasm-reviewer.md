# Informe de revision — TASK-046 (ronda 2)

- Commit revisado: 83f83d690e1069f9fa2c28c04630593d1c1fd353
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMPORTANTE-1 | IMPORTANTE | corregido (c6ffbc1), verificado: nombres con espacios y no ASCII siguen el cierre; quedan dos variantes residuales (ver nota) | crates/tc2/src/confianza/cierre.rs | implementacion |
| IMPORTANTE-2 | IMPORTANTE | corregido (c6ffbc1, f7a9be6), verificado: `./`, mayusculas, globs invalidos y globs sin casar | crates/tc2/src/confianza/cierre.rs, crates/tc2/src/config/validar.rs | implementacion |
| MENOR-1 | MENOR | corregido (c6ffbc1), verificado: la mutacion de la insensibilidad a mayusculas ya la mata un test | crates/tc2/tests/verificar.rs | implementacion |
| MENOR-2 | MENOR | corregido (c6ffbc1), verificado: la mutacion de la shell fuera de la huella ya la mata un test | crates/tc2/tests/verificar.rs | implementacion |
| MENOR-3 | MENOR | aceptado: Claude Code cierra stdin tras escribir el JSON; documentado en docs/ci/verificacion-reutilizable.md (8fd2d67) | crates/tc2/src/verificacion/ejecutar.rs | implementacion |
| MENOR-4 | MENOR | aceptado: limites de diseno (D2a, no versionados, TOCTOU, enlaces y submodulos) documentados en docs/ci/verificacion-reutilizable.md (8fd2d67) | docs/ci/verificacion-reutilizable.md | plan |
| MENOR-5 | MENOR | corregido (1c9f9e9) | crates/tc2/src/confianza/mod.rs:43-46 | implementacion |

Sin CRITICO ni IMPORTANTE abiertos. Ningun hallazgo nuevo CRITICO. MENOR-5 es cosmetico y no bloquea; puede corregirse en el commit de MENORES (`taskctl verificar-menores`) o aceptarse con su motivo.

### MENOR-5 — doc comment duplicado en VERSION_ALMACEN
- Donde: `crates/tc2/src/confianza/mod.rs:43-46`
- Que pasa: quedan dos lineas `///` seguidas, la original y la nueva; rustdoc las une en una sola descripcion repetida.
- Reproduccion: lectura del fichero tras el diff de c6ffbc1 (`/// Version del fichero del almacen.` seguido de `/// Version del fichero del almacen. La 2 (ronda 1 de TASK-046) ...`).
- Impacto: ninguno funcional; documentacion confusa.
- Sugerencia: borrar la primera linea.

## Nota (fuera de la tabla, no es hallazgo nuevo de esta ronda)

Residual de IMPORTANTE-1, no introducido por la correccion (la version anterior tampoco seguia estos casos). `entrecomillados` se detiene en la primera comilla sin cerrar de la linea, y los textos entre comillas que contienen comillas anidadas no se separan bien. Reproducido en el clon de HEAD 15a1859:
- cmd: `echo it's done & call "%~dp0lib\mi paso.bat"` con `scripts/lib/mi paso.bat` modificado tras `tc2 confiar verificacion.comando`: `verificar --perfil completa` da `confianza=confiado hasheados=0` y ejecuta el script modificado (salida `MALO`).
- sh: `. "$(dirname "$0")/lib/mi paso.sh"` con `shell: 'sh'`, `scripts/lib/mi paso.sh` modificado: `confiado` y ejecuta `MALO-SH`.
Son casos poco frecuentes (apostrofe en la misma linea antes de una ruta con espacios; ruta con espacios dentro de un `$(dirname ...)` en sh). Sugerencia: anadir ambos al apartado de limites de docs/ci/verificacion-reutilizable.md, o tratar la comilla sin cerrar como texto ordinario y seguir el resto de la linea.

## Alcance
- Ficheros revisados: crates/tc2/src/confianza/cierre.rs, crates/tc2/src/confianza/mod.rs, crates/tc2/src/config/validar.rs, crates/tc2/src/verificacion/ejecutar.rs, crates/tc2/src/lib.rs, crates/tc2/tests/verificar.rs, crates/tc2/tests/desinstalar.rs, docs/ci/verificacion-reutilizable.md, Cargo.toml de tc2 (dependencia `ring`).
- Contexto adicional pedido: `crates/tc2/src/config/claves.rs` (para ver que `GlobAcotado` solo usa `confianza_incluye`). El commit de la peticion era 83f83d6 pero HEAD de la rama es 15a1859: el delta entre ambos son solo ficheros de tareas/, sin codigo. Revisado HEAD 15a1859.
- Limite JS-WASM: no aplica. El diff no toca `#[wasm_bindgen]` ni tipos exportados, no hay llamador ni `.d.ts` que pedir.
- Lecturas fuera del contexto permitido: `crates/tc2/src/config/claves.rs` (motivo: ver si `GlobAcotado` tiene mas usuarios); `crates/tc2/Cargo.toml` (motivo: confirmar que `ring` es dependencia directa); la skill `task-workflow/revision.md` y la skill revisora (motivo: plantilla y reglas). `tareas/.../tarea.md` no se leyo.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-046-r2-rust`, checkout de 15a1859e4e8eaf4b206441239cfef8e0e499f744. Repo temporal de pruebas en `...\task-046-r2-rust-work\repo`, `LOCALAPPDATA` propio en `...\task-046-r2-rust-work\datos`. Ambos borrados al terminar.
- Build: `cargo build -p tc2 --locked` (debug) y `cargo test -p tc2 --test verificar --locked`. Sin release, sin wasm32 (no hay wasm en el diff).
- Formato: `cargo fmt --check -p tc2` -> exit 0.
- Lints: `cargo clippy` no se ejecutó en esta ronda (queda sin verificar).
- Suite: `cargo test -p tc2 --test verificar --locked` -> 12 passed, 0 failed, 1 ignored (el pesado, que solo corre con `--release --ignored`), 10,86 s. Linea base: la misma pasada es la de la ronda, sin repetir.
- Casos construidos con `target/debug/tc2.exe` (LOCALAPPDATA propio):
  - IMPORTANTE-1, cmd: `call "%~dp0lib\mi paso.bat"` y `verificacion.bat` versionados. `confiar` da `ficheros=2`. Tras modificar `mi paso.bat`: `verificar` da `confianza=cambiado` y sale 3 (no ejecuta). Con `call %~dp0verificación.bat` (no ASCII, sin comillas) y `verificación.bat` modificado: `cambiado`, sale 3. Corregido.
  - IMPORTANTE-2, globs (`confianza_incluye`):
    - `'./cfg/*.cfg'`: `ficheros=2` (incluye `cfg/a.cfg`). Tras modificar `cfg/a.cfg`: `cambiado`, sale 3. Corregido.
    - `'CFG/*.cfg'`: `ficheros=2`. Corregido en Windows.
    - `'cfg/*.CFG'`: `ficheros=2`. Corregido.
    - `'cfg/['`: `confiar` sale 3 con `config-invalida`, mensaje con el glob y el error. Corregido (rechazo cerrado).
    - `'./'`: rechazado por la configuracion (`nunca todo el arbol`), sin ejecutar. Corregido.
    - `'nada/*.x'`: `ficheros=1` y `globs-sin-ficheros=nada/*.x` mas una linea en stderr. Corregido (aviso).
- Mutacion, cada una sobre el clon con `cargo test -p tc2 --test verificar --locked`:
  - M1 `clave_busqueda` sin insensibilidad (MENOR-1): mata `el_cierre_no_distingue_mayusculas_en_windows_y_macos` (1 rojo).
  - M2 globs sin `case_insensitive` (IMPORTANTE-2): mata `globs_de_confianza_incluye_normalizados_invalidos_y_vacios` (1 rojo).
  - M3 globs sin quitar `./` (IMPORTANTE-2): mata `globs_de_confianza_incluye_normalizados_invalidos_y_vacios` (1 rojo).
  - M4 `orden_configurada` sin `shell` (MENOR-2): mata `cambiar_cierre_globs_u_orden_da_cambiado_y_volver_da_confiado` (1 rojo).
  - Todas revertidas con `git checkout`; el clon quedó limpio.
- Medidas: no aplican en esta ronda (sin cambios de rendimiento; el presupuesto de 1 s es el de la ronda 1).

## Revisado sin hallazgos
- `compilar_glob`: `./` repetido, globs con `**`, sensibilidad a mayusculas segun plataforma; `globs_vacios` cuenta bien aunque el glob case con un fichero ya encontrado por tokens.
- `calcular`: el guard `mtime != 0` evita la cache sin mtime; `ErrorCierre::Lectura` y `Glob` mapean a los informes correctos; la rama `Glob` queda como defensa en profundidad (la configuracion ya la rechaza con el mismo `compilar_glob`).
- `--hook`: `stop_hook_active` se mira antes que los argumentos; con perfil invalido sale 0 y avisa en stderr; la captura se crea con `create_new` y nombre aleatorio de 128 bits, se borra en todos los caminos que llegan a lanzar, y `cola` lee solo el final.
- `validar.rs`: el nuevo rechazo solo aplica a `GlobAcotado`, que solo usa `confianza_incluye` (claves.rs). Sin regresion en otras claves.
- Sin `unwrap`, `expect` ni `panic!` nuevos en el diff.
- `ring` ya es dependencia directa de tc2, sin dependencias nuevas.
