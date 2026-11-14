---
id: TASK-015
titulo: "F1-13 ADR adaptador de plataforma GitHub y GitLab"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-015-f1-13-adr-adaptador-de-plataforma-github
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 473765
tokens_implementacion: 17994852
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-005]
---
## Objetivo

ADR: adaptador de plataforma GitHub y GitLab (RR-15): equivalencias (rulesets y ramas protegidas, checks y pipelines, reviews y approvals, cola y tren de merge, Apps y tokens de proyecto) y qué depende del plan de pago

Tarea F1-13 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-4.

## Criterios de aceptacion
- [ ] Tabla de equivalencias verificada en las dos
- [ ] cómo se prueba cada adaptador
- [ ] Cumple lo que pide la tarea F1-13 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T19:36:45Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T19:42:06Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T19:42:08Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T20:12:05Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T20:26:08Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T19:42:06Z | diseno | agente:a55220d0c7c37c631 | 22 | 58503 | 397181 | 18059 | 25 | 473765 | - | - |
| 2026-11-14T20:12:05Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 13611 | 996156 | 3319 | 0 | 1013094 | - | - |
| 2026-11-14T20:12:05Z | implementacion | agente:a4b9d65020a563362 | 172 | 332079 | 16508906 | 140601 | 19 | 16981758 | - | - |

## Autorrevision

- code-quality-reviewer — b94e6fe
- code-quality-reviewer — ca2d75f

## Verificacion ligera

- 2026-11-14T20:37:49Z: ronda 2; commit revisado dd39c4108a460d33676e2d283ea2012b370bd381; commit verificado b6e01816f2517ca8520394f404cdc1b8056da087; ficheros: docs/adr/F1-13-adaptador-de-plataforma.md, docs/adr/README.md; corregidos: MEN-7, MEN-8; pendientes: MEN-6
