---
id: TASK-013
titulo: "F1-12 ADR actualizacion de proyectos entre versiones"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-013-f1-12-adr-actualizacion-de-proyectos-ent
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1028522
tokens_implementacion: 38206341
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-002, TASK-010]
---
## Objetivo

ADR: actualización de proyectos entre versiones (RR-13)

Tarea F1-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-9.

## Criterios de aceptacion
- [ ] Qué migra, simulación, vuelta atrás, convivencia 0.16/2.0
- [ ] Cumple lo que pide la tarea F1-12 del plan y los requisitos que cita

## Decisiones de la persona

- 2026-11-14, Carlos, en la conversacion de la tarea tras ver la propuesta D1a: "Aprobamos los nombres". Quedan aprobados `tc2` (CLI), `taskcodetwo` (plugin, prefijo de skills, `.git/taskcodetwo/`, rama `taskcodetwo/tareas`) y el paquete npm `taskcodetwo` con binario `tc2`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T13:24:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T13:30:16Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T13:30:19Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T15:07:32Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T15:57:40Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T13:30:13Z | diseno | agente:ae62d302be5a9fd09 | 42 | 81404 | 929104 | 17972 | 31 | 1028522 | - | - |
| 2026-11-14T15:07:32Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 12 | 69560 | 1732286 | 3615 | 1 | 1805473 | - | - |
| 2026-11-14T15:07:32Z | implementacion | agente:a16bae7be7f2e33f9 | 274 | 391337 | 35860826 | 148431 | 40 | 36400868 | - | - |

## Autorrevision

- code-quality-reviewer — 7f431ff
- code-quality-reviewer — 3c44c18
- code-quality-reviewer — d4799bc
