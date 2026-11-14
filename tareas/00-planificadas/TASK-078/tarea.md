---
id: TASK-078
titulo: "F2-14 taskctl desinstalar: quitar TaskCodeTwo de forma limpia"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-078-f2-14-taskctl-desinstalar-quitar-taskcod
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
dependencias: [TASK-018, TASK-025, TASK-027]
---
## Objetivo

`taskctl desinstalar`: quita TaskCodeTwo de forma limpia (plugin, CLI, status line, hooks, worktree de la rama de estado); en un proyecto, la config solo con confirmación y nunca las tareas ni su historial sin confirmación explícita; con `--simular` (RR-16)

Tarea F2-14 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3, F2-10, F2-12.

Contexto (Carlos, 2026-11-14, RR-16): hace falta un comando para desinstalar de forma limpia tanto TaskCode (0.16) como TaskCodeTwo, sin dejar rastro en la máquina ni en los proyectos (plugin, CLI, status line, hooks, ficheros de estado, logs, caché del marketplace) y sin perder tareas ni historial salvo confirmación explícita de la persona.

## Criterios de aceptacion
- [ ] Tras desinstalar no queda nada de TaskCodeTwo en la máquina ni en el proyecto salvo lo que la persona decide conservar
- [ ] Cumple lo que pide la tarea F2-14 del plan y los requisitos que cita
