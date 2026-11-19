# Informe de revision — TASK-038 (ronda 1)

- Commit revisado: 5ae20d8b11347556bddbfe0fe074e9827f4825f5
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/plan/aprobar.rs:340 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — Un `tarea.md` ilegible del hito se omite en silencio del lote
- Donde: crates/tc2/src/plan/aprobar.rs:340 (`del_hito`, `.ok()?`) y su uso en `AprobarHito::aplicar` (`continue`).
- Que pasa: si el `tarea.md` de una tarea no se entiende, `del_hito` devuelve `None` y `aplicar` la salta sin contarla ni listarla; el "todo o nada" no la ve (no se sabe su sprint) y el lote se firma sin ella. El motivo `ilegible` que ya existe en F4-2 (`tarea_de`) no se alcanza desde `--hito`.
- Reproduccion: no ejecutada (por lectura del codigo). Caso a ejecutar: un hito de 3 tareas listas mas una cuarta con `tarea.md` con YAML roto; esperado segun la adenda ("si alguna tarea abierta del hito no se puede aprobar, no se aprueba ninguna"): rechazo; observable segun el codigo: se aprueban las 3 y la cuarta ni aparece.
- Impacto: de borde (requiere un `tarea.md` corrupto, que `doctor` ya senala); la persona firma el hito creyendo que esta completo. No hay perdida de datos: la tarea queda sin aprobar.
- Sugerencia: o bien anadirla a `fallos` con motivo `ilegible` cuando la ruta sea de una tarea y no se pueda leer, o dejar escrito en la adenda F4-3 de F1-6 que las ilegibles no cuentan.

## Que se ejecuto (sin otros hallazgos)

- Suite: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre 5112652 segun `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/verif-038.log` (ultima linea: "xtask: verificacion Rapida en verde"); `git diff --stat 5112652 5ae20d8 -- crates docs` vacio, asi que el codigo es identico al revisado. No se repitio.
- En clon limpio en 5ae20d8: `cargo test -p tc2 --test aprobar -- --test-threads=4`: 9 passed, 0 failed (197 s, incluye `f4_3_approve_hito_aprueba_diez_en_un_commit_firmado` con firma SSH real y `verify-commit`, `f4_3_approve_hito_todo_o_nada`, `f4_3_approve_hito_agente_y_uso`). `cargo test -p tc2 --lib plan`: 36 passed.
- CLI real (via el binario en los tests E2E): `approve --hito 4` sin persona da `approve rechazada hito=4 motivo=identidad rol=persona`, exit 3, con RR-04 en stderr y remoto intacto; `TASK-001 --hito 4`, `--hito 4 TASK-001`, `--hito 4 --pedir-cambios x.md`, `--hito x`, `--hito` dan `uso` (exit 2) con el motivo esperado y sin tocar remoto ni clon.
- Mutacion (clon, restaurado tras cada una): `>` por `>=` en el tope: cae `hito_justo_en_el_tope`; incluir las ya aprobadas: cae `hito_justo_en_el_tope`; no excluir `descartada`: cae `hito_todo_o_nada`; quitar `hito-vacio`: cae `hito_todo_o_nada`; no filtrar por abierta: caen `aplicar_hito_aprueba_diez` y `hito_todo_o_nada`; segundo `op.sello()` de `transaccion.rs` sustituido por trailers vacios: cae el E2E de diez (`trailers == blobs`). Sobreviven dos mutantes sin efecto observable: aceptar `--hito H` vacio (el `parse` ya lo rechaza, equivalente) y quitar el `lote.clear()` inicial de `aplicar` (el lote solo se lee tras un `aplicar` correcto, que lo reescribe; equivalente en la practica).
- F4-2 no editado: `git diff 1f2243d 5ae20d8 -- crates/tc2/tests/aprobar.rs` solo anade (246 inserciones, 0 borrados); en `src/plan/aprobar.rs` solo se borra el helper `rechazo` de `Aprobar` (movido a `aprobable`), ningun test.
- Mensajes: `no-listas` lleva `tareas=ID:motivo,...` y el texto dice que se arreglen y se relance; `demasiadas` dice aprobar una a una con `tc2 approve TASK-NNN`; `hito-vacio` remite a `tc2 roadmap`. Dicen que hacer.
- Adendas: F1-6 y F1-16 coinciden con el codigo (forma, uso, todo o nada, tope contando solo las que cambian, `sin-cambios` sin firmar, N `Plan-Blob` en orden de ID con blobs del arbol del propio commit, doble llamada a `sello()`: las unicas implementaciones previas, `Aprobar` y el doble de `estado_casos.rs`, devuelven lo mismo las dos veces). Dicen "No verificado" la firma con persona real y FIDO, coherente con F4-2. La regla 5 del `carril` queda como pendiente de F6, fuera de alcance.
- Cumple F4-3 del plan ("diez planes aprobados de una vez, con nombre y fecha"): E2E con 10 tareas en un commit, firma verificada, `Operacion: approve-hito|4|1001|ana` y fila de Transiciones identica con fecha.

## Lecturas fuera del contexto permitido

ninguna
