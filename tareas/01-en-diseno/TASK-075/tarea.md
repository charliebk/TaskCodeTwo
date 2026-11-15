---
id: TASK-075
titulo: "F1-17 ADR progreso en vivo y estimacion de tiempo"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: feature/task-075-f1-17-adr-progreso-en-vivo-y-estimacion
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
actualizado: 2026-11-15
dependencias: [TASK-010, TASK-011]
---
## Objetivo

ADR: progreso en vivo y estimación (RR-19): estimación por fase (carril, complejidad, tamaño y tareas parecidas), cuándo se recalcula, dónde se muestra y cómo sin tokens

Tarea F1-17 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9, F1-10.

Contexto (Carlos, 2026-11-14, RR-19, imprescindible): mientras se trabaja lo importante no es un porcentaje sino qué fases faltan y el tiempo estimado de cada tarea, como información para el programador, visible en el terminal de trabajo (status line de Claude Code y `taskctl status --seguir`), escrito por el CLI sin gastar tokens; en un lote, además, tareas que faltan y hora estimada de fin. Al cerrar, métricas finales: estimado frente a real por fase, tokens, rondas y hallazgos, con la calibración de la estimación en el informe por versión. Base en la 0.16: `taskctl estado [--seguir]`, la status line (fase, paso, x/N, minutos) y la «Duración estimada» del plan. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 quater.

## Criterios de aceptacion
- [ ] Estimación probada contra las tareas terminadas de ToolsaWeb y OpenDockTrading, con su error medio
- [ ] Cumple lo que pide la tarea F1-17 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T08:34:51Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T09:52:36Z | approve | automatico | persona | 0.16.1 |
