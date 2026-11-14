---
id: TASK-064
titulo: "F9-2 taskctl actualizar"
tipo: feature
sprint: 9
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-064-f9-2-taskctl-actualizar
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
dependencias: [TASK-018, TASK-063]
---
## Objetivo

`taskctl actualizar`: lleva `tareas/` a la rama de estado aplanando las carpetas por estado (una carpeta fija por tarea) y conservando historial, migra la config, conserva IDs, dependencias e hitos; vuelta atrás con un comando

Tarea F9-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F9-1, F2-3.

## Criterios de aceptacion
- [ ] Copia de ToolsaWeb migrada: 404 tareas, mismas dependencias, roadmap igual
- [ ] Cumple lo que pide la tarea F9-2 del plan y los requisitos que cita
