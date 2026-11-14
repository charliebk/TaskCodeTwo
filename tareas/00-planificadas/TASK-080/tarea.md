---
id: TASK-080
titulo: "F3-9 Inventario de skills y agentes instalados"
tipo: feature
sprint: 3
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: planificada
plan_aprobado: false
rama: feature/task-080-f3-9-inventario-de-skills-y-agentes-inst
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
dependencias: [TASK-016, TASK-073]
---
## Objetivo

Inventario de skills y agentes instalados (máquina y proyecto) y `doctor` que avisa de un catálogo que nombra algo no instalado (RF-14)

Tarea F3-9 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-1, F1-15.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] Lista lo instalado
- [ ] una recomendación a algo inexistente es error con el arreglo
- [ ] Cumple lo que pide la tarea F3-9 del plan y los requisitos que cita
