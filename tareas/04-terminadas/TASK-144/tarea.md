---
id: TASK-144
titulo: "Auditoria 4D: A6 estimacion, progreso, roadmap y tablero (6 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-144-auditoria-4d-a6-estimacion-progreso-road
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1411378
tokens_implementacion: 21799570
tokens_revision: 14767061
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 6 casos IMPORTANTE del grupo A6 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 5 Duplicaciones (una `divergente`) y 1 Disfraz de test con mutante confirmado. A6-01 va en la tarea de la capa de órdenes (A8-06). No se pide el MENOR (A6-08). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A6-04 (`divergente`, va primero): el juego de tareas de `tests/siguiente.rs:53-94` y su copia de `tests/roadmap.rs:437-486` salen de una sola fuente. Si roadmap necesita `TASK-009` y otros títulos, lo dice su propio juego y no un comentario que remite al otro.
- [x] A6-02: `linea_esperada` (`grafo.rs:642-656`) usa `ids_de_la_linea` (`roadmap/generar.rs:401-416`) en vez de reimplementarla.
- [x] A6-03: la operación de prueba `Poner`, `fila` y `terminada` (`tests/estimacion.rs:278-345` ~ `tests/progreso.rs:92-155`) quedan en un módulo común de tests.
- [x] A6-05 y A6-06: `siguiente::tests` usa el `EnMemoria` de `roadmap::tests` (`siguiente.rs:139-149` ~ `roadmap/mod.rs:578-588`), y `c5_sin_red_y_sin_origin_sin_claves` llama a `start_de_luis` (`tests/solape.rs:514-529` ~ `:602-617`).
- [x] A6-07: `c7_fuera_de_un_repo` (`progreso/mod.rs:479-497`) tiene una entrada cuyo resultado depende de usar la carpeta del proceso como respaldo (adenda TASK-084 de F1-17, `orden_estado` en `:205-210`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib progreso::tests::c7_fuera_de_un_repo` sale en rojo.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 6 IDs con su commit.

## Resultado

Corregidos los IMPORTANTE del grupo A6 (A6-01 va en A8-06, TASK-149), solo en tests; sin cambios de produccion ni de API publica.

| Ficha | Commit |
|---|---|
| A6-04 (`divergente`) | e5a3dd8 |
| A6-02 | 2a027aa |
| A6-03 | 3a27379 |
| A6-06 | e40a036 |
| A6-07 (D4) | 250cf46 |
| A6-05 | 771e187 (TASK-148e: `estado::prueba::EnMemoria`) |

- A6-04: juego de tareas en `comun::tareas::juego_de_next()` (titulo `"Titulo de {id}"`) y `Fixture::publicar_estado`; `roadmap.rs` anade TASK-009 en su propio juego. Cambia solo el lado de siguiente (titulos de `"t"` a `"Titulo de {id}"`; ninguna asercion mira titulos). Decision 2: migradas tambien las cuatro publicaciones identicas de `doctor.rs`, `recursos.rs` y `tablero.rs`.
- A6-07: `c7_fuera_de_un_repo` comprueba el respaldo a la carpeta del proceso y falla si el temporal esta dentro de un repo. Verde con el codigo actual; rojo con el mutante de la ficha (`cwd.filter(|_| false)`: «tc2 · sin tarea en curso» frente a la linea de TASK-001) y con el control (`None => String::new()`: `""` frente a «tc2 · sin tarea en curso»).
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 800 (cierre de TASK-143); despues 48 binarios, 800 ok, 0 fallos, 13 ignorados (sin tests nuevos: solo se reescriben).
- El freno del hook de Stop dio un rojo intermitente en `hallazgos::la_ronda_siguiente_mide_nuevos_y_cerrados` (`PermissionDenied`, con otra compilacion a la vez); `hallazgos.rs` no cambia en esta tarea.
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- `doctor.rs` tenia su propio `tarea(id, estado)`, equivalente a `comun::tareas::tarea(id, 1, estado, &[])`: se sustituye (va en el commit de A6-04).
- `estimacion` conserva un `terminada` local de 5 lineas sobre el comun (carril, complejidad y dos criterios); el texto es identico byte a byte al de antes.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T04:29:29Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T04:34:52Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T04:34:55Z | start | automatico | persona | 0.17.1 |
| 2026-11-25T05:54:32Z | review | automatico | persona | 0.17.1 |
| 2026-11-25T06:51:50Z | finish | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T04:34:50Z | diseno | agente:a3be8fa8a8d8a9717 | 48 | 85474 | 1309237 | 16619 | 36 | 1411378 | - | - |
| 2026-11-25T05:54:32Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 28 | 14225 | 9448433 | 5538 | 0 | 9468224 | - | - |
| 2026-11-25T05:54:32Z | implementacion | agente:a93d1c403792c28cd | 164 | 194836 | 12095569 | 40777 | 22 | 12331346 | - | - |
| 2026-11-25T06:28:37Z | revision | agente:a9a15c295f6060d76 | 202 | 179430 | 14557980 | 29449 | 13 | 14767061 | 1 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — 250cf46
- 4D — 250cf46: helper buscado, criterios citados, 2 mutantes vistos en rojo

## Duracion

- Estimada: 150 min
- Real (de la rama abierta al cierre con merge, sin pausas): 137 min
- Paso de la hora: 137 min reales frente a 150 estimados
- Cierre (finish): 1340 s, de 2026-11-25T06:29:30Z a 2026-11-25T06:51:50Z
