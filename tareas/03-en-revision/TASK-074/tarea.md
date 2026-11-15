---
id: TASK-074
titulo: "F1-16 ADR lotes desatendidos con revision humana al final"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-074-f1-16-adr-lotes-desatendidos-con-revisio
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1214551
tokens_implementacion: 59106376
tokens_revision: 54281119
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-006, TASK-011]
---
## Objetivo

ADR: lotes desatendidos (RR-18): rama de lote, política de carriles, qué para el lote y qué aparta una tarea, presupuesto, reanudación, informe y su equivalente en GitLab

Tarea F1-16 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5, F1-10.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.

## Criterios de aceptacion
- [ ] Diseño probado a mano con un lote real de ToolsaWeb
- [ ] Cumple lo que pide la tarea F1-16 del plan y los requisitos que cita

## Autorrevision

- code-quality-reviewer — 8b7ecd7

## Rondas extra

- Ronda 3: Carlos (consigna «opcion recomendada» y fidelidad sobre coste); la ronda 2 encontro un IMPORTANTE introducido por la correccion (NUE-1, nucleo del manifiesto por lista blanca), ya corregido en d574e7f.
- Ronda 4: Carlos (consigna «opcion recomendada» y fidelidad sobre coste); la ronda 3 encontro dos IMPORTANTE introducidos por la correccion (NUE-5 lista de escrituras del CLI medida solo en ToolsaWeb, NUE-6 reversion parcial sin caso), ya corregidos en da557fc.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T22:31:25Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T22:40:40Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T22:40:41Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T01:35:02Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T03:30:20Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T05:32:03Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T07:29:31Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T22:40:38Z | diseno | agente:aaf90b533374b7743 | 26 | 64483 | 568740 | 12650 | 20 | 645899 | - | - |
| 2026-11-14T22:40:39Z | diseno | agente:acee77252357bf7a7 | 24 | 41249 | 351481 | 12111 | 23 | 404865 | - | - |
| 2026-11-14T22:40:39Z | diseno | agente:a07b5e3cf1cddbf60 | 10 | 37027 | 113758 | 12992 | 8 | 163787 | - | - |
| 2026-11-15T01:35:02Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 20 | 74213 | 1609308 | 6203 | 2 | 1689744 | - | - |
| 2026-11-15T01:35:02Z | implementacion | agente:a8fc16ef560a0e868 | 358 | 704231 | 56499646 | 212397 | 23 | 57416632 | - | - |
| 2026-11-15T02:23:24Z | revision | agente:abdc898e2e310828d | 148 | 324074 | 17678060 | 79997 | 18 | 18082279 | 1 | - |
| 2026-11-15T04:37:41Z | revision | agente:acfb15b4baef812b8 | 138 | 257097 | 13283983 | 59074 | 13 | 13600292 | 2 | - |
| 2026-11-15T06:29:49Z | revision | agente:af1f04fcab8b9ff6f | 178 | 777213 | 12708471 | 54329 | 11 | 13540191 | 3 | - |
| 2026-11-15T08:33:54Z | revision | agente:a8db2a077311e16a0 | 116 | 715346 | 8311400 | 31495 | 12 | 9058357 | 4 | - |

## Verificacion ligera

- 2026-11-15T08:33:56Z: ronda 4; commit revisado d57f5cc2451a17816ee38f9763f150a3349ebf15; commit verificado fd5b6e8ac99dbac3e17823e6085d5b75f8673e90; ficheros: ninguno; corregidos: ninguno; pendientes: ninguno
