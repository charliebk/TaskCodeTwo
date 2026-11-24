# Informe de revision — TASK-148b (ronda 1)

- Commit revisado: e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/importar.rs:62 | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/tests/comun/git.rs:15 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/tests/comun/lanzador.rs:63 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/tests/comun/remoto.rs:173 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Veredicto propuesto: cambios-solicitados (un IMPORTANTE de Regla 4D sin corregir). Los tres MENOR no bloquean.

### IMP-1 — Duplicacion residual de `publicado` (Regla 4D)

`Fixture::publicado(bare, ruta)` (`comun/remoto.rs:131`) es el ayudante nuevo y toma y liberar lo usan, pero quedan copias con otro nombre en ficheros del propio diff:

- `importar.rs:62`: `publicado(f, bare, id)` hace `f.crudo(bare, ["show", "RAMA:{id}/tarea.md"])`, identica a `f.publicado(bare, "{id}/tarea.md")`.
- `revision.rs:198`: `publicado(f, bare)` con `TASK-001/tarea.md` fijo, el mismo `git show` sobre `estado::RAMA`.
- `roadmap.rs:44`: `publicado(f, bare)` lanza `Command::new("git")` a mano con `f.entorno()` (sin `git_en`, que es lo que centraliza A8-20), solo para conservar los bytes sin `from_utf8_lossy`.
- Cerca: `roadmap.rs:22` conserva su propio `tc2` con el cuerpo de `lanzar` (devuelve la tripleta codigo/stdout/stderr), `frescura.rs:37` otro `lanzar` con idioma, y `aprobar.rs:189` y `:395` repiten el bloque `Command::new("git") ... verify-commit` (este ultimo ya existia antes).

Evidencia ejecutada: `grep -rn "fn publicado" crates/tc2/tests` da definiciones fuera de `comun/` en importar, revision y roadmap (ademas de las de ficheros fuera del diff); `grep -rn 'Command::new("git")' crates/tc2/tests | grep -v comun/` da roadmap.rs:45, revision.rs:498, aprobar.rs:189 y :395, estado_casos.rs:64. Las de importar y revision se sustituyen sin cambio por `f.publicado`; la de roadmap, por `f.publicado` o por una variante en bytes en `comun`. Los criterios de aceptacion no nombran `publicado`, pero el diff ya toca importar, revision y roadmap, y la Regla 4D pide no dejar el ayudante migrado duplicado con otro nombre.

### MEN-1 — `git_en`: tres protecciones sin test que las mate

Mutantes sobre `comun/git.rs`:

- Quitar `GIT_CONFIG_NOSYSTEM`: sobrevive (soporte 5 ok, inyeccion 4 ok). Ademas `Fixture::entorno()` (`comun/mod.rs:51`) ya lo fija, asi que solo importaria a `agente_git::Clon::git`, cuyo entorno no se comprueba. El test `git_en_aisla_la_configuracion_del_usuario` solo cubre `GIT_CONFIG_GLOBAL`.
- Quitar `LC_ALL=C`: sobrevive (soporte, inyeccion, git_binario 2 ok, toma 13 ok).
- Cambiar `o.status.success()` por `true`: sobrevive (soporte, inyeccion); ningun test espera el fallo de un git de escenario.
- Quitar el `assert!` de `GIT_CONFIG_GLOBAL`: lo mata `git_en_sin_gitconfig_propio_no_lanza_git` (bien).

No es regresion: el codigo anterior tampoco lo probaba. Basta un test `#[should_panic]` para un git que falla y otro que compruebe el entorno con el que se lanza.

### MEN-2 — `lanzador::salida`: el assert de stderr vacio en un exito no lo ejerce ningun test muestreado

`salida` sin `assert!(err.is_empty(), ...)` sobrevive en inyeccion (4 ok) y liberar (12 ok). Sustituir el codigo por `Some(0)` si lo mata liberar (8 fallos). Antes cada copia comprobaba el contrato «un exito no escribe en stderr»; ahora es una sola y `soporte.rs` no tiene test de `salida`.

### MEN-3 — `con_fila`: la comprobacion de fase, autor y version no la ejerce ningun test

Sustituir `fila.ends_with(" | {fase} | manual | {por} | {VERSION} |")` por `true` (`comun/remoto.rs:173`) sobrevive en liberar (12 ok) y ciclo (15 ok). El ayudante construye el texto esperado desde la propia fila, de modo que ese assert es la unica guarda de lo que escribio `tc2`. Falta un test en `soporte.rs` con una fila equivocada y `#[should_panic]`.

