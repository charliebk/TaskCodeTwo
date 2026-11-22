---
id: TASK-052
titulo: "F6-6 Rondas incrementales"
tipo: feature
sprint: 6
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-052-f6-6-rondas-incrementales
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1812506
tokens_implementacion: 64883494
tokens_revision: 9201677
creado: 2026-11-14
actualizado: 2026-11-22
dependencias: [TASK-051]
---
## Objetivo

Rondas incrementales: la siguiente mira lo cambiado y si se cerraron los hallazgos; tope y MENORES sin ronda (RF-33)

Tarea F6-6 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F6-5.

## Criterios de aceptacion
- [x] Hallazgos nuevos por ronda, medidos
- [x] Cumple lo que pide la tarea F6-6 del plan y los requisitos que cita

## Resultado

Rondas incrementales de la revision IA (F6-6, RF-33). Cada check `revision-ia` describe su ronda y lleva todos los hallazgos abiertos; la siguiente ronda parte del ultimo check de la App del revisor (no de la rama de estado, que escribe el agente). `tc2` decide con funciones puras y el workflow transporta.
- Contrato v3 de hallazgos: respuesta `{hallazgos, cierres}`; un abierto solo se cierra con un `cierre` explicito anclado al diff incremental (y, con evidencia `paso_rojo`, con ese paso ya en verde en el informe F5-4). Motivos nuevos `cierre-desconocido`, `cierre-fuera-del-diff`, `paso-rojo-sigue-rojo`. `AceptadoDe` gana `ronda` y `estado`; los `raw_details` y titulos v2 se siguen leyendo.
- `hallazgos/ronda.rs` y ordenes `tc2 hallazgos ronda`, `abiertos` y `desde`: numero de ronda y modo `completa`, `incremental`, `ligera` (solo MENOR, sin modelo) o `tope` (sin modelo, `failure`). Tope = `tope_rondas` de la base + elementos de `## Rondas extra` / `## Ronda extra`. Una ronda `incompleta` o `ligera` no sirve de base; tras una ligera la siguiente va con modelo.
- Medida por ronda: titulo `revision-ia: <dictamen> (ronda=R ...)`, resumen e informe de la tarea con `nuevos`, `persisten` y `cerrados`; los cerrados salen como anotacion `notice`.
- `tc2 revisoras --ronda N` aplica S13. `review` guarda el informe del head anterior antes de empujar y su linea gana `ronda=`, `nuevos=`, `cerrados=`.
- `revision-ia.yml`: `preparar` (con `checks: read`) vuelca los check-runs de los commits del PR (`filter=all`), calcula la ronda y el punto de partida del diff con `tc2 hallazgos desde` (sin lo que entra de la base por merge, via `git merge-tree`; rebase o conflicto vuelven a `base...head`). El lint de `xtask` lo exige; el llamador admite `checks: read`.
- Guia `docs/ci/revision-ia.md` («Rondas (F6-6)» y limites) y adendas F6-6 en F1-15, F1-8, F1-10 y F1-16.

Decisiones (opcion recomendada, delegada por Carlos, 2026-11-22): las nueve del plan. La ejecucion de la evidencia (mutantes y bordes) NO entra en F6-6: queda como accion de Carlos crear una fila F6-x en el plan; RR-03 se cumple solo en parte.

Evidencia (2026-11-22):
- C1 en `tc2`: `tests/hallazgos.rs` con el binario real recorre ronda 1 -> ronda 2 (abiertos heredados, cierre valido, hallazgo nuevo, cierre fuera del diff rechazado y el IMPORTANTE sigue rojo); `el_diff_incremental_no_arrastra_lo_que_trae_la_base` con repo Git real; `el_informe_de_la_tarea_cuenta_como_el_check` en los cuatro modos.
- C1 E2E real en `charliebk/taskcodetwo-e2e-github` (PR 26, run `ronda-e2e` 44975983192, sin modelo): check de ronda 1 publicado con la App del revisor en el commit A, push de B, el workflow calculo la ronda 2 desde A y el veredicto de ronda 2 trajo el MENOR persistente (`ronda=2`) y el cerrado como `notice`. Repo limpio despues: PR cerrado, rama borrada, workflows retirados (commit «e2e TASK-052: limpieza» en `main`).
- C2: unitarios de `hallazgos::ronda::decidir` (tope, rondas extra, ligera, incremental, incompleta), esquema v3 contra el codigo, lint con una mutacion por regla.
- `cargo xtask verificar --perfil rapida` en verde sobre 8822e8e (ronda 1) y sobre d032160 (correcciones). La primera ejecucion del implementador cayo solo por el intermitente conocido `identidad.rs::cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache` (`identidad sinred`, mock HTTP en Windows); no es del cambio.
- Nota: el informe final del agente implementador se perdio al reiniciarse la sesion; lo de arriba sale de los commits, del estado del repo E2E y de las dos rondas de revision, que reprodujeron por su cuenta.

