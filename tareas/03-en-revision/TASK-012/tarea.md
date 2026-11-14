---
id: TASK-012
titulo: "F1-11 ADR extensiones declaradas"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-012-f1-11-adr-extensiones-declaradas
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2006835
tokens_implementacion: 17876333
tokens_revision: 13249104
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-002, TASK-008]
---
## Objetivo

ADR: extensiones declaradas (P8)

Tarea F1-11 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-7.

## Criterios de aceptacion
- [ ] Contrato que cubre lo que hoy hacen ToolsaWeb y OpenDockTrading a mano
- [ ] Cumple lo que pide la tarea F1-11 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T11:39:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T11:48:00Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T11:48:01Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T12:12:41Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T12:58:19Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T13:17:14Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T11:47:59Z | diseno | agente:a7a81b541b5f7788e | 76 | 94799 | 1889130 | 22830 | 72 | 2006835 | - | - |
| 2026-11-14T12:12:41Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 32 | 31131 | 7143929 | 15007 | 0 | 7190099 | - | - |
| 2026-11-14T12:12:41Z | implementacion | agente:a64e221658539d488 | 116 | 479790 | 10096105 | 110223 | 17 | 10686234 | - | - |
| 2026-11-14T12:27:28Z | revision | agente:a69450eddd3b1991b | 98 | 199118 | 8462467 | 58566 | 8 | 8720249 | 1 | - |
| 2026-11-14T13:05:25Z | revision | agente:a6be17e5ef0fa24af | 52 | 221490 | 4262871 | 44442 | 11 | 4528855 | 2 | - |

## Autorrevision

- code-quality-reviewer — 8b01847
- code-quality-reviewer — 81aee93
- code-quality-reviewer — 8f63e07

## Rondas extra

- Ronda 3: Carlos (2026-11-14: «me interesa que los ADR de este plugin sean lo mas fiel posibles a los datos recolectados»); la correccion de la ronda 1 introdujo IMPORTANTE-5 (check estado ejecutaba el comprobador del PR), que se corrige siempre y se verifica en una ronda mas.
