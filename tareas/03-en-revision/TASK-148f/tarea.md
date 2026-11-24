---
id: TASK-148f
titulo: "dobles de Plataforma con Parcial y Doble (A8-05, absorbe A2-06) (parte f de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 739874
tokens_implementacion: 6446319
tokens_revision: 2353594
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148e]
---
## Objetivo

Parte f del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `plataforma::prueba::{Parcial, Doble}` y `comun/plataforma.rs` con `panic!` ante un método no esperado, y migra `identidad.rs`, `revision/mod.rs`, `tablero.rs`, `tests/revision.rs` y `tests/tablero.rs`.

## Criterios de aceptacion
- [x] `git grep -n 'impl Plataforma for' -- crates/tc2` solo devuelve `github.rs`, `gitlab.rs` y los dos `Doble<P>`, y no queda ningún `sin_uso`.
- [x] Dos tests `#[should_panic(expected = "merge-pr")]` sobre un `Doble` vacío, uno en `plataforma::prueba` y otro en `tests/soporte.rs`.
- [x] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-05 y A2-06 con su commit.

## Resultado

Dobles de `Plataforma` sobre `Parcial` y `Doble<P>` (por defecto `panic!` ante un metodo no esperado), una copia por crate: `plataforma::prueba` (unitarios) y `tests/comun/plataforma.rs` (integracion).

- **A8-05**: 50656db (`plataforma::prueba`; migrados los dobles de `identidad.rs`, `revision/mod.rs` y `tablero.rs`) y e7cb0e8 (`comun::plataforma`; migrados `tests/tablero.rs` y `tests/revision.rs`).
- **A2-06** (fusionado en A8-05): e7cb0e8.
- Criterio 1: `git grep -nE 'impl(<[^>]*>)? Plataforma for' -- crates/tc2` da solo `github.rs`, `gitlab.rs` y los dos `Doble<P>` (el patron literal `impl Plataforma for` no ve los genericos); `git grep -n sin_uso -- crates/tc2` vacio.
- Tests nuevos: `plataforma::prueba::tests::un_doble_vacio_no_admite_merge_pr` y `soporte::un_doble_vacio_no_admite_merge_pr`, en rojo con el mutante del `merge_pr` por defecto que devuelve `Err`.
- Ningun test llego a un metodo no esperado: no hubo que anadir metodos a ningun doble.
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 781 (780 medido en TASK-148e + 1 test de su correccion MEN-2); despues 48 binarios, 783 ok, 0 fallos, 13 ignorados (781 + 2).
- Hallazgo nuevo para la auditoria (no se arregla aqui): las constantes de operacion `MERGE_PR`, `PRS_ABIERTOS` y hermanas estan duplicadas y son privadas en `github.rs` y `gitlab.rs`.
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- El grep del criterio 1 se amplia a `impl(<[^>]*>)? Plataforma for` porque los dobles son genericos.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T08:18:58Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T08:22:54Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T08:22:55Z | start | automatico | persona | 0.17.1 |
| 2026-11-24T08:51:47Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T08:22:53Z | diseno | agente:a8e723e43eb235164 | 30 | 69947 | 657013 | 12884 | 29 | 739874 | - | - |
| 2026-11-24T08:51:47Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 12 | 10138 | 2814001 | 3809 | 0 | 2827960 | - | - |
| 2026-11-24T08:51:47Z | implementacion | agente:a76b08cd1ad0858b6 | 62 | 137093 | 3458406 | 22798 | 5 | 3618359 | - | - |
| 2026-11-24T09:10:22Z | revision | agente:ac937d5cc4b9f2583 | 40 | 133522 | 2206089 | 13943 | 3 | 2353594 | 1 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — e7cb0e8
- 4D — e7cb0e8: helper buscado, criterios citados, 2 tests mutados

## Verificacion ligera

- 2026-11-24T09:10:25Z: ronda 1; commit revisado fd776e6274ef35d6a465d6b19761918159920678; commit verificado 3b4cb728afca894c1087fdda118b2251d2361769; ficheros: crates/tc2/src/identidad.rs; corregidos: MEN-1; pendientes: ninguno
