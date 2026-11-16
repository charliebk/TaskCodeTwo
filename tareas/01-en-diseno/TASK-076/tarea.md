---
id: TASK-076
titulo: "F2-4b Nombres unicos de tarea y rama"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: feature/task-076-f2-4b-nombres-unicos-de-tarea-y-rama
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
actualizado: 2026-11-16
dependencias: [TASK-019]
---
## Objetivo

Nombres únicos: rama derivada del ID comprobada también contra el remoto; título duplicado o casi igual a otro abierto → aviso con la tarea parecida (RR-12)

Tarea F2-4b del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-4.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] Dos altas con el mismo título a la vez: IDs y ramas distintos y aviso de duplicado
- [ ] Cumple lo que pide la tarea F2-4b del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T19:49:05Z | plan | automatico | persona | 0.16.1 |
