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
tokens_revision: 4219489
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
| 2026-11-19T05:56:44Z | revision | agente:a1a31f66f13841e9a | 28 | 141610 | 1595487 | 12020 | 4 | 1749145 | 1 | rust-wasm-reviewer |
| 2026-11-19T05:56:44Z | revision | agente:a5663b5c690198b6c | 40 | 148224 | 2306425 | 15655 | 3 | 2470344 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 7b98d98
- code-quality-reviewer — 7b98d98

## Verificacion ligera

- 2026-11-19T06:07:10Z: ronda 1; commit revisado b9a6a55f86af56fe615da1af20a1480ea2d9ae0a; commit verificado 9399fbeb1b6a5e71ac294c11d53f0f9c2dbcaf11; ficheros: crates/tc2/src/git/estado.rs, crates/tc2/src/git/puerto.rs, crates/tc2/src/plan/plantillas.rs, crates/tc2/tests/aprobar.rs, crates/tc2/tests/estado_casos.rs, docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md; corregidos: MEN-2, MEN-3, MEN-2, MEN-3; pendientes: MEN-1, MEN-1
