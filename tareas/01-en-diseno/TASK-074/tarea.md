---
id: TASK-074
titulo: "F1-16 ADR lotes desatendidos con revision humana al final"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: feature/task-074-f1-16-adr-lotes-desatendidos-con-revisio
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1214551
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-006, TASK-011]
---
## Objetivo

ADR: lotes desatendidos (RR-18): rama de lote, política de carriles, qué para el lote y qué aparta una tarea, presupuesto, reanudación, informe y su equivalente en GitLab

Tarea F1-16 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5, F1-10.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.

## Criterios de aceptacion
- [ ] Diseño probado a mano con un lote real de ToolsaWeb
- [ ] Cumple lo que pide la tarea F1-16 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T22:31:25Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T22:40:40Z | approve | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T22:40:38Z | diseno | agente:aaf90b533374b7743 | 26 | 64483 | 568740 | 12650 | 20 | 645899 | - | - |
| 2026-11-14T22:40:39Z | diseno | agente:acee77252357bf7a7 | 24 | 41249 | 351481 | 12111 | 23 | 404865 | - | - |
| 2026-11-14T22:40:39Z | diseno | agente:a07b5e3cf1cddbf60 | 10 | 37027 | 113758 | 12992 | 8 | 163787 | - | - |
