---
id: TASK-021
titulo: "F2-6 Adaptador GitHub"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-021-f2-6-adaptador-github
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1588800
tokens_implementacion: 11346466
tokens_revision: 13119925
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-015, TASK-017]
---
## Objetivo

Adaptador GitHub (PR, checks, reviews, identidad) sobre la API, con errores dirigidos a la persona

Tarea F2-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-13, F2-2.

## Criterios de aceptacion
- [ ] E2E contra un repo de pruebas en GitHub
- [ ] Cumple lo que pide la tarea F2-6 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T16:50:45Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T16:58:01Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T16:58:09Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T17:23:03Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T17:58:36Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T16:58:01Z | diseno | agente:a14748c38791202ec | 44 | 98344 | 1467204 | 23208 | 33 | 1588800 | - | - |
| 2026-11-15T17:23:03Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 88 | 111688 | 11159050 | 75640 | 8 | 11346466 | - | - |
| 2026-11-15T17:45:39Z | revision | agente:aea0aea00fe5fdb3c | 80 | 196363 | 6112208 | 39711 | 9 | 6348362 | 1 | rust-wasm-reviewer |
| 2026-11-15T17:45:39Z | revision | agente:adc5fa1c079dffb72 | 84 | 206486 | 6530530 | 34463 | 14 | 6771563 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — cd3479a
- code-quality-reviewer — cd3479a
- rust-wasm-reviewer — 229caed
- code-quality-reviewer — 229caed
