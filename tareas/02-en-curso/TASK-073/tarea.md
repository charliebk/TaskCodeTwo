---
id: TASK-073
titulo: "F1-15 ADR seleccion de skills, agentes y modelo por tarea y rol"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-073-f1-15-adr-seleccion-de-skills-agentes-y
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2497389
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-010, TASK-011]
---
## Objetivo

ADR: selección de skills, agentes y modelo por tarea y por rol (RR-17): fuentes (catálogo del proyecto, skills y agentes instalados, revisoras del plugin), reglas deterministas por etiquetas, rutas, dominio, complejidad y carril, qué se anota en la tarea y cómo se mide el acierto

Tarea F1-15 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9, F1-10.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] Reglas probadas a mano sobre 10 tareas reales de ToolsaWeb y OpenDockTrading
- [ ] nada lo decide el modelo «a ojo»
- [ ] Cumple lo que pide la tarea F1-15 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T20:39:39Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T20:49:09Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T20:49:11Z | start | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T20:49:08Z | diseno | agente:ab10be6189a7b5c59 | 56 | 142811 | 2324615 | 29907 | 48 | 2497389 | - | - |
