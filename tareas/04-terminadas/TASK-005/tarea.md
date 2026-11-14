---
id: TASK-005
titulo: "F1-4 ADR capa de operaciones Git/GitHub deterministas y su contrato"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-005-f1-4-adr-capa-de-operaciones-git-github
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 682282
tokens_implementacion: 7556598
tokens_revision: 7233669
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-004]
---
## Objetivo

ADR: capa de operaciones Git/GitHub deterministas y su contrato de salida para el agente (P5)

Tarea F1-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-3.

## Criterios de aceptacion
- [x] Lista cerrada de operaciones
- [x] objetivo de tokens por tarea
- [x] Cumple lo que pide la tarea F1-4 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T04:50:35Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T04:54:31Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T04:54:32Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T05:13:55Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T05:38:30Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T05:44:53Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T04:54:30Z | diseno | agente:a52066c5d1c1952ea | 30 | 72705 | 596577 | 12970 | 22 | 682282 | - | - |
| 2026-11-14T05:13:55Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9865 | 889356 | 2466 | 0 | 901695 | - | - |
| 2026-11-14T05:13:55Z | implementacion | agente:ad78d29441f54bf07 | 80 | 226819 | 6358323 | 69681 | 17 | 6654903 | - | - |
| 2026-11-14T05:24:17Z | revision | agente:a7436ff55cb219908 | 50 | 178256 | 3296140 | 35581 | 8 | 3510027 | 1 | - |
| 2026-11-14T05:44:45Z | revision | agente:ad47ea606ebbfe7ad | 58 | 165841 | 3534167 | 23576 | 9 | 3723642 | 2 | - |

## Autorrevision

- code-quality-reviewer — 1965931
- code-quality-reviewer — 9000b48

## Resultado

- **Implementado**: ADR [F1-4](../../../docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md) (aceptada) y su fila en `docs/adr/README.md`. Capa interna del CLI con dos puertos (`Git` con entorno fijo y salida porcelana; `Plataforma` por API HTTP, sin `gh`/`glab`); lista cerrada de 14 operaciones locales mas `commit-estado` y 3 de plataforma, cada una con comando dueno e idempotencia; operaciones prohibidas (`rebase`, `push --force`, `reset --hard`, `stash`, `clean`, merge local en `develop`/`main`, borrar ramas) comprobadas con un detector por lista permitida; contrato de salida de como mucho 5 lineas con primera linea `comando resultado clave=valor`; codigos 0..7. Objetivo de tokens: 0 llamadas `git`/`gh` de escritura o red del agente, salida visible de operaciones ≤ 2 KB por tarea y tokens totales por debajo de la 0.16. Guion y log en `poc/`.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a, D2a, D3a, D4a, D5a.
- **Linea base medida** (transcripciones de TASK-002..004): 17/14/26 llamadas `git`/`gh`, la salida de Git pesa el 1,4-3,6 % de los tokens de la tarea.
- **Divergencias con el plan** (en el ADR): la capa sola no basta para bajar los tokens totales (lo recoge F8-2); la PoC de F1-2 usaba `reset`, F1-4 lo sustituye por `commit-tree` + `push <sha>:ref`; el hook de D4a debe limitarse al repo del proyecto; la prueba de locale pasa de forma trivial (Git for Windows sin traducciones).
- **No verificado**: puerto `Plataforma` (sin repo real), Git con traducciones y otras versiones, conversion bytes→tokens, el hook de D4a, `--json`, codigos 1, 5 y 7, `commit-estado` agotada, pendientes sin red de F1-2 D2b (propuesta sin reset en `refs/taskcode/pendientes/NNN`).
- **Revision**: ronda 1 cambios solicitados: IMP-1 (excepcion de `reset --hard` innecesaria → `commit-tree`, probado con dos escritores), IMP-2 (`tag-anotado` con tag en otro commit → 3), IMP-3 (cobertura declarada de mas → caminos ejercitados o «No verificado»), MEN-1 (lista permitida en vez de grep), MEN-2 (cuenta de operaciones), todos corregidos. Ronda 2 aprobada sin hallazgos nuevos. Notas no bloqueantes aceptadas: un push que falla tras un `fetch` correcto devuelve `agotada` (5) en vez de `sinred` (4), para distinguirlo en F2-3; el grep de estado asume LF.
- Implementacion por un agente implementador; revision por otros dos agentes independientes. Temporales borrados.

## Duracion

- Estimada: 55 min
- Real (de la rama abierta al cierre con merge, sin pausas): 50 min
- Cierre (finish): 7 s, de 2026-11-14T05:44:46Z a 2026-11-14T05:44:53Z
