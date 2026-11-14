---
id: TASK-014
titulo: "F1-14 ADR distribucion e instalacion"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: true
rama: feature/task-014-f1-14-adr-distribucion-e-instalacion
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 941903
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-001]
---
## Objetivo

ADR: distribución e instalación (RR-16): ejecutable por sistema (empaquetador, tamaño, arranque), firma de código en Windows y macOS, instaladores de una línea, canales (releases, winget, Homebrew, npm, marketplace), compatibilidad entre ejecutable y plugin

Tarea F1-14 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-0.

## Criterios de aceptacion
- [ ] Prueba de concepto de un ejecutable en los tres sistemas
- [ ] decisión sobre firma y canales
- [ ] Cumple lo que pide la tarea F1-14 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T17:40:08Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T17:49:18Z | approve | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T17:49:17Z | diseno | agente:a4df657c5212133fd | 38 | 66186 | 850363 | 25316 | 46 | 941903 | - | - |
