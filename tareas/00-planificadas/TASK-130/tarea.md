---
id: TASK-130
titulo: "F11-2 Datos del panel: tareas con estado, hito, persona, dependencias, tokens por fase y fechas"
tipo: feature
sprint: 11
etiquetas: [panel, datos]
complejidad: media
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-130-f11-2-datos-del-panel-tareas-con-estado
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
dependencias: [TASK-129]
---
## Objetivo

Implementa la capa de datos del contrato de F11-1, con el mismo nucleo situacion/grafo de tc2 roadmap y tc2 next.

Tarea F11-2 del plan (docs/plan/PLAN-TASKCODE-2.md, Fase 11).

## Criterios de aceptacion
- [ ] tc2 panel --json devuelve por tarea id, titulo, hito, estado, asignado_a, persona que la cerro (autor del commit de cierre), dependencias, tokens por fase y fechas de alta, inicio y cierre (test)
- [ ] Con una copia de ToolsaWeb da 399 tareas y 171 terminadas a 2026-11-18, como el dashboard de referencia (test)
- [ ] Lee tanto las carpetas por estado de la 0.16 como la rama de estado de la 2.0 (test)
