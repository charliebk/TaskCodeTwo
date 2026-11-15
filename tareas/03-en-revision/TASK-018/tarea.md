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
tokens_revision: 26299473
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
| 2026-11-15T14:21:47Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T14:35:32Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T13:24:44Z | diseno | agente:ad963b707cfa50972 | 14 | 77235 | 250282 | 23500 | 15 | 351031 | - | - |
| 2026-11-15T13:52:56Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 90 | 157794 | 11517081 | 97304 | 3 | 11772269 | - | - |
| 2026-11-15T14:14:07Z | revision | agente:aaee6d3c5fc9c5e05 | 56 | 196204 | 4414663 | 28648 | 11 | 4639571 | 1 | rust-wasm-reviewer |
| 2026-11-15T14:14:07Z | revision | agente:a3be1dff0db736040 | 96 | 216195 | 8142986 | 48079 | 10 | 8407356 | 1 | code-quality-reviewer |
| 2026-11-15T14:29:30Z | revision | agente:af38693194bfe9653 | 110 | 176252 | 6457777 | 44771 | 6 | 6678910 | 2 | code-quality-reviewer |
| 2026-11-15T14:29:30Z | revision | agente:a234246fefe90c88a | 34 | 135312 | 1805947 | 23206 | 1 | 1964499 | 2 | rust-wasm-reviewer |
| 2026-11-15T14:41:31Z | revision | agente:a4bb1806bca1301ea | 50 | 143472 | 2529559 | 20797 | 2 | 2693878 | 3 | code-quality-reviewer |
| 2026-11-15T14:41:31Z | revision | agente:a07493d29b6b10a53 | 34 | 130249 | 1763848 | 21128 | 2 | 1915259 | 3 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — d78931e
- code-quality-reviewer — d78931e
- rust-wasm-reviewer — f96b878
- code-quality-reviewer — f96b878
- rust-wasm-reviewer — 9e99510
- code-quality-reviewer — 9e99510
- rust-wasm-reviewer — 965ef23
- code-quality-reviewer — 965ef23

## Rondas extra

- Ronda 3: la decide Carlos (persona) al llegar al tope: la ronda 2 abrió dos IMPORTANTES introducidos por las correcciones de la ronda 1 (IMPORTANTE-2 de calidad y R2-01 de Rust), con arreglos acotados y test.
- Ronda 4: la decide Carlos (persona) al llegar otra vez al tope: la ronda 3 abrió el IMPORTANTE-3 (un `permission denied` de la red clasificado como fallo local, introducido por la corrección de la ronda 2), confirmado con un test que falla; arreglo acotado con test, junto al MENOR R3-01.
