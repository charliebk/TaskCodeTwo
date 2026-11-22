---
id: TASK-085
titulo: "F4-6 plan propone skills, agentes y modelo por rol"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-085-f4-6-plan-propone-skills-agentes-y-model
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 4322228
tokens_implementacion: 64605497
tokens_revision: 10946991
creado: 2026-11-14
actualizado: 2026-11-22
dependencias: [TASK-036, TASK-049, TASK-080]
---
## Objetivo

`plan` propone skills, agentes y modelo para diseño, implementación y revisión; se anotan en la tarea; el orquestador y la revisión los usan (RF-12, RF-13)

Tarea F4-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F4-1, F3-9, F6-3.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] En 10 tareas reales la propuesta coincide con la que haría una persona
- [ ] el modelo cambia según complejidad y carril
- [ ] Cumple lo que pide la tarea F4-6 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-22T17:13:20Z | plan | automatico | persona | 0.16.1 |
| 2026-11-22T17:19:30Z | approve | automatico | persona | 0.16.1 |
| 2026-11-22T17:19:31Z | start | automatico | persona | 0.16.1 |
| 2026-11-22T18:43:47Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-22T17:19:29Z | diseno | agente:a4af724a5e04d173f | 58 | 199960 | 4097817 | 24393 | 1 | 4322228 | - | - |
| 2026-11-22T18:43:47Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 34 | 23126 | 6411833 | 8631 | 0 | 6443624 | - | - |
| 2026-11-22T18:43:47Z | implementacion | agente:ab73915092e08af44 | 362 | 408808 | 57599241 | 153462 | 65 | 58161873 | - | - |
| 2026-11-22T19:03:54Z | revision | agente:a2e01d7a4a39a342a | 92 | 218701 | 7855057 | 27288 | 4 | 8101138 | 1 | rust-wasm-reviewer |
| 2026-11-22T19:03:54Z | revision | agente:ad3cafaa4f13575f2 | 44 | 154328 | 2672568 | 18913 | 2 | 2845853 | 1 | code-quality-reviewer |
