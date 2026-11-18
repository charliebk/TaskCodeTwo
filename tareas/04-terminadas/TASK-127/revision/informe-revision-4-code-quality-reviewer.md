# Informe de revision — TASK-127 (ronda 4)

- Commit revisado: e7646219e90763898c5c0fb75a8298e8391f4486
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/git/estado.rs:355 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — verificado como corregido

IMP-1 (ronda 3): `construir_commit` no comprobaba que `hash-object` devolviera un sha por fichero, asi que una salida incompleta dejaba que el emparejamiento por orden diera a cada ruta el contenido de otra. La ronda 3 lo dejo abierto porque ningun test ejercitaba `construir_commit` con esa salida (mutacion W verde).

Verificacion en esta ronda:

- Cableado: `construir_commit` llama a `hashear(git, dir, op, &rutas, cambios.len())` en `crates/tc2/src/git/estado.rs:355`, y `hashear` (linea 304) exige `shas.len() == n` (linea 310) o devuelve `HASH_OBJECT_INCOMPLETO`.
- Test nuevo `construir_commit_rechaza_shas_incompletos` (lineas 570-594): git falso que responde un solo sha a `hash-object`, dos ficheros en el commit. Comprueba `Resultado::Error`, el diagnostico `HASH_OBJECT_INCOMPLETO` y que no se llega a `commit-tree`.
- Mutacion W (en el clon): en `construir_commit` se sustituye la llamada a `hashear` por una lectura directa de `con_entrada(... HashObject ...)` sin comprobar la cuenta. Resultado: `cargo test -p tc2 --lib tests_stdin` ROJO, `construir_commit_rechaza_shas_incompletos` falla en `estado.rs:584` (la mutacion llega hasta `write-tree` con `invalid object`, en lugar de dar `HASH_OBJECT_INCOMPLETO`). La proteccion ya tiene red de tests. Clon restaurado despues con `git checkout`.
- Sin mutar, `cargo test -p tc2 --lib tests_stdin --locked` en el clon de e7646219: 3 pasan (`hashear_exige_un_sha_por_fichero`, `construir_commit_rechaza_shas_incompletos`, `con_entrada_da_error_si_git_falla`).

### Hallazgos nuevos

Ninguno CRITICO. La correccion no introduce ningun fallo nuevo: `hashear` devuelve el mismo `Err` que antes, el `?` de `construir_commit` lo propaga antes de `update-index`, y `n = 0` no entra en el bloque porque `cambios` esta vacio. El `con_binario` de `#[cfg(test)]` solo existe en tests y no cambia el camino de produccion.

### Suite y puerta

- Suite: no repetida. La peticion trae la verificacion de la ronda en verde: `verif-127-r4b.log` en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo`, con la linea `xtask: verificacion Rapida en verde` y `exit=0` en `verif-127-r4b.log.fin`.
- Identidad del codigo: `git diff --stat cdbd5ba e764621 -- crates docs` vacio, asi que el codigo verificado en cdbd5ba es identico al de e7646219 (revisado).
- Tests concretos ejecutados por mi en un clon de e7646219 (carpeta borrada al terminar): `cargo test -p tc2 --lib tests_stdin --locked`, 3 pasan; y la mutacion W descrita arriba, roja.

### Notas (no son hallazgos de la tabla)

- La mutacion W se detecta por un diagnostico distinto del esperado (`write-tree` en lugar de `HASH_OBJECT_INCOMPLETO`). El test sigue siendo rojo, asi que cumple, pero si alguien quiere que el fallo apunte a la causa, el test podria comprobar tambien que el error es el de `hash-object`. MENOR, no bloquea.
- El git falso se ha ejecutado solo en Windows (rama `cfg!(windows)`). La rama Unix (`#!/bin/sh`, `$5 = hash-object`) no se ha ejecutado en esta maquina: queda sin verificar.
- El test `construir_commit_rechaza_shas_incompletos` comprueba la ausencia de `git commit-tree` en el registro de `Git`; el registro es el del proceso Rust, no el de los subprocesos. La comprobacion principal es el diagnostico, y esa es la que se ha visto fallar en la mutacion.

## Veredicto propuesto

`aprobada`, con IMP-1 `corregido` y ningun CRITICO ni IMPORTANTE abierto. La linea de veredicto de la cabecera queda sin tocar (PENDIENTE), para escribirla con `taskctl veredicto`.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-127/revision/informe-revision-3-code-quality-reviewer.md`: origen y criterio de IMP-1 (la peticion lo nombra como fuente).
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.2\skills\task-workflow\revision.md`: reglas comunes de revision que la skill manda leer.
- No se leyeron `tarea.md` ni `planificacion/` (listados en el contexto permitido): la verificacion es acotada a IMP-1 y no los necesita.
