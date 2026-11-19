---
id: TASK-037
titulo: "F4-2 approve con la identidad de la persona"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-037-f4-2-approve-con-la-identidad-de-la-pers
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2138683
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-007, TASK-036]
---
## Objetivo

`approve` con la identidad de la persona; el agente no puede aprobar (RR-04)

Tarea F4-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-6, F4-1.

## Criterios de aceptacion
- [ ] Sin identidad de persona → rechazo
- [ ] test
- [ ] Cumple lo que pide la tarea F4-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T04:59:16Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T05:05:00Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T05:05:01Z | start | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T05:04:59Z | diseno | agente:ac9a3cbc6b8d25b98 | 48 | 125281 | 1995222 | 18132 | 31 | 2138683 | - | - |

## Autorrevision

- rust-wasm-reviewer — 7b98d98
- code-quality-reviewer — 7b98d98
