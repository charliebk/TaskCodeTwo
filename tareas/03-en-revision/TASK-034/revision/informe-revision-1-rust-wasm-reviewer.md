# Informe de revisión — TASK-034 (ronda 1)

- Commit revisado: db0bc53
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| 1 | MENOR | abierto | crates/tc2/src/identidad.rs:380-407 (`personas_de_base`) | heredado (TASK-033 y anteriores) |

### 1. Desde una subcarpeta del clon o del worktree, `pausa` y `descartar` dan `rechazada motivo=sin-personas`

`personas_de_base` hace `git ls-tree --name-only origin/develop -- .taskcode/personas.yml` con `dir` = cwd; el pathspec es relativo al cwd y desde una subcarpeta no casa. Reproducido con la fixture de `tests/ciclo.rs`: `tc2 pausa TASK-003` desde `<clon>/crates` y `tc2 descartar TASK-002 "x"` desde `<worktree>/crates` rechazan; desde la raiz funcionan. Falla de forma segura (no escribe). Afecta igual a `start`. Sugerencia: pathspec `:(top)` o resolver la raiz antes.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-034-r1-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios; `--test ciclo --test toma` 25/25.

Bordes probados con tests temporales: motivos raros (`a"b\c`, `x: y`, espacios, `null`, emoji) publican y repetir da `sin-cambios`; pausas pendientes sin red se publican luego con la hora original y en orden; una pausa pendiente de una tarea descartada por otro clon sale `rechazadas=1` sin bloquear; tarea en revision se pausa y descarta; worktree propio en uso se conserva (`en-uso`); desde otro worktree con rama sin publicar, `conservado sin-publicar`. Carrera con dos procesos simultaneos no reproducida (cubierta por la reejecucion de la transaccion, `estado/transaccion.rs:300-345`).

Mutaciones: re-descartar reescribiendo el motivo pone rojo `descartar_rechazos_y_sin_cambios`; `Pausar::exige_red` a `true` pone rojo `pausa_y_reanudar_por_filas`.

## Revisado sin hallazgos

`Lanzada::tarea` (RefCell reiniciado), `desde_anotacion` con `splitn(5)`, `motivo_valido` (control, U+2028, vacio, tope 500), `con_worktree`/`dentro_de`, `de_la_orden`, `Cuerpo::en_pausa`, rechazo `pausada` y filtro en `next`/ROADMAP. Sin unwrap/expect/panic ni unsafe en codigo no-test; sin dependencias nuevas. Limite JS-WASM: no aplica.
