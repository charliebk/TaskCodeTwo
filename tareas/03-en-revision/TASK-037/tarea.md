---
id: TASK-037
titulo: "F4-2 approve con la identidad de la persona"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-037-f4-2-approve-con-la-identidad-de-la-pers
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2138683
tokens_implementacion: 24406246
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
| 2026-11-19T05:47:44Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T05:04:59Z | diseno | agente:ac9a3cbc6b8d25b98 | 48 | 125281 | 1995222 | 18132 | 31 | 2138683 | - | - |
| 2026-11-19T05:47:44Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 16 | 14032 | 5087140 | 4043 | 0 | 5105231 | - | - |
| 2026-11-19T05:47:44Z | implementacion | agente:a3a1ca06d00d1cb52 | 150 | 328040 | 18896870 | 75955 | 28 | 19301015 | - | - |

## Autorrevision

- rust-wasm-reviewer — 7b98d98
- code-quality-reviewer — 7b98d98
