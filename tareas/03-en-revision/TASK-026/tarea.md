---
id: TASK-026
titulo: "F2-11 Instaladores de una linea"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-026-f2-11-instaladores-de-una-linea
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 352073
tokens_implementacion: 23843522
tokens_revision: 10199339
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-025]
---
## Objetivo

Instaladores de una línea (PowerShell y shell) que descargan, verifican la firma o el hash y ponen `taskctl` en el PATH

Tarea F2-11 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-10.

## Criterios de aceptacion
- [ ] Instalación en una máquina limpia de cada sistema en menos de un minuto
- [ ] Cumple lo que pide la tarea F2-11 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T08:55:05Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T09:18:26Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T09:18:38Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T11:00:28Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T11:26:17Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T11:49:49Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T09:00:18Z | diseno | agente:a5d80e07e9effb23b | 16 | 66714 | 268770 | 16573 | 21 | 352073 | - | - |
| 2026-11-16T11:00:28Z | implementacion | sesion:bb6e0871-29bf-42e4-b8be-661dbe95cd05 | 200 | 193055 | 23542909 | 107358 | 12 | 23843522 | - | - |
| 2026-11-16T11:06:37Z | revision | agente:affebe3dec23302b7 | 20 | 112325 | 919266 | 7836 | 2 | 1039447 | 1 | rust-wasm-reviewer |
| 2026-11-16T11:06:37Z | revision | agente:a08316f48c9d20785 | 28 | 142626 | 1586182 | 19765 | 1 | 1748601 | 1 | code-quality-reviewer |
| 2026-11-16T11:35:39Z | revision | agente:a60563144f58d1cf8 | 86 | 178083 | 6171117 | 55621 | 9 | 6404907 | 2 | code-quality-reviewer |
| 2026-11-16T11:35:39Z | revision | agente:a78d78d01a8598de9 | 20 | 108244 | 889801 | 8319 | 1 | 1006384 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — f309ec2
- code-quality-reviewer — f309ec2
- rust-wasm-reviewer — dedddf7
- code-quality-reviewer — dedddf7
- code-quality-reviewer — 0274adf

## Rondas extra

- Ronda 3: la persona (Carlos), en el chat de la sesion, para corregir R2-1 (perdida de lineas del perfil del shell que introdujo la correccion de IMP-2) antes de integrar.
