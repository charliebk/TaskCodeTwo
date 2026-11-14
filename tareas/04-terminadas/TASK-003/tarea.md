---
id: TASK-003
titulo: "F1-2 ADR escrituras atomicas, reserva de IDs y toma de tareas"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-003-f1-2-adr-escrituras-atomicas-reserva-de
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 242980
tokens_implementacion: 7702714
tokens_revision: 1965316
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-002]
---
## Objetivo

ADR: escrituras atómicas, reserva de IDs y toma de tareas (P2)

Tarea F1-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1.

## Criterios de aceptacion
- [x] Protocolo de push con reintento
- [x] qué pasa sin red
- [x] Cumple lo que pide la tarea F1-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T03:06:51Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T03:10:31Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T03:10:40Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T03:44:48Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T03:58:25Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T03:10:22Z | diseno | agente:aef5cb6ada7ad952d | 16 | 43789 | 188550 | 10625 | 15 | 242980 | - | - |
| 2026-11-14T03:44:48Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 10 | 12748 | 895784 | 2637 | 0 | 911179 | - | - |
| 2026-11-14T03:44:48Z | implementacion | agente:a6bb76b046a01c68d | 96 | 185702 | 6551216 | 54521 | 16 | 6791535 | - | - |
| 2026-11-14T03:57:58Z | revision | agente:a3509ae8439189572 | 32 | 141675 | 1810179 | 13430 | 1 | 1965316 | 1 | - |

## Autorrevision

- code-quality-reviewer — 1024279

## Verificacion ligera

- 2026-11-14T03:58:02Z: ronda 1; commit revisado 98746546804291bc7e1a18cc2fc2616a84dc8cd1; commit verificado bd7f2d1d663113637677385f5873755b40035870; ficheros: docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md; corregidos: MEN-1, MEN-2, MEN-3; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-2](../../../docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md) (aceptada) y su fila en `docs/adr/README.md`. Transaccion optimista sobre `taskcode/tareas`: cerrojo local, `fetch`, operacion sobre el estado fresco, commit y push sin force; ante el rechazo non-fast-forward se descarta el commit y se **re-ejecuta** la operacion (nunca rebase). IDs `TASK-NNN` = maximo + 1 dentro de la operacion; `import` en un solo push; `start` escribe `asignado` antes de crear rama. Sin red: compartidas devuelven `SinRed`, las de la tarea propia quedan pendientes. Codigos propuestos 0/3/4/5. Guion y log en `poc/`.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a re-ejecutar, D2b sin red solo para la tarea propia, D3a 5 intentos con espera exponencial y jitter, D4a `TASK-NNN` consecutivo.
- **Divergencia con el plan** (documentada en el ADR): con la base de espera de 0,1 s una alta de 12 acababa en `Agotada`; el implementador subio la base a 2 s dentro de D3a (era un parametro). No elimina las `Agotada` bajo rafaga; se revisa en F2-3 y F9-4. D1a sustituye el rebase que decian P2 y F1-1.
- **No verificado**: pushes simultaneos contra GitHub/GitLab y entre maquinas (F9-4), duracion con el CLI real y el cerrojo local con varios procesos (F2-3).
- **Revision (ronda 1, aprobada con correcciones)**: 3 MENOR, los tres corregidos (MEN-1 un rechazo de hook cae en `Agotada`, no en «sin red»; MEN-2 D1a sustituye el rebase de P2/F1-1; MEN-3 muestra pequena y espera maxima de ~30 s). Sin pendientes.
- Implementacion por un agente implementador; revision por otro agente independiente. Temporales de `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-003*` borrados.

## Duracion

- Estimada: 45 min
- Real (de la rama abierta al cierre con merge, sin pausas): 48 min
- Cierre (finish): 8 s, de 2026-11-14T03:58:17Z a 2026-11-14T03:58:25Z
