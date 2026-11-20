---
id: TASK-049
titulo: "F6-3 Revisor en CI con las skills de dominio de la 0.16"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 998648
tokens_implementacion: 45593290
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-20
dependencias: [TASK-044, TASK-048]
---
## Objetivo

Revisor en CI con las skills de dominio de la 0.16 (angular-vue, rust-wasm, C#, Java, genérica), enrutado por lo que toca el diff (RR-02)

Tarea F6-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-2, F5-4.

Contexto (Carlos, 2026-11-14, RR-17, imprescindible): para cada tarea y cada rol (diseño, implementación, revisión) hay que elegir las skills y los agentes adecuados y el modelo según complejidad y carril, solo entre lo que está instalado en la máquina y el proyecto, anotar la elección en la tarea y medir si acierta (hallazgos por revisora y coste por modelo). En la 0.16 existe con `catalogo-skills.yml`, `skills_recomendados` y `modelo_sugerido`, pero se usó en 21 / 0 / 2 tareas de ToolsaWeb / OpenDockTrading / TaskCode porque el catálogo apenas se configuró: hoy la elección la hace el agente sin criterio registrado. Nada debe decidirlo el modelo «a ojo»: reglas deterministas. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §1–4.

## Criterios de aceptacion
- [ ] Un diff Rust + Vue lanza las dos de dominio
- [ ] Cumple lo que pide la tarea F6-3 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T21:59:15Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T22:23:19Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T22:23:21Z | start | automatico | persona | 0.16.1 |
| 2026-11-20T00:26:36Z | review | automatico | persona | 0.16.1 |
| 2026-11-20T13:03:28Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T22:23:18Z | diseno | agente:a7d15d7dc5efda9d6 | 40 | 71750 | 909469 | 17389 | 32 | 998648 | - | - |
| 2026-11-20T00:26:36Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 46 | 68052 | 10530571 | 10298 | 4 | 10608967 | - | - |
| 2026-11-20T00:26:36Z | implementacion | agente:a26939dbbe09eb062 | 248 | 422570 | 34426743 | 134762 | 68 | 34984323 | - | - |
