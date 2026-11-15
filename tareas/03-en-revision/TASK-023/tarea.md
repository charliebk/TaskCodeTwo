---
id: TASK-023
titulo: "F2-7 Configuracion minima con esquema validado y valores por defecto"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-023-f2-7-configuracion-minima-con-esquema-va
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1976052
tokens_implementacion: 10920973
tokens_revision: 20259729
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-011]
---
## Objetivo

Configuración mínima con esquema validado y valores por defecto

Tarea F2-7 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-10.

## Criterios de aceptacion
- [ ] ≤ 12 claves
- [ ] esquema publicado
- [ ] tests
- [ ] Cumple lo que pide la tarea F2-7 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T21:13:01Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T21:23:05Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T21:23:08Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T21:41:24Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T22:06:51Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T21:22:48Z | diseno | agente:a24b1e29bef7bcb47 | 56 | 111726 | 1835655 | 28615 | 51 | 1976052 | - | - |
| 2026-11-15T21:41:24Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 104 | 112784 | 10748806 | 59279 | 26 | 10920973 | - | - |
| 2026-11-15T22:06:39Z | revision | agente:aa511526881b78cf4 | 126 | 189621 | 9443208 | 34022 | 14 | 9666977 | 1 | rust-wasm-reviewer |
| 2026-11-15T22:06:39Z | revision | agente:add420c2a5a24921f | 94 | 172612 | 6216763 | 29281 | 7 | 6418750 | 1 | code-quality-reviewer |
| 2026-11-15T22:15:02Z | revision | agente:a693fd6dabd87d706 | 28 | 131593 | 1427784 | 20660 | 2 | 1580065 | 2 | rust-wasm-reviewer |
| 2026-11-15T22:15:02Z | revision | agente:a7ca65a90c98fa2da | 42 | 152779 | 2403853 | 37263 | 5 | 2593937 | 2 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 29cf97f
- code-quality-reviewer — 29cf97f
- rust-wasm-reviewer — 8671970
- code-quality-reviewer — 8671970

## Verificacion ligera

- 2026-11-15T22:15:29Z: ronda 2; commit revisado 964898f7d7e9298ce912ae2a54c2b8b39e88f998; commit verificado 0fdb170ded4cce845ffbfcc87bc512a68bb52b9d; ficheros: crates/tc2/src/config/validar.rs, crates/tc2/tests/config.rs; corregidos: MEN-6, MEN-7; pendientes: ninguno
