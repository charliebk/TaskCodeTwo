---
id: TASK-148b
titulo: "escenario ssh, git de sistema y operacion de prueba (A8-01, A8-16, A3-06, A4-09, A8-20 integracion) (parte b de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 3544116
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-23
dependencias: [TASK-148a]
---
## Objetivo

Parte b del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/remoto.rs` y `comun/git.rs`, extiende `comun/ops.rs`, pone `valor` y `salida` en `lanzador.rs`, migra aprobar, ciclo, estimacion, frescura, inyeccion, liberar, plan, progreso, solape, toma, alta, importar y `agente_git`, y funde `como_persona` y `hito_como_persona` en `aprobar.rs`.

## Criterios de aceptacion
- [ ] `entorno_ssh`, `entorno_sin_red`, `identificar`, `otro_clon`, `editar`, `sin_catalogo`, `Escribir`, `salida`, `con_fila` y `preparar` solo se definen en `crates/tc2/tests/comun/`.
- [ ] Hay una sola `valor`, con `trim_end`, y un test de `tests/soporte.rs` comprueba `valor("k=v\n", "k") == Some("v")`.
- [ ] `agente_git::Clon::git` y `Fixture::crudo` usan `comun::git::git_en`, que aísla la configuración del usuario.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde, con el recuento en `## Resultado` y los IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-23T22:55:26Z | plan | automatico | persona | 0.17.1 |
| 2026-11-23T23:05:05Z | approve | automatico | persona | 0.17.1 |
| 2026-11-23T23:05:30Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23T23:05:03Z | diseno | agente:acc58ef3c6137b25e | 68 | 160352 | 3347341 | 36355 | 59 | 3544116 | - | - |
