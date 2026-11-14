---
id: TASK-004
titulo: "F1-3 ADR flujo de ramas develop/main, rama por tarea, worktrees, release"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-004-f1-3-adr-flujo-de-ramas-develop-main-ram
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 565895
tokens_implementacion: 6258767
tokens_revision: 4231336
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-001]
---
## Objetivo

ADR: flujo de ramas `develop`/`main`, rama por tarea, worktrees, release y hotfix (P4)

Tarea F1-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-0.

## Criterios de aceptacion
- [x] Nombres, ciclo completo, qué queda de Git-Flow
- [x] Cumple lo que pide la tarea F1-3 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T03:58:59Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T04:02:50Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T04:02:52Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T04:14:14Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T04:43:39Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T04:50:25Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T04:02:49Z | diseno | agente:ab0cf0508f7832d0e | 26 | 65103 | 488333 | 12433 | 25 | 565895 | - | - |
| 2026-11-14T04:14:14Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9821 | 779331 | 2217 | 0 | 791377 | - | - |
| 2026-11-14T04:14:14Z | implementacion | agente:aa4dd1d805c5a1533 | 74 | 157307 | 5262396 | 47613 | 18 | 5467390 | - | - |
| 2026-11-14T04:20:36Z | revision | agente:ad3141abac661826c | 30 | 153447 | 1793435 | 19947 | 11 | 1966859 | 1 | - |
| 2026-11-14T04:50:15Z | revision | agente:a9beee2def15847a6 | 30 | 177984 | 2056489 | 29974 | 9 | 2264477 | 2 | - |

## Autorrevision

- code-quality-reviewer — bba1cd5
- code-quality-reviewer — 0bdc9d8

## Resultado

- **Implementado**: ADR [F1-3](../../../docs/adr/F1-3-flujo-de-ramas-worktrees-release.md) (aceptada) y su fila en `docs/adr/README.md`. Ramas `feature|fix|hotfix/task-NNN-<slug>` (unicidad por ID con `*/task-NNN-*` en remoto y en local); worktrees de tarea en `.taskcode/worktrees/TASK-NNN` que escriben el estado en `tareas/` del principal por el punto unico de F1-2; release sin rama `release/*` (PR `develop`→`main`, tag anotado en el merge); hotfix desde el ultimo tag (o la punta de `main` sin tags) con PR de vuelta a `develop`; merge commit y ramas conservadas. Tabla de los 22 scripts de Git-Flow de la 0.16 con su destino (comando 2.0 o retirado). Guion y log en `poc/`.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a, D2a, D3a, D4a.
- **Divergencias con el plan** (documentadas en el ADR): las notas de release omiten los merges cuyo segundo padre ya esta en el tag anterior (el PR de vuelta del hotfix se repetia); el calculo de version queda para F7-3; precisiones de la revision 1.
- **No verificado**: metodo de merge y cola de GitHub/GitLab (F1-5, F9-4), permisos de Claude Code en `.taskcode/worktrees/`, herramientas que ven worktrees anidados, rutas profundas en Windows (la PoC llego a ~160 caracteres sin problema), escrituras concurrentes desde varios worktrees, tags simultaneos.
- **Revision**: ronda 1 cambios solicitados: IMPORTANTE-1 (`start` idempotente frente a rechazo por rama existente: ahora reanuda si la lleva quien lanza y rechaza si es de otro, PoC 1f) y MENOR-1 (hotfix sin tags, PoC 1g), MENOR-2 (ID tambien en local), MENOR-3 (patron `*/task-007-*`, PoC 1e), todos corregidos. Ronda 2 (verificacion acotada) aprobada, sin hallazgos nuevos. Notas no bloqueantes de la ronda 2 aceptadas sin accion: base invalida al reanudar (sin caso real), la rama recreada desde la base (decision 2c) se ejercito en la revision pero no esta en el log, tracking distinto en (c), el guion imprime RESULTADO en vez de abortar.
- Implementacion por un agente implementador; revision por otros dos agentes independientes. Temporales borrados.

## Duracion

- Estimada: 55 min
- Real (de la rama abierta al cierre con merge, sin pausas): 48 min
- Cierre (finish): 8 s, de 2026-11-14T04:50:17Z a 2026-11-14T04:50:25Z