Sin comprobar: un PR real con modelo (no hay saldo en la API y no se pondra); que un llamador sin `checks: read` falle al validarse en GitHub (documentado, no probado en un run); anotaciones de persistentes cuya linea ya cambio en el head nuevo.

Revision por pares:
- Ronda 1 (8822e8e): rust-wasm-reviewer y code-quality-reviewer, cambios solicitados por 2 IMPORTANTE y 9 MENOR.
  - Rust: IMP-1 la ronda `ligera` daba verde a codigo no revisado y se volvia base; MEN-1 informe y check contaban distinto en la ligera; MEN-2 `## Rondas extra` contaba lineas vacias y bloques de codigo; MEN-3 el ancla del cierre no tiene relacion con el abierto (aceptado: limite documentado, exigir el mismo fichero romperia arreglos legitimos en otro fichero); MEN-4 un objeto sin `hallazgos` tiraba los cierres.
  - Calidad: IMP-1 el diff incremental arrastraba lo que trae la base por merge; MEN-1 un `incompleta` tapaba el rojo anterior del mismo commit; MEN-2 la guia decia mal que pasa sin `checks: read`; MEN-3 limite de la ligera sin documentar; MEN-4 sin test de elementos `* `; MEN-5 un check con anotaciones incompletas bloqueaba el PR (ahora falla en cerrado con un mensaje que dice como salir).
  - Corregidos en cf116db, d4ecb7a y d032160, con test y mutacion por hallazgo comprobable.
- Ronda 2 (3df9643, acotada): las dos revisoras, aprobada, sin hallazgos; comprobaron cada correccion (calidad reprodujo `tc2 hallazgos desde` con merge, rebase, conflicto y argumentos invalidos).

## Divergencias

- Orden nueva `tc2 hallazgos desde` (no prevista en el plan) para el punto de partida del diff incremental sin los merges de la base; anade `Subcomando::MergeTree` al puerto Git.
- `elegir` devuelve `Eleccion{check, ligera_pendiente}` y salta las rondas ligeras como base (correccion de IMP-1 de rust, que matiza la decision 4 del plan).
- El volcado de check-runs pide `filter=all`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-22T00:53:43Z | plan | automatico | persona | 0.16.1 |
| 2026-11-22T01:03:06Z | approve | automatico | persona | 0.16.1 |
| 2026-11-22T01:03:08Z | start | automatico | persona | 0.16.1 |
| 2026-11-22T10:58:37Z | review | automatico | persona | 0.16.1 |
| 2026-11-22T12:33:53Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-22T01:03:05Z | diseno | agente:a4891d81e874ec542 | 48 | 136071 | 1638928 | 37459 | 41 | 1812506 | - | - |
| 2026-11-22T10:58:37Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 54 | 238895 | 6612337 | 11198 | 0 | 6862484 | - | - |
| 2026-11-22T10:58:37Z | implementacion | agente:a964c0a0891879187 | 336 | 947551 | 56885004 | 188119 | 59 | 58021010 | - | - |
| 2026-11-22T11:10:41Z | revision | agente:a1c80949403328a0b | 50 | 178165 | 3492411 | 30873 | 2 | 3701499 | 1 | rust-wasm-reviewer |
| 2026-11-22T11:10:41Z | revision | agente:a28467849fa693716 | 44 | 179498 | 3101094 | 25666 | 10 | 3306302 | 1 | code-quality-reviewer |
| 2026-11-22T12:42:20Z | revision | agente:a0a65929bf386b854 | 20 | 120113 | 980002 | 5914 | 6 | 1106049 | 2 | rust-wasm-reviewer |
| 2026-11-22T12:42:20Z | revision | agente:abec821515871329e | 20 | 119014 | 960823 | 7970 | 3 | 1087827 | 2 | code-quality-reviewer |
