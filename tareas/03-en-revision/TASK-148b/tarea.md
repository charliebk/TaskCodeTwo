---
id: TASK-148b
titulo: "escenario ssh, git de sistema y operacion de prueba (A8-01, A8-16, A3-06, A4-09, A8-20 integracion) (parte b de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 3544116
tokens_implementacion: 40771690
tokens_revision: 17841616
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148a]
---
## Objetivo

Parte b del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/remoto.rs` y `comun/git.rs`, extiende `comun/ops.rs`, pone `valor` y `salida` en `lanzador.rs`, migra aprobar, ciclo, estimacion, frescura, inyeccion, liberar, plan, progreso, solape, toma, alta, importar y `agente_git`, y funde `como_persona` y `hito_como_persona` en `aprobar.rs`.

## Criterios de aceptacion
- [x] `entorno_ssh`, `entorno_sin_red`, `identificar`, `otro_clon`, `editar`, `sin_catalogo`, `Escribir`, `salida`, `con_fila` y `preparar` solo se definen en `crates/tc2/tests/comun/`.
- [x] Hay una sola `valor`, con `trim_end`, y un test de `tests/soporte.rs` comprueba `valor("k=v\n", "k") == Some("v")`.
- [x] `agente_git::Clon::git` y `Fixture::crudo` usan `comun::git::git_en`, que aísla la configuración del usuario.
- [x] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde, con el recuento en `## Resultado` y los IDs con su commit.

## Resultado

Escenario ssh, git de sistema y operacion de prueba comunes en `crates/tc2/tests/comun` (`git.rs` y `remoto.rs` nuevos; `lanzador.rs`, `ops.rs` y `mod.rs` ampliados), usados por los tests de integracion que los copiaban.

- **A8-20** (integracion): d7f9c56. `comun::git::git_en` aisla la configuracion (exige `GIT_CONFIG_GLOBAL`, fija `GIT_CONFIG_NOSYSTEM`, `LC_ALL`, `GIT_TERMINAL_PROMPT`); lo usan `Fixture::crudo` y `agente_git::Clon::git`.
- **A3-06**: c88204c. Una sola `valor` (primera linea, `trim_end`), `texto` y `volcado` en `lanzador.rs`; `como_persona` comun para aprobar (dos copias) y liberar.
- **A8-01**: b0c8ebc. `remoto.rs` con `ORIGIN`, `PERSONAS`, `CONFIG_AUTOMATICO`, `entorno_ssh`, `entorno_sin_red`, `git_ssh`, `git_ssh_con`, `clon_github`, `cuenta`, `identificar`; `lanzar` en `lanzador.rs`.
- **A8-16** y **A4-09**: c6f65f0. `ops::Escribir`, `sin_catalogo`, `lanzador::salida`, `Preparar`, `Fixture::{preparar, transaccion, cambiar_estado, punta, publicado}`, `editar`, `con_fila`, `worktree`.
- Criterio 1: el grep del plan sobre `crates/tc2/tests` sin `comun` sale vacio; `git grep 'fn valor'` solo da `comun/lanzador.rs`.
- Tests nuevos en `tests/soporte.rs` (cada uno visto en rojo con su mutante): `git_en_aisla_la_configuracion_del_usuario`, `git_en_sin_gitconfig_propio_no_lanza_git` y `valor_recorta_y_lee_la_linea_de_contrato` (quitar solo `trim_end` es mutante equivalente: `lines()` ya recorta el fin de linea).
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 768 ok, 13 ignorados (cierre de TASK-148a); despues 48 binarios, 771 ok, 1 fallo, 13 ignorados. El fallo es `prs_abiertos::fallos_tipados` (`sinred` del mock HTTP, intermitencia conocida en Windows; `prs_abiertos.rs` no cambia): repetido solo, 4 ok. Cuadre: 768 + 3 tests nuevos + 1 copia del test de `http_mock` por `mod comun;` en `plataforma_gitlab_e2e.rs` = 772.
- Decision 5: E2E real `plataforma_e2e::persona_aprueba_pr_del_agente` (usa `Clon::git` sobre `git_en`) contra `charliebk/taskcodetwo-e2e-github`: ok.
- Ronda 1 corregida en 1b90eac (IMP-1) y be874cb (MEN-1 a MEN-3, 4 tests nuevos en soporte, cada uno visto en rojo con su mutante). Suite tras la correccion: 48 binarios, 776 ok, 0 fallos, 13 ignorados (772 + 4).
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- `sin_catalogo` y un primer `remoto.rs` con `como_persona` entran en el commit de A3-06 porque `como_persona` los necesita.
- Anadidos `Fixture::git_ssh_con` (absorbe `git_con` de liberar y el de estimacion, con variables de fecha) y `Fixture::transaccion` (base de `cambiar_estado`, la usan tambien los `poner` locales de estimacion y progreso).
- `lanzar` sustituye tambien al `tc2` de alta, importar, estimacion, progreso y solape, y se quitan las copias de `worktree` de toma, liberar, solape y progreso (misma regla 4D).
- `preparar` hace siempre `add .taskcode` con el mensaje «personas y config» (el plan ya lo daba por inocuo).

## Rondas extra

- Ronda 3: la decide Carlos por su consigna del 2026-11-23 (modo automatico, opcion recomendada): IMP-2 de la ronda 2 es barato y lo introdujo la propia correccion de MEN-2; corregido en 759c0ca (el stderr lo produce un alias de git, sin `sh` en el PATH; probado con `sh` fuera del PATH, 9 ok, y con el mutante del assert de `salida`, rojo).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-23T22:55:26Z | plan | automatico | persona | 0.17.1 |
| 2026-11-23T23:05:05Z | approve | automatico | persona | 0.17.1 |
| 2026-11-23T23:05:30Z | start | automatico | persona | 0.17.1 |
| 2026-11-24T00:48:15Z | review | automatico | persona | 0.17.1 |
| 2026-11-24T03:07:42Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23T23:05:03Z | diseno | agente:acc58ef3c6137b25e | 68 | 160352 | 3347341 | 36355 | 59 | 3544116 | - | - |
| 2026-11-24T00:48:15Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 46 | 267772 | 6770035 | 9992 | 3 | 7047845 | - | - |
| 2026-11-24T00:48:15Z | implementacion | agente:ac199e4502e7dfc94 | 306 | 1108902 | 32539848 | 74789 | 35 | 33723845 | - | - |
| 2026-11-24T02:00:17Z | revision | agente:ad7d672ce353439e9 | 128 | 504865 | 7971666 | 30838 | 9 | 8507497 | 1 | rust-wasm-reviewer |
| 2026-11-24T03:26:08Z | revision | agente:adf16bddc2b1ec7e9 | 130 | 187634 | 9103471 | 42884 | 3 | 9334119 | 2 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — c6f65f0
- 4D — c6f65f0: helper buscado, criterios citados, 3 tests mutados
- rust-wasm-reviewer — be874cb
- 4D — be874cb: helper buscado, criterios citados, 4 tests mutados
- rust-wasm-reviewer — 759c0ca
- 4D — 759c0ca: helper buscado, criterios citados, 1 test mutado
