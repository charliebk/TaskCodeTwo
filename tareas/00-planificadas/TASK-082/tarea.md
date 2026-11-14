---
id: TASK-082
titulo: "F3-11 Aviso de solape entre tareas en curso"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-082-f3-11-aviso-de-solape-entre-tareas-en-cu
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-033, TASK-036]
---
## Objetivo

Aviso de solape: al tomar una tarea, `start` dice qué tareas en curso (y de quién) tocan los mismos ficheros o módulos (RR-12)

Tarea F3-11 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-6, F4-1.

Contexto (Carlos, 2026-11-14, RR-12 reforzado): el trabajo con varias personas debe estar garantizado: todos saben qué tarea tiene cada uno y no se pueden pisar ni los nombres ni las tareas. Además de lo ya previsto (IDs por push atómico, toma atómica en start, roadmap con quién lleva cada tarea, status), se cierran los huecos: nombres de rama y títulos únicos, identidad real de la plataforma (no texto libre), liberar y reasignar tareas abandonadas, aviso de solape de ficheros entre tareas en curso y estado siempre fresco con hora de sincronización. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 ter.

## Criterios de aceptacion
- [ ] Dos tareas que tocan el mismo módulo → aviso con nombres y tareas
- [ ] Cumple lo que pide la tarea F3-11 del plan y los requisitos que cita
