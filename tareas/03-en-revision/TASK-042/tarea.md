---
id: TASK-042
titulo: "F5-2 Cache por SHA del arbol compartida"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-042-f5-2-cache-por-sha-del-arbol-compartida
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 293610
tokens_implementacion: 14727181
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-041]
---
## Objetivo

Caché por SHA del árbol compartida por PR, cola de merge y release

Tarea F5-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F5-1.

## Criterios de aceptacion
- [ ] Cero repeticiones sobre el mismo SHA, medido
- [ ] Cumple lo que pide la tarea F5-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T19:19:46Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T20:06:12Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T20:06:14Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T20:59:49Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T21:05:32Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T20:06:12Z | diseno | agente:a1c0b5a7a377327c4 | 16 | 55667 | 225327 | 12600 | 15 | 293610 | - | - |
| 2026-11-19T20:59:49Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 20 | 13796 | 3904494 | 4611 | 1 | 3922921 | - | - |
| 2026-11-19T20:59:49Z | implementacion | agente:a446c748ef3c41da7 | 128 | 211612 | 10520406 | 72114 | 15 | 10804260 | - | - |

## Revisores no relanzados

- ronda 2: rust-wasm-reviewer - solo tenia hallazgos MENOR pendientes y la correccion no toca sus ficheros
