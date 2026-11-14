---
id: TASK-084
titulo: "F3-12 Progreso en vivo: fases que faltan y tiempo restante"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-084-f3-12-progreso-en-vivo-fases-que-faltan
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
dependencias: [TASK-018, TASK-083]
---
## Objetivo

Progreso en vivo en el terminal: fase actual, **fases que faltan** y **tiempo estimado restante**, en la status line de Claude Code y en `taskctl status --seguir`, escrito por el CLI (RR-19)

Tarea F3-12 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-13, F2-3.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] Durante una tarea real se ven las fases que faltan y el tiempo restante, actualizados en cada fase, sin coste de tokens
- [ ] Cumple lo que pide la tarea F3-12 del plan y los requisitos que cita
