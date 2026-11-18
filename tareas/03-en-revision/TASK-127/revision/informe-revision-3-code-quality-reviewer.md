# Informe de revision — TASK-127 (ronda 3)

- Commit revisado: fa39e430c433bf9411d29cb9b108ee2daf7b55df
- Revisor: code-quality-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/git/estado.rs:355 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

## Verificacion (ronda 3, acotada)

### Puerta y suite

- Suite: no repetida. La peticion trae la verificacion de la ronda en verde sobre eb0eb3d: log `verif-127-r3.log` en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo`, con `xtask: verificacion Rapida en verde` y `exit=0` en `verif-127-r3.log.fin`. `git diff --stat eb0eb3d fa39e43 -- crates docs` esta vacio: el codigo verificado es identico al revisado.
- Tests ejecutados por mi en un clon de fa39e43 (carpeta borrada al terminar):
  - `cargo test -p tc2 --lib tests_stdin`: 2 pasan (`hashear_exige_un_sha_por_fichero`, `con_entrada_da_error_si_git_falla`).
  - `cargo test -p tc2 --test estado_casos`: 7 pasan, incluido `mil_ficheros_en_una_transaccion_con_procesos_acotados`. Log: `estado-casos-r3.log`.

### Cableado de `hashear` (fuente de IMP-1)

- `hashear` esta en `crates/tc2/src/git/estado.rs:304`; el control `if shas.len() == n` en la linea 310 devuelve `HASH_OBJECT_INCOMPLETO`.
- `construir_commit` (linea 320) llama a `hashear(git, dir, op, &rutas, cambios.len())` en la linea 355. Confirmado por lectura del codigo.
- Mutacion A (control anulado: linea 310 `if shas.len() == n {` pasa a `if true {`): `tests_stdin` ROJO. `hashear_exige_un_sha_por_fichero` falla con panic en `estado.rs:527`. La red de tests del control existe.
- Mutacion W (cableado retirado: la linea 355 vuelve a leer la salida de `hash-object` sin pasar por `hashear`, es decir, sin control): `tests_stdin` VERDE (2/2) y `estado_casos` VERDE (7/7). Log: `mut-w-r3.log`. Ningun test llama a `construir_commit` con una salida de `hash-object` incompleta, asi que el fallo original vuelve sin que la suite lo note.

### Por que IMP-1 queda abierto

La ronda 2 dejo IMP-1 con una pieza concreta: un test que ejercite `construir_commit` con un `hash-object` que devuelva menos lineas que rutas, y que compruebe `HASH_OBJECT_INCOMPLETO` y que el estado no avanza. Esa prueba no se ha anadido. La correccion mueve el control a `hashear`, y `hashear` esta probada (mutacion A), pero el hecho que IMP-1 describia, que `construir_commit` use el control, sigue sin red (mutacion W). Por eso se mantiene `abierto`.

Los llamadores de `construir_commit` (`crates/tc2/src/estado/transaccion.rs:370` y `:535`, `crates/tc2/src/init/aplicar.rs:129`) pasan por este camino, asi que el hueco afecta a commits reales.

### Hallazgos nuevos

Ninguno. Se busco el fallo que abre la correccion: `hashear` devuelve `Err` con el mismo diagnostico que antes, `construir_commit` lo propaga con `?` y el caso `n = 0` con salida vacia sigue siendo correcto. No se abre ningun CRITICO.

### Verificacion no ejecutada

No se ha ejecutado la suite entera sobre fa39e43 por mi cuenta; se cita la de la ronda (eb0eb3d, identico en `crates` y `docs`).

## Veredicto propuesto

`cambios-solicitados`: IMP-1 sigue abierto. Falta el test que ejercite `construir_commit` con una salida de `hash-object` incompleta. Con eso, IMP-1 pasaria a `corregido`.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-127/revision/informe-revision-2-code-quality-reviewer.md`: origen y criterio de IMP-1 (la peticion lo nombra como fuente).
- Llamadores de `construir_commit` en `crates/tc2/src/estado/transaccion.rs` (lineas 370 y 535) e `crates/tc2/src/init/aplicar.rs` (linea 129): para medir el alcance del hueco.
- Lectura de `crates/tc2/src/git/estado.rs` solo en las lineas de `hashear` y `construir_commit`. No se han leido `tarea.md` ni `planificacion/`, al no ser necesarios para esta verificacion acotada.
