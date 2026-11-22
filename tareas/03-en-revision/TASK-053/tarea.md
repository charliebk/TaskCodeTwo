---
id: TASK-053
titulo: "F6-7 Defensa contra prompt injection en PR y tareas"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-053-f6-7-defensa-contra-prompt-injection-en
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2837471
tokens_implementacion: 37286137
tokens_revision: 6809317
creado: 2026-11-14
actualizado: 2026-11-22
dependencias: [TASK-009, TASK-049]
---
## Objetivo

Defensa contra prompt injection en PR y tareas

Tarea F6-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-8, F6-3.

## Criterios de aceptacion
- [ ] Batería de PR maliciosos
- [ ] Cumple lo que pide la tarea F6-7 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-22T13:06:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-22T13:16:03Z | approve | automatico | persona | 0.16.1 |
| 2026-11-22T13:16:05Z | start | automatico | persona | 0.16.1 |
| 2026-11-22T14:21:37Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-22T13:16:03Z | diseno | agente:abd337c8ada9470fb | 70 | 119915 | 2690589 | 26897 | 46 | 2837471 | - | - |
| 2026-11-22T14:21:37Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 22 | 16433 | 3670211 | 4713 | 0 | 3691379 | - | - |
| 2026-11-22T14:21:37Z | implementacion | agente:adb56f3f403a0bc63 | 290 | 675360 | 32793408 | 125700 | 38 | 33594758 | - | - |
| 2026-11-22T14:29:42Z | revision | agente:a52c89e7e1e0f5db3 | 62 | 147209 | 3884951 | 21255 | 4 | 4053477 | 1 | rust-wasm-reviewer |
| 2026-11-22T14:29:42Z | revision | agente:a77c229bd25e5de96 | 44 | 154953 | 2580472 | 20371 | 2 | 2755840 | 1 | code-quality-reviewer |

## Verificacion ligera

- 2026-11-22T14:44:28Z: ronda 1; commit revisado bc834c8d5e302c27d69f32e2d7ac3de286836974; commit verificado 51ba9f4a48189a2db09281d13295f633e31741c3; ficheros: crates/tc2/src/hallazgos/mod.rs, docs/ci/revision-ia.md, xtask/src/workflows.rs; corregidos: MEN-1, MEN-2, MEN-1, MEN-2; pendientes: ninguno
