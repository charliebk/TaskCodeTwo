---
id: TASK-020
titulo: "F2-5 Modelo de tarea y parser/serializador de frontmatter YAML valido"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-020-f2-5-modelo-de-tarea-y-parser-serializad
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 325030
tokens_implementacion: 11473406
tokens_revision: 8564420
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-010]
---
## Objetivo

Modelo de tarea y parser/serializador de frontmatter YAML válido

Tarea F2-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-9.

## Criterios de aceptacion
- [ ] Ida y vuelta con un parser estándar
- [ ] compatible con tareas de la 0.16
- [ ] Cumple lo que pide la tarea F2-5 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T15:32:31Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T15:37:00Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T15:37:21Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T15:53:45Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T16:14:52Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T15:37:00Z | diseno | agente:abe862ead99e9122e | 16 | 56796 | 255414 | 12804 | 18 | 325030 | - | - |
| 2026-11-15T15:53:45Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 108 | 86332 | 11332910 | 54056 | 12 | 11473406 | - | - |
| 2026-11-15T16:14:41Z | revision | agente:a0a5ea30f94b82940 | 66 | 172227 | 4589114 | 37893 | 3 | 4799300 | 1 | rust-wasm-reviewer |
| 2026-11-15T16:23:58Z | revision | agente:acaee80f2f2678c83 | 62 | 136878 | 3605340 | 22840 | 2 | 3765120 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — edd0790
- rust-wasm-reviewer — 53c942f

## Verificacion ligera

- 2026-11-15T16:27:49Z: ronda 2; commit revisado 28ac126257f93838179e03a8e9905745575499a1; commit verificado ca5407411dbd1009fb4ae179b222905a55980ed1; ficheros: crates/tc2/src/tarea/cuerpo.rs, crates/tc2/src/tarea/yaml.rs, crates/tc2/tests/tarea.rs; corregidos: MEN-1, MEN-2; pendientes: ninguno
