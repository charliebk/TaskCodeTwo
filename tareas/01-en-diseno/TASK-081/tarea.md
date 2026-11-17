---
id: TASK-081
titulo: "F3-10 Liberar y reasignar tareas abandonadas"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-081-f3-10-liberar-y-reasignar-tareas-abandon
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-17
dependencias: [TASK-033]
---
## Objetivo

Liberar y reasignar tareas: `taskctl liberar` y `reasignar`, aviso de tareas tomadas sin actividad en N días, con aviso a quien la tenía (RR-12)

Tarea F3-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-6.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] Una tarea abandonada vuelve a «lista para empezar» sin perder su trabajo ni su rama
- [ ] Cumple lo que pide la tarea F3-10 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-17T21:53:28Z | plan | automatico | persona | 0.16.1 |
