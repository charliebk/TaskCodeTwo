---
id: TASK-011
titulo: "F1-10 ADR carriles y modo automatico"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-011-f1-10-adr-carriles-y-modo-automatico
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 461024
tokens_implementacion: 9566139
tokens_revision: 8970061
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-006]
---
## Objetivo

ADR: carriles y modo automático (qué para y dónde) (P7)

Tarea F1-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5.

## Criterios de aceptacion
- [ ] Política por carril, configurable
- [ ] Cumple lo que pide la tarea F1-10 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T09:09:11Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T09:12:57Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T09:12:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T09:30:38Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T09:43:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T09:51:37Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T11:34:55Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T09:12:50Z | diseno | agente:ab43d0f671eafb514 | 18 | 76303 | 373195 | 11508 | 16 | 461024 | - | - |
| 2026-11-14T09:30:38Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 4633 | 1567218 | 2252 | 0 | 1574111 | - | - |
| 2026-11-14T09:30:38Z | implementacion | agente:a680e063f95f50532 | 96 | 200771 | 7709541 | 81620 | 18 | 7992028 | - | - |
| 2026-11-14T09:37:14Z | revision | agente:a319c3cadfb076d57 | 62 | 128372 | 4190788 | 26790 | 3 | 4346012 | 1 | - |
| 2026-11-14T09:47:28Z | revision | agente:ada6025bdb1180fa2 | 30 | 167075 | 1916134 | 29158 | 9 | 2112397 | 2 | - |
| 2026-11-14T09:55:35Z | revision | agente:a9be7fa0f720a02ab | 40 | 113244 | 2373887 | 24481 | 8 | 2511652 | 3 | - |

## Autorrevision

- code-quality-reviewer — 4f82305
- code-quality-reviewer — 232628f
- code-quality-reviewer — e8e3306

## Rondas extra

- Ronda 3: Carlos, con la consigna «responder la opcion recomendada»; la correccion de IMP-1 en la ronda 1 quedo incompleta (release contra main sin exigir head develop) y un IMPORTANTE se corrige siempre; es barato, asi que se verifica en una ronda mas.
- code-quality-reviewer — 4f79940
- Ronda 4: Carlos (2026-11-14: «me interesa que los ADR de este plugin sean lo mas fiel posibles a los datos recolectados»); la correccion de la ronda 2 introdujo IMP-2 (union de puertas sin la politica configurada), que se corrige y se verifica en una ronda mas.
