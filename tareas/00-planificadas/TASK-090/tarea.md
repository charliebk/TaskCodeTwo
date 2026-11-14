---
id: TASK-090
titulo: "F8-7 Metricas finales de cada tarea y de cada lote"
tipo: feature
sprint: 8
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-090-f8-7-metricas-finales-de-cada-tarea-y-de
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
dependencias: [TASK-058, TASK-083]
---
## Objetivo

Métricas finales al cerrar cada tarea y cada lote: estimado frente a real por fase, tokens, rondas y hallazgos, en el terminal y guardadas; calibración de las estimaciones en el informe por versión (RR-19)

Tarea F8-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F8-1, F3-13.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] `finish` termina con su resumen
- [ ] el informe por versión da el error medio de la estimación
- [ ] Cumple lo que pide la tarea F8-7 del plan y los requisitos que cita
