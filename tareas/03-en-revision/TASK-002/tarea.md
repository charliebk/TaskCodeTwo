---
id: TASK-002
titulo: "F1-1 ADR carpeta fija por tarea y rama de estado"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-002-f1-1-adr-carpeta-fija-por-tarea-y-rama-d
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 226256
tokens_implementacion: 3147544
tokens_revision: 1272317
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-001]
---
## Objetivo

ADR: carpeta fija por tarea con estado como campo, en rama propia `taskcode/tareas` montada como worktree en `tareas/` (P1), frente a estado en `develop`; contenido del ROADMAP

Tarea F1-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-0.

## Criterios de aceptacion
- [ ] Decisión con pruebas de concepto de las dos
- [ ] cómo se ve en GitHub
- [ ] Cumple lo que pide la tarea F1-1 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T02:48:25Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T02:53:41Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T02:53:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T03:00:28Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T02:53:14Z | diseno | agente:aef3992f14554e6a0 | 16 | 35501 | 179008 | 11731 | 17 | 226256 | - | - |
| 2026-11-14T03:00:28Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 40 | 35155 | 3095625 | 16724 | 0 | 3147544 | - | - |
| 2026-11-14T03:05:12Z | revision | agente:a481e4ea76a12e0a3 | 24 | 115790 | 1147095 | 9408 | 1 | 1272317 | 1 | - |

## Autorrevision

- code-quality-reviewer — f4795b5

## Verificacion ligera

- 2026-11-14T03:05:43Z: ronda 1; commit revisado 6fefde2c4f2945c2736aa409299f5a5cc4d2bc91; commit verificado fab33650e9cf5144c05797c253bdfdbffdadb3be; ficheros: docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md; corregidos: MENOR-1, MENOR-2, MENOR-3; pendientes: ninguno
