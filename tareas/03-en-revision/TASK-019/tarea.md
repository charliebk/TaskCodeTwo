---
id: TASK-019
titulo: "F2-4 Reserva de IDs por push atomico"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-019-f2-4-reserva-de-ids-por-push-atomico
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 457193
tokens_implementacion: 4012088
tokens_revision: 3859867
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-018]
---
## Objetivo

Reserva de IDs por push atómico

Tarea F2-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3.

## Criterios de aceptacion
- [ ] Dos clones crean tarea a la vez → IDs distintos
- [ ] test E2E
- [ ] Cumple lo que pide la tarea F2-4 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T15:00:18Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T15:04:10Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T15:04:12Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T15:13:06Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T15:04:10Z | diseno | agente:a3b86e10be5377cdf | 24 | 57311 | 387358 | 12500 | 22 | 457193 | - | - |
| 2026-11-15T15:13:06Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 46 | 55785 | 3936888 | 19369 | 12 | 4012088 | - | - |
| 2026-11-15T15:21:52Z | revision | agente:aee0dbc80a635524c | 26 | 132947 | 1365623 | 12761 | 2 | 1511357 | 1 | rust-wasm-reviewer |
| 2026-11-15T15:21:52Z | revision | agente:acda122e8d090f708 | 42 | 127883 | 2206439 | 14146 | 2 | 2348510 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — c121de2
- code-quality-reviewer — c121de2

## Verificacion ligera

- 2026-11-15T15:24:19Z: ronda 1; commit revisado f7f9446522807d5cea787a20dc1286abdf525a6d; commit verificado b82ba1bf6d54241566e624ea15d0ddf6b152c3a0; ficheros: crates/tc2/src/alta.rs, crates/tc2/src/lib.rs; corregidos: MENOR-1, MENOR-2, MEN-1, MEN-2; pendientes: ninguno
