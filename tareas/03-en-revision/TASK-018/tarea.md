---
id: TASK-018
titulo: "F2-3 Rama de estado"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-018-f2-3-rama-de-estado
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 351031
tokens_implementacion: 11772269
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-002, TASK-003, TASK-017]
---
## Objetivo

Rama de estado: crear, montar como worktree en `tareas/`, leer y escribir de forma atómica (P1, P2)

Tarea F2-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-2, F2-2.

## Criterios de aceptacion
- [x] Dos procesos escriben a la vez sin perder nada (`tests/estado_concurrencia.rs`: dos clones y dos procesos del mismo clon)
- [x] test (`cargo xtask verificar --perfil completa` en verde: 33 tests de tc2)
- [x] Cumple lo que pide la tarea F2-3 del plan y los requisitos que cita (`tests/estado_casos.rs`; adendas en F1-1, F1-2 y F1-4)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T13:18:19Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T13:24:44Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T13:24:46Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T13:52:56Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T13:24:44Z | diseno | agente:ad963b707cfa50972 | 14 | 77235 | 250282 | 23500 | 15 | 351031 | - | - |
| 2026-11-15T13:52:56Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 90 | 157794 | 11517081 | 97304 | 3 | 11772269 | - | - |

## Autorrevision

- rust-wasm-reviewer — d78931e
- code-quality-reviewer — d78931e
