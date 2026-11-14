---
id: TASK-007
titulo: "F1-6 ADR identidades del agente, del revisor y de la persona"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-007-f1-6-adr-identidades-del-agente-del-revi
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 461783
tokens_implementacion: 5339774
tokens_revision: 7743763
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-006]
---
## Objetivo

ADR: identidades del agente implementador, del revisor IA y de la persona (P3)

Tarea F1-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5.

## Criterios de aceptacion
- [x] Permisos mínimos
- [x] que ninguna IA pueda aprobar ni mergear
- [x] Cumple lo que pide la tarea F1-6 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T06:26:21Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T06:29:57Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T06:29:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T06:39:54Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T06:53:53Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T07:05:58Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T06:29:56Z | diseno | agente:aad4a39e360deeb1e | 24 | 54426 | 394421 | 12912 | 16 | 461783 | - | - |
| 2026-11-14T06:39:54Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 8 | 9536 | 1116619 | 2166 | 0 | 1128329 | - | - |
| 2026-11-14T06:39:54Z | implementacion | agente:a4dce7ab31254dc11 | 62 | 186387 | 3977407 | 47589 | 13 | 4211445 | - | - |
| 2026-11-14T06:45:56Z | revision | agente:a6d30e739c73e979e | 54 | 185632 | 3740093 | 34539 | 3 | 3960318 | 1 | - |
| 2026-11-14T07:00:07Z | revision | agente:a1caf9920cf80c601 | 46 | 207381 | 3522867 | 53151 | 6 | 3783445 | 2 | - |

## Autorrevision

- code-quality-reviewer — c751784
- code-quality-reviewer — ed9f807

## Verificacion ligera

- 2026-11-14T07:05:49Z: ronda 2; commit revisado 3e5bc5759cbe1e6ea885006723839294d66bb540; commit verificado 179771280fc3543e804e45dacc4813298f8f66c9; ficheros: docs/adr/F1-6-identidades-agente-revisor-persona.md; corregidos: MEN-9; pendientes: MEN-7, MEN-8

## Resultado

- **Implementado**: ADR [F1-6](../../../docs/adr/F1-6-identidades-agente-revisor-persona.md) (aceptada) y su fila en `docs/adr/README.md`. Tabla operacion → identidad sobre la lista cerrada del puerto `Plataforma` (proveedor de credenciales por rol `agente | revisor | persona`; nueva `leer-permisos-actions`, solo de la persona). Permisos minimos: agente (App) `contents:write`, `pull_requests:write`, `checks:read`, `statuses:read`, `metadata:read`, sin `administration`, `workflows` ni `checks:write`; revisor (otra App) `checks:write` y lecturas, sin poder hacer reviews ni merge; persona con su cuenta y clave de firma fuera del alcance del agente. Contrato del firmante (firma verificada, `User`, en la lista de `.taskcode/` por `id` numerico, no `web-flow`) y regla 6 de reviews. «Ninguna IA aprueba ni mergea» en tres capas, con el limite escrito: `contents:write` incluye merge, el agente puede aprobar PR ajenos y crear tags/releases (F1-10; `repository_dispatch` a F1-8). El guion lee las tablas del ADR entre marcadores y las contrasta con la fuente de permisos por endpoint de `github/docs`.
- **Decisiones de la persona** (consigna: opcion recomendada): D1a, D2a dos Apps, D3a, D4a FIDO (o (b) sin llave), D5a revisor en Actions, D6a.
- **Divergencias** (en el ADR): `abrir-pr` en vez de `crear-pr`/`actualizar-pr`; permisos por endpoint en `github/docs`, no en OpenAPI; `web-flow` es `User` y se excluye; el revisor no comenta en el PR (F6-1 debe actualizarse); y las de la ronda 1.
- **No verificado**: permisos efectivos de Apps reales, si GitHub cuenta la aprobacion de una App, merge real con `contents:write`, push de workflows sin permiso, la marca «Verified», llave FIDO real, GitLab, interfaz.
- **Revision**: ronda 1 cambios (IMP-1 `estado-puertas` sin `checks:read`, IMP-2 `can_approve` exige `administration:read`, IMP-3 el guion no leia el ADR; MEN-1..6), todo corregido; ronda 2 aprobada con correcciones: MEN-7 (todas las operaciones de cada fila), MEN-8 (codigo de salida), MEN-9 (`doctor` sin token de persona) corregidos.
- Implementacion por un agente implementador; revision por dos agentes independientes. `taskctl doctor` con 0 errores. Temporales borrados.

## Duracion

- Estimada: 60 min
- Real (de la rama abierta al cierre con merge, sin pausas): 36 min
- Cierre (finish): 7 s, de 2026-11-14T07:05:51Z a 2026-11-14T07:05:58Z
