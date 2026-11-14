---
id: TASK-073
titulo: "F1-15 ADR seleccion de skills, agentes y modelo por tarea y rol"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-073-f1-15-adr-seleccion-de-skills-agentes-y
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2497389
tokens_implementacion: 13822945
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

## Decisiones de la persona

- 2026-11-14, Carlos, comprobacion a mano (D10a) de la tabla de las 10 tareas reales de F1-15 (ToolsaWeb TASK-306, 399, 403, 299, 402; OpenDockTrading TASK-063, 014, 067, 069, 096): "Coincide las 10". Las 10 propuestas (roles de diseno, revisoras y modelo por rol) coinciden con lo que habria elegido.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T20:39:39Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T20:49:09Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T20:49:11Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T21:24:48Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T22:07:30Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T20:49:08Z | diseno | agente:ab10be6189a7b5c59 | 56 | 142811 | 2324615 | 29907 | 48 | 2497389 | - | - |
| 2026-11-14T21:24:48Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 16061 | 1155072 | 3582 | 0 | 1174723 | - | - |
| 2026-11-14T21:24:48Z | implementacion | agente:af178d5c044e54716 | 124 | 335797 | 12180069 | 132232 | 16 | 12648222 | - | - |

## Autorrevision

- code-quality-reviewer — 90330c6
- code-quality-reviewer — 806f024
