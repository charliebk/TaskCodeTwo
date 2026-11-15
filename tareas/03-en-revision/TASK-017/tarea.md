---
id: TASK-017
titulo: "F2-2 Capa de operaciones Git deterministas"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-017-f2-2-capa-de-operaciones-git-determinist
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 235247
tokens_implementacion: 6797656
tokens_revision: 2855473
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-005, TASK-016]
---
## Objetivo

Capa de operaciones Git deterministas: rama, worktree, commit de lo escrito, push con reintento, tag (P5)

Tarea F2-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-4, F2-1.

## Criterios de aceptacion
- [x] Tests contra repos reales
- [x] salida ≤ 5 líneas en el caso normal
- [x] Cumple lo que pide la tarea F2-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T12:29:47Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T12:33:25Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T12:33:27Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T12:49:09Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T13:05:50Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T12:33:18Z | diseno | agente:a5e8e6789a0fd006c | 14 | 43470 | 180062 | 11701 | 15 | 235247 | - | - |
| 2026-11-15T12:49:09Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 60 | 92766 | 6631285 | 73545 | 3 | 6797656 | - | - |
| 2026-11-15T12:57:32Z | revision | agente:a0fc86c42bdd6b290 | 38 | 143695 | 2682613 | 29127 | 1 | 2855473 | 1 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — 743a5d3
- rust-wasm-reviewer — ca600a5
