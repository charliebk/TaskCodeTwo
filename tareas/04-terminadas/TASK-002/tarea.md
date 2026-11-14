---
id: TASK-002
titulo: "F1-1 ADR carpeta fija por tarea y rama de estado"
tipo: feature
sprint: 1
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-002-f1-1-adr-carpeta-fija-por-tarea-y-rama-d
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 226256
tokens_implementacion: 3147544
tokens_revision: 1272317
creado: 2026-11-14
actualizado: 2026-11-14
dependencias: [TASK-001]
---
## Objetivo

ADR: carpeta fija por tarea con estado como campo, en rama propia `taskcode/tareas` montada como worktree en `tareas/` (P1), frente a estado en `develop`; contenido del ROADMAP

Tarea F1-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-0.

## Criterios de aceptacion
- [x] Decisión con pruebas de concepto de las dos
- [x] cómo se ve en GitHub
- [x] Cumple lo que pide la tarea F1-1 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-14T02:48:25Z | plan | automatico | persona | 0.16.1 |
| 2026-11-14T02:53:41Z | approve | automatico | persona | 0.16.1 |
| 2026-11-14T02:53:58Z | start | automatico | persona | 0.16.1 |
| 2026-11-14T03:00:28Z | review | automatico | persona | 0.16.1 |
| 2026-11-14T03:06:23Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-14T02:53:14Z | diseno | agente:aef3992f14554e6a0 | 16 | 35501 | 179008 | 11731 | 17 | 226256 | - | - |
| 2026-11-14T03:00:28Z | implementacion | sesion:f7ef1603-d233-4a21-a36b-4afb2c4b0c0e | 40 | 35155 | 3095625 | 16724 | 0 | 3147544 | - | - |
| 2026-11-14T03:05:12Z | revision | agente:a481e4ea76a12e0a3 | 24 | 115790 | 1147095 | 9408 | 1 | 1272317 | 1 | - |

## Autorrevision

- code-quality-reviewer — f4795b5

## Verificacion ligera

- 2026-11-14T03:05:43Z: ronda 1; commit revisado 6fefde2c4f2945c2736aa409299f5a5cc4d2bc91; commit verificado fab33650e9cf5144c05797c253bdfdbffdadb3be; ficheros: docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md; corregidos: MENOR-1, MENOR-2, MENOR-3; pendientes: ninguno

## Resultado

- **Implementado**: ADR [F1-1](../../../docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md) (aceptada: alternativa A, rama huerfana `taskcode/tareas` montada como worktree en `tareas/`; ROADMAP con un Mermaid por hito abierto) y `docs/adr/README.md` (indice y plantilla para F1-2..F1-17, nombres `F1-N-<tema>.md`). Guiones y log de las PoC en `poc/` de esta tarea.
- **Decisiones de la persona** (consigna: opcion recomendada): D1 repo temporal, **privado** en vez de publico porque el token de `gh` no tiene `delete_repo`; D2 nombres por F-id; D3 un Mermaid por hito; D4 alternativa A.
- **Evidencia**: PoC A y B reproducidas por el revisor (mismo resultado); GitHub: sin Actions en el push a la rama huerfana, PR imposible desde ella, diff de PR de A sin ficheros de tareas.
- **No verificado**: render de un Mermaid de 400 nodos en GitHub (repo privado sin sesion en el navegador; publicar un gist secreto fue denegado) → pasa como criterio a F3-4. Alternativa a `worktree add --orphan` para Git < 2.42 → F2-3.
- **Revision (ronda 1, aprobada con correcciones)**: 4 MENOR. Corregidos MENOR-1 (A' marcada como no probada), MENOR-2 (RN-05 matizada: `git clean -ffdx` si borra el worktree de `tareas/`; pasa a los modos de fallo de F2-3), MENOR-3 (RR-12: visibilidad si, frescura segun F1-2). Aceptado MENOR-4: se resuelve aqui.
- **Pendiente para Carlos**: borrar a mano el repo privado temporal `charliebk/taskcodetwo-poc-f1-1` (Settings → Delete), o dar permiso `delete_repo` al token (`gh auth refresh -s delete_repo`) y borrarlo con `gh repo delete charliebk/taskcodetwo-poc-f1-1 --yes`.
- Carpetas temporales de `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-002*` borradas.

## Duracion

- Estimada: 60 min
- Real (de la rama abierta al cierre con merge, sin pausas): 12 min
- Cierre (finish): 9 s, de 2026-11-14T03:06:14Z a 2026-11-14T03:06:23Z