## Regla 4D

- Duplicacion: IMP-1
- Desviacion: cumple
- Diluvio: cumple
- Disfraz: MEN-2, MEN-3 (asserts de ayudante sin test que los mate; ningun test pasa con el codigo roto en los ayudantes ejercitados)

## Lecturas fuera del contexto permitido

- `git diff 8421bef..e7cc8d0` de toma, aprobar, roadmap, importar, revision y soporte: la peticion no embebe el diff de tests; hacia falta para comparar contra el codigo anterior.
- `skills/task-workflow/revision.md` del plugin: severidad de la Regla 4D.
- Resto de `crates/tc2/tests/*.rs` fuera del diff, solo con grep: buscar copias de los ayudantes migrados.

## Alcance

- Ficheros revisados: los 25 de tests de la peticion (`crates/tc2/tests/**`); solo cambian tests.
- Contexto adicional pedido: ver «Lecturas fuera del contexto permitido».
- Limite JS-WASM: sin cambios (tests nativos de un CLI Rust).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148b-r1`, commit a6fbbd3 (los tests son los de e7cc8d0), con `CARGO_TARGET_DIR` propio junto al clon; ambos borrados al terminar.
- Build: `cargo build -p tc2 --examples --tests --locked`, ok.
- Formato y lints: `cargo fmt --check -p tc2` limpio. Clippy no pasado (solo cambian tests de integracion).
- Suite: no repetida entera; se cita la de la peticion (771 ok, 1 fallo intermitente ajeno `prs_abiertos::fallos_tipados`, 13 ignorados). Muestra propia con `cargo test -p tc2 --locked --test <x>`, todo en verde: soporte 5, toma 13, liberar 12, solape 15, aprobar 10, frescura 12, estimacion 11, verificar 15 (1 ignorado), alta 12, ciclo 15, progreso 12, inyeccion 4, importar 9, actualizar_cli 11 (plan se lanzo tambien; su resultado no se anoto antes de pasar a mutantes). El hook rtk filtra la salida de cargo, se uso `rtk proxy cargo`.
- Casos construidos: grep de duplicados sobre `tests/` (IMP-1) y mutantes de los ayudantes de `comun`.
- Mutacion (25 mutantes en dos tandas; solo se corrieron los binarios citados, un superviviente lo es en ellos):
  - Muertos: `valor` con `.last()` (soporte), `valor` sin `strip_prefix('=')` (importar 7 fallos, soporte), `GIT_SSH_COMMAND` sin la rama `receive-pack` (frescura 6 fallos), `preparar` sin `personas.yml` (inyeccion), `Fixture::publicado` leyendo HEAD (liberar 5 fallos), `Fixture::punta` leyendo HEAD (aprobar 2 fallos), `identificar` con otro id (toma 4 fallos), `ORIGIN` en otro host (toma 11 fallos), `salida` con codigo fijo (liberar 8 fallos), `git_en` sin el assert de `GIT_CONFIG_GLOBAL` (soporte).
  - Supervivientes: los de MEN-1, MEN-2 y MEN-3. Equivalentes: `exit 255` por `exit 0` en `entorno_sin_red` (el «ssh» sigue sin servir nada; toma y frescura ok) y `editar` sin su assert de precondicion (el `replacen` sin efecto rompe luego los asserts del test; liberar 12 ok).
- Medidas: no aplican.

## Revisado sin hallazgos

- Migracion de `aprobar.rs` (`como_persona` fundido con `hito_como_persona` mediante closure; `planificada` equivale al `preparar` anterior), `toma.rs`, `liberar.rs` y `solape.rs`: mismas expectativas y mismo entorno (ssh falso, fechas por `git_ssh_con`, configuracion por test con `Preparar`).
- `valor` unica: ahora lee solo la primera linea y recorta. `roadmap.rs::lista` pasaba de `split_whitespace` a `split(' ')`, pero sus asserts son sobre listas concretas no vacias y los tests pasan.
- `git_en` exige `GIT_CONFIG_GLOBAL` y lo usan `Fixture::crudo` y `agente_git::Clon::git`; no queda `Command::new("git")` de escenario en los ficheros del criterio 1.
