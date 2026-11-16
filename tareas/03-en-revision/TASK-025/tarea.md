---
id: TASK-025
titulo: "F2-10 Build de ejecutables por sistema en la CI, firmados"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-025-f2-10-build-de-ejecutables-por-sistema-e
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 571325
tokens_implementacion: 8835453
tokens_revision: 8333584
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-014, TASK-016]
---
## Objetivo

Build de ejecutables por sistema en la CI, firmados, con su hash, publicados en cada release (RR-16)

Tarea F2-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-14, F2-1.

## Criterios de aceptacion
- [ ] El ejecutable arranca en Windows, macOS y Linux limpios sin Node
- [ ] Cumple lo que pide la tarea F2-10 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T00:44:28Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T01:04:04Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T01:04:14Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T01:35:08Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T00:50:33Z | diseno | agente:adeb100acb51b0731 | 22 | 71992 | 485910 | 13401 | 24 | 571325 | - | - |
| 2026-11-16T01:35:08Z | implementacion | sesion:44e4ecae-26d8-46b9-b8f7-1cf87902cbae | 102 | 80379 | 8714711 | 40261 | 13 | 8835453 | - | - |
| 2026-11-16T01:48:12Z | revision | agente:a421c053ad6987e2e | 60 | 132495 | 3386750 | 14941 | 9 | 3534246 | 1 | rust-wasm-reviewer |
| 2026-11-16T01:48:12Z | revision | agente:a5fb13610feaf1969 | 72 | 157097 | 4613910 | 28259 | 3 | 4799338 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 7527334
- code-quality-reviewer — 7527334

## Verificacion ligera

- 2026-11-16T02:07:12Z: ronda 1; commit revisado e531238f359fa059a30904b61c3017ec947a5318; commit verificado e368253c8f54f1ac0fb73dfd1e3144e67b8bfaa4; ficheros: .github/scripts/humo.sh, .github/workflows/release.yml, docs/distribucion/releases.md, xtask/src/dist.rs, xtask/src/main.rs; corregidos: MEN-1, MEN-2, MEN-3, MEN-4, MEN-5, MEN-7, MEN-8, MEN-1, MEN-2, MEN-3, MEN-4; pendientes: ninguno
