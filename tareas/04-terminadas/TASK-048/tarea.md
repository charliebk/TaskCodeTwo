---
id: TASK-048
titulo: "F6-2 review: abre el PR contra develop con la identidad del agente"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-048-f6-2-review-abre-el-pr-contra-develop-co
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 917276
tokens_implementacion: 20892658
tokens_revision: 3553700
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-021, TASK-047]
---
## Objetivo

`review`: abre el PR contra `develop` con la identidad del agente y lanza el revisor

Tarea F6-2 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-1, F2-6.

## Criterios de aceptacion
- [x] PR enlazado a la tarea
- [x] check `revision-ia`
- [x] Cumple lo que pide la tarea F6-2 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T12:41:12Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T12:45:59Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T12:46:00Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T13:39:25Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T14:18:06Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T12:45:56Z | diseno | agente:ada29a9245fa1f565 | 34 | 81201 | 820930 | 15111 | 28 | 917276 | - | - |
| 2026-11-19T13:39:25Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 32 | 17280 | 3954277 | 7167 | 1 | 3978756 | - | - |
| 2026-11-19T13:39:25Z | implementacion | agente:a12ad75258e6880c9 | 152 | 292318 | 16546206 | 75226 | 23 | 16913902 | - | - |
| 2026-11-19T13:46:48Z | revision | agente:a83193e1d96820945 | 26 | 140026 | 1431606 | 7771 | 13 | 1579429 | 1 | rust-wasm-reviewer |
| 2026-11-19T13:46:48Z | revision | agente:ad61b88e473465765 | 34 | 137675 | 1825994 | 10568 | 3 | 1974271 | 1 | code-quality-reviewer |

## Resultado

`tc2 review TASK-NNN` (comando de fase nuevo, `crates/tc2/src/revision/`): fetch, regla de `start` para quien lo lanza, exige base `develop` (`hotfix/` sale `rechazada motivo=carril-no-soportado`, D3) y worktree; push desde el worktree con la identidad del agente (`Plataforma::entorno_push` + `Git::derivado`: cabecera limitada a la URL https del remoto, token nunca en argv ni en el registro); `abrir_pr(rama, "develop", "TASK-NNN: <titulo>", cuerpo_pr)` con las rutas de la tarea y del plan; solo despues `en-curso` -> `en-revision` por transaccion; lee `revision-ia` con el agente. Linea `review ok id= rama= pr= estado=en-revision revision-ia=<ok|pendiente|fallo|sin-revisor|sin-leer>`; repetir converge a `sin-cambios`. Adenda F6-2 en el ADR F1-6.

Evidencia:
- Verificacion completa (`cargo xtask verificar --perfil completa`) en verde sobre a30688c; `finish` vuelve a verificar.
- E2E reales (2026-11-19): `review_abre_el_pr_del_agente_contra_develop` en GitHub (PR #8 abierto por la App del agente contra develop, repetir converge, `revision-ia=pendiente`) y `review_abre_el_mr_del_agente_contra_develop` en GitLab (MR !12). Ambos limpian PR/MR y rama. El primer intento de GitLab fallo por una asercion del test: GitLab guarda la descripcion sin el salto final (corregido en a30688c).
- Tests de integracion con remoto bare y plataforma doble: push, PR, transicion y convergencia; el push lleva `Authorization: Basic` del agente (mutacion: sin el entorno, rojo).

Criterio `revision-ia` segun D1 (A): `review` lo lee e informa; no sale en verde hasta que el revisor lo publique en F6-3 (TASK-049). Divergencia anotada en la adenda.

Revision por pares, ronda 1: rust-wasm-reviewer aprobada (MEN-1 pushurl distinto sin cabecera, MEN-2 sin test que adelante la transaccion al PR) y code-quality-reviewer aprobada (MENOR-1 rechazos de `review_en` sin test de integracion). Los tres aceptados con motivo en sus informes; son seguimiento.

Divergencias (adenda F6-2): `Git` no es componible por peticion, resuelto con `Git::derivado`; `estado_puertas` no expone un check suelto, el estado de `revision-ia` se deduce de `falta=`/`checks=`; la cabecera va a la URL de fetch de `origin`; el E2E llama a `revision::publicar` y no a `review_en` completo para no escribir en `taskcodetwo/tareas` del repo de pruebas.

## Duracion

- Estimada: 90 min
- Real (de la rama abierta al cierre con merge, sin pausas): 92 min
- Paso de la hora: 92 min reales frente a 90 estimados
- Cierre (finish): 1838 s, de 2026-11-19T13:47:28Z a 2026-11-19T14:18:06Z
