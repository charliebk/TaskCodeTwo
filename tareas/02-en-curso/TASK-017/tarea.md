---
id: TASK-017
titulo: "F2-2 Capa de operaciones Git deterministas"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-017-f2-2-capa-de-operaciones-git-determinist
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 235247
tokens_implementacion: null
tokens_revision: null
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

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T12:33:18Z | diseno | agente:a5e8e6789a0fd006c | 14 | 43470 | 180062 | 11701 | 15 | 235247 | - | - |

## Autorrevision

- rust-wasm-reviewer — 743a5d3
