---
id: TASK-137
titulo: "F6-3 seguimiento: revision-ia cita el informe del run de origen cuando la verificacion sale de cache"
tipo: fix
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-137-f6-3-seguimiento-revision-ia-cita-el-inf
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: null
tokens_implementacion: 8463850
tokens_revision: null
creado: 2026-11-20
actualizado: 2026-11-20
dependencias: []
---
## Objetivo

Cuando verificacion reutiliza un perfil de la cache por arbol (F5-2), su run no sube informe-verificacion-<perfil> y revision-ia.yml (job preparar) lo busca solo en ese run: el check sale con informe=sin-informe y las revisoras van sin la seccion del informe (F5-4). Se cierra el hueco citando el informe del run de origen. Descubierto en el E2E de TASK-049 (run de verificacion 44740343425 reutilizado, run de revision-ia 44740402515 con informe=sin-informe).

## Criterios de aceptacion
- [x] Un PR cuya verificacion sale de cache produce prompts con la seccion del informe del run de origen, probado en charliebk/taskcodetwo-e2e-github (un PR que ejecuta la verificacion y un commit vacio que la reutiliza)
- [x] Si el informe del origen ha caducado o no se encuentra, el paso `Informe de verificacion` de `.github/workflows/revision-ia.yml` lo avisa (`::warning::`), el resumen del check lleva `informe=sin-informe` y las revisoras van sin seccion, como hoy
- [x] docs/ci/revision-ia.md (Limites conocidos) actualizado
- [x] El lint de xtask/src/workflows.rs sigue en verde

## Resultado

El paso `Informe de verificacion` del job `preparar` de `.github/workflows/revision-ia.yml` (alternativa 1 del plan, aprobada por Carlos) ya no se queda sin informe cuando el run de `verificacion` reutiliza el perfil de la cache por arbol (F5-2): si ese run no tiene `informe-verificacion-<perfil>`, toma el job del perfil, lee en su log (con `curl`) la ultima linea `Perfil <p>: reutilizada de <sha> (origen <sha>, ...)`, lista los runs en verde del mismo workflow con `head_sha` = origen y cita el mas reciente con el artefacto sin caducar. La seccion nombra el artefacto con su run y su origen y el resumen del check anade `informe-de=run-<id>`. Sin linea: `sin artefacto ... ni linea de reutilizacion` (como hoy). Con origen pero sin artefacto (caducado): `::warning::` e `informe=sin-informe`. Sin cambios de permisos, de llamadores, del lint ni del formato de la cache; `verificacion.yml` solo gana un comentario sobre la linea de log de la que depende. Guia: seccion nueva «Informe de una verificacion reutilizada» y limite nuevo en `docs/ci/revision-ia.md`.

Evidencia (repo E2E `charliebk/taskcodetwo-e2e-github`, PR 15, verificacion fijada a f22cc95d y revision-ia a la rama de la tarea):
- Run de verificacion 44760343474 (b7a4186a) ejecuta y sube `informe-verificacion-completa` (333 bytes); su revision-ia 44760410351 da `informe=verde informe-de=run-44760343474`.
- Commit vacio 1957395c: verificacion 44760498241 reutiliza (log `reutilizada de b7a4186a... (origen b7a4186a...`), sin artefactos. Con el primer commit de la tarea (b33fe3a) su revision-ia 44760584702 salio `informe=sin-informe`: el `gh` del runner se niega a volcar el log («the response contains terminal escape sequences; pass --allow-escape-sequences»), opcion que el gh 2.95 local no conoce. Arreglado con `curl` (a6aafa5).
- Segundo commit vacio 3b912e94: verificacion 44761381047 reutiliza en cadena (`reutilizada de 1957395c... (origen b7a4186a...`); su revision-ia 44761462738 (a6aafa5) cita el informe del run 44760343474: resumen `informe=verde informe-de=run-44760343474` y prompt con `## Informe de verificacion (F5-4)` y «Artefacto `informe-verificacion-completa del run 44760343474 (verificacion reutilizada por la cache por arbol; origen b7a4186a...)`». Las revisoras dan 400 por la cuenta sin saldo (conocido, no afecta).
- Caso sin artefacto en el origen: el script del paso, extraido del workflow y ejecutado en local contra el run 44740343425 con la busqueda de artefactos anulada, da el `::warning::` y no deja `informe.json`; contra un perfil sin job (`rapida`) da `sin artefacto`.
- `cargo test -p xtask` (lint de workflows incluido): 29 en verde.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-20T14:01:02Z | plan | automatico | persona | 0.16.1 |
| 2026-11-20T14:44:08Z | approve | automatico | persona | 0.16.1 |
| 2026-11-20T14:44:11Z | start | automatico | persona | 0.16.1 |
| 2026-11-20T15:27:02Z | review | automatico | persona | 0.16.1 |
| 2026-11-20T16:16:03Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-20T15:27:02Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 10 | 1270 | 875313 | 884 | 0 | 877477 | - | - |
| 2026-11-20T15:27:02Z | implementacion | sesion:ffc7f907-2a90-4fe5-a6d2-510800ddbc4d | 86 | 39687 | 7520879 | 25721 | 0 | 7586373 | - | - |

## Verificacion ligera

- 2026-11-20T15:39:09Z: ronda 1; commit revisado 92f28c9b8bf2191a5531bcf016dbbb47efe50a07; commit verificado a329520010cc35d0fd6475e44684544a06bfb1aa; ficheros: .github/workflows/revision-ia.yml; corregidos: MEN-2; pendientes: ninguno

## Duracion

- Estimada: 120 min
- Real (de la rama abierta al cierre con merge, sin pausas): 92 min
- Paso de la hora: 92 min reales frente a 120 estimados
- Cierre (finish): 1330 s, de 2026-11-20T15:53:53Z a 2026-11-20T16:16:03Z
