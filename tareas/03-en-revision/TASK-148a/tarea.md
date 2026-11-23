---
id: TASK-148a
titulo: "lanzador del binario en tests/comun (A8-02, absorbe A5-07) (parte a de TASK-148)"
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
tokens_diseno: 2072119
tokens_implementacion: 10611113
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-23
dependencias: []
---
## Objetivo

Parte a del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/lanzador.rs` (`BINARIO`, `tc2`, `texto`) con `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `TC2_IDIOMA=es` y sin tokens ni `TC2_APP_*`, y migra los 26 ficheros que lanzan `CARGO_BIN_EXE_tc2`.

Objetivo de la tarea entera:

> Extraer a `crates/tc2/tests/comun/` (integración) y a módulos `prueba` de `src` (tests unitarios) el soporte de tests que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiado entre dueños: 9 casos IMPORTANTE (siete `divergente`, contando A3-06) de los grupos lanzador-tests, servidor-http-test, instantanea-de-carpeta, soporte-tests-unitarios y dobles-plataforma. La tarea también cierra las fichas fusionadas en ellos (A3-06 en parte, A4-09, A5-07, A7-06, A2-06). Toca los tests de A1 a A8. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion

- [ ] `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo devuelve ficheros de `tests/comun/`.
- [ ] Un test de `tests/soporte.rs` comprueba que `comun::tc2(dir)` quita `TC2_TOKEN_PERSONA`, `TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR` y las variables `TC2_APP_*`, y fija `TC2_IDIOMA=es`.
- [ ] `cargo test -p tc2 --no-fail-fast` da el recuento de antes más los tests nuevos y las copias del test de `http_mock` justificadas, y `cargo xtask verificar` sale en verde; el `## Resultado` lista A8-02 y A5-07 con su commit.

## Resultado

Lanzador unico del binario en `crates/tc2/tests/comun/lanzador.rs` (commit 1466bd4): `BINARIO` y `orden_tc2()`, que devuelve el `Command` con `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0` y `TC2_IDIOMA=es` (con `tc2::i18n::VARIABLE`) y quita los tokens de los tres roles y las variables de GitHub App (`Rol::variable` y `Rol::variables_app`). Los 26 ficheros que lanzaban `CARGO_BIN_EXE_tc2` lo usan; cada test anade despues lo suyo (`current_dir`, `stdin`, `PATH`, `CLAUDE_CONFIG_DIR`, otro idioma...), y en `Command` gana la ultima operacion por variable.

- **A8-02** y **A5-07** (fusionado en A8-02): 1466bd4.
- `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo devuelve `crates/tc2/tests/comun/lanzador.rs:15`.
- Test nuevo `tests/soporte.rs::el_lanzador_quita_credenciales_y_fija_el_idioma`.
- Recuento con `cargo test -p tc2 --no-fail-fast` (target propio en `%LOCALAPPDATA%\pruebas-claude`):
  - antes (6f699e7): 47 binarios, 762 ok, 1 fallo, 13 ignorados. El fallo es `desinstalar::puertos::tests::el_auxiliar_real_borra_tras_salir_el_proceso` (plazo de 10 s), con la suite del hook de Stop corriendo a la vez; no lo toca este cambio y despues pasa.
  - despues (1466bd4): 48 binarios, 768 ok, 0 fallos, 13 ignorados.
  - Cuadre: 763 + 1 test nuevo + 4 copias del test `comun::http_mock::tras_una_respuesta_completa_el_mock_no_manda_fin` en los binarios que pasan a declarar `mod comun;` (`soporte`, `cli`, `actualizar_cli`, `plataforma_e2e`) = 768.
- Cambio de comportamiento deliberado: los ficheros que no quitaban tokens (alta, importar, convivencia_016, roadmap, seleccion, siguiente, hallazgos, revisoras, verificar, cli, actualizar_cli, plataforma_e2e, git_binario) o no fijaban `TC2_IDIOMA` pasan a aislarse. Ninguno se puso en rojo.
- `cargo clippy -p tc2 --tests --all-features -- -D warnings` y `cargo fmt --check` limpios.

## Divergencias

- La funcion se llama `comun::orden_tc2()` y no `comun::tc2(dir)`: una funcion `tc2` dentro de `comun` choca con la ruta del crate `tc2` (`use tc2::...` del propio modulo), y sin carpeta porque cuatro lanzadores (cli, hallazgos, actualizar_cli, plataforma_e2e) no la fijan. El criterio 2 se comprueba igual sobre `orden_tc2()`.
- `texto` no se crea aqui: las copias de `texto` son de la parte b (su criterio la nombra), y crearla sin usarla seria codigo muerto.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-23T14:17:53Z | plan | automatico | persona | 0.17.1 |
| 2026-11-23T14:29:33Z | approve | automatico | persona | 0.17.1 |
| 2026-11-23T14:29:41Z | start | automatico | persona | 0.17.1 |
| 2026-11-23T16:15:13Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23T14:26:39Z | diseno | agente:a3d66ffa5f9c04410 | 50 | 121972 | 1919336 | 30761 | 36 | 2072119 | - | - |
| 2026-11-23T16:15:13Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 106 | 82112 | 10505065 | 23830 | 8 | 10611113 | - | - |

## Autorrevision

- rust-wasm-reviewer — 3e2dba4
