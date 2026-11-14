---
id: TASK-008
titulo: "F1-7 ADR verificacion en CI, perfiles, cache por SHA y runners"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-008-f1-7-adr-verificacion-en-ci-perfiles-cac
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1298087
tokens_implementacion: 7943285
tokens_revision: 2399604
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-006]
---
## Objetivo

ADR: verificación en CI, perfiles, caché por SHA y runners (GitHub y self-hosted Windows) (P6)

Tarea F1-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5.

## Criterios de aceptacion
- [ ] Coste estimado con las suites de ToolsaWeb y OpenDockTrading
- [ ] Cumple lo que pide la tarea F1-7 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T07:06:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T07:12:24Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T07:12:25Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T07:27:02Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T07:12:23Z | diseno | agente:af104e72ebb7fb2bf | 42 | 96950 | 1182579 | 18516 | 30 | 1298087 | - | - |
| 2026-11-14T07:27:02Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9650 | 1231125 | 2147 | 0 | 1242930 | - | - |
| 2026-11-14T07:27:02Z | implementacion | agente:a3bb03f6be501e154 | 86 | 221121 | 6412845 | 66303 | 15 | 6700355 | - | - |
| 2026-11-14T07:31:46Z | revision | agente:aa8f7011cd12b481e | 38 | 156583 | 2215961 | 27022 | 1 | 2399604 | 1 | - |

## Autorrevision

- code-quality-reviewer — 1a083e6
- code-quality-reviewer — fee7038
