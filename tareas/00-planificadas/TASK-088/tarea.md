---
id: TASK-088
titulo: "F4-9 Progreso del lote en el terminal"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-088-f4-9-progreso-del-lote-en-el-terminal
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
dependencias: [TASK-084, TASK-086]
---
## Objetivo

Progreso del lote: tarea actual, tareas que faltan, apartadas y hora estimada de fin (RR-19)

Tarea F4-9 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-7, F3-12.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] En un lote de 5 tareas se ve cuántas faltan y cuándo acabará
- [ ] Cumple lo que pide la tarea F4-9 del plan y los requisitos que cita
