---
id: TASK-134
titulo: "F11-6 Pestana Planificacion: tareas por persona y predecesoras"
tipo: feature
sprint: 11
etiquetas: [panel, vue, planificacion]
complejidad: alta
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-134-f11-6-pestana-planificacion-tareas-por-p
asignado_a: null
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-18
actualizado: 2026-11-18
dependencias: [TASK-130, TASK-132, TASK-122]
---
## Objetivo

Vista de planificacion del equipo: que lleva cada persona, que viene despues y de que depende. Usa el grafo completo de TASK-122 (tc2 roadmap --grafo).

Tarea F11-6 del plan (docs/plan/PLAN-TASKCODE-2.md, Fase 11).

## Criterios de aceptacion
- [ ] Vista por persona con sus tareas en curso y las siguientes en el orden de tc2 next
- [ ] Cada tarea muestra sus predecesoras y sucesoras y avisa si espera a una tarea de otra persona
- [ ] Grafo de dependencias con el mismo nucleo que tc2 roadmap --grafo y resaltado del camino de una tarea
- [ ] Las tareas sin asignar aparecen aparte
