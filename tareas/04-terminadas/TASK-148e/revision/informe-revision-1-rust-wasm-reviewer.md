# Informe de revision — TASK-148e (ronda 1)

- Commit revisado: b784ddac5325664edac24005c42ed909e40e3a2a
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: **aprobada-con-correcciones** (dos MENOR, ninguno bloquea; el soporte comun cumple su objetivo, la suite esta en verde y los mutantes sobre `estado::prueba` mueren todos).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/estado/montaje.rs:321 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/git/prueba.rs:34 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `type R<T>` queda copiado en `estado/montaje.rs`

`montaje.rs` ya importa `crate::git::prueba::{aislar, sh}` pero conserva su propio
`type R<T> = Result<T, Box<dyn std::error::Error>>;` (linea 321), identico a
`git::prueba::R` (`pub(crate)`). Es una copia exacta del alias que la tarea unifica
(Duplicacion de la Regla 4D, de bajo coste). Reproduccion: `grep -rn "type R<" crates/tc2/src`
devuelve `estado/montaje.rs:321` ademas de `git/prueba.rs:502`. Correccion: anadir `R` al `use`
y borrar la linea 321.

### MEN-2 — la rama de error de `git::prueba::sh` no la ejerce ningun test

Mutante: `if !s.status.success()` -> `if false && ...` (`sh` devuelve `Ok` aunque git falle).
La suite `--lib` sigue en verde: 398 ok, 0 fallos. El unico test propio
(`sh_lee_la_configuracion_aislada`) solo cubre el camino feliz. No es regresion (el `sh` original de
`ciclo/mod.rs` tenia el mismo comportamiento y tampoco estaba cubierto), pero ahora `sh` es soporte
compartido por tres modulos y un fallo silencioso de git en un escenario se reflejaria mas tarde
y peor. Correccion sugerida: un test que llame `sh(&raiz, &raiz, &["comando-inexistente"])` y
espere `Err` con `git [...]` en el mensaje.

## Regla 4D

- Duplicacion: MEN-1 (`type R` en montaje.rs). Resto cumple: en `crates/tc2/src` no queda
  `struct EnMemoria`, `impl Estado for EnMemoria`, `fn estado/motivo/texto/entorno/sh` ni `const CONFIG`
  de tests fuera de `estado::prueba` y `git::prueba` (`grep -rnE` sobre el clon; las coincidencias
  restantes de `texto`/`motivo`/`estado` son metodos o funciones de produccion con otro cometido).
  `EnMemoria` en `crates/tc2/tests/roadmap.rs:556` es del crate de tests de integracion (otro crate,
  fuera de alcance, y `git/prueba.rs` lo declara).
- Desviacion: cumple. El diff de `crates/` solo toca modulos `#[cfg(test)]` y los dos `mod prueba`
  declarados con `#[cfg(test)]`; ni codigo de produccion ni API publica cambian (los `pub(crate)` solo
  existen en test).
- Diluvio: cumple. Dos ficheros de soporte, funciones planas, sin traits nuevos.
- Disfraz: cumple. Los tests migrados conservan sus aserciones; el unico test nuevo asserta un valor
  concreto (`prueba@example.com`).

## Alcance

- Ficheros revisados: los 16 de la peticion (liberar, ciclo/mod, ciclo/operacion, estado/mod,
  estado/montaje, estado/prueba, git/estado, git/mod, git/prueba, plan/aprobar, plan/guardar,
  plan/operacion, revision/operacion, roadmap/mod, siguiente, toma/operacion).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada; el diff es solo codigo de tests nativo.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148e-r1`, rama
  `fix/task-148-auditoria-4d-soporte-de-tests-compartido` (HEAD 70347a1, con el commit revisado
  b784dda dentro), `CARGO_TARGET_DIR` dentro del clon. Borrado al terminar.
- Build/lints: `cargo fmt --check` limpio; `cargo clippy -p tc2 --all-targets --locked -- -D warnings`
  sin avisos. No se compila para wasm32 (el diff no toca codigo de ese objetivo).
- Suite: no se repitio la completa (la peticion trae 779 -> 780 ok, 0 fallos, 13 ignorados). Se ejecuto
  `cargo test -p tc2 --lib --locked`: 398 ok, 0 fallos, 2 ignorados, a la primera, incluido
  `desinstalar::puertos::tests::el_auxiliar_real_borra_tras_salir_el_proceso`. Antes del diff: 397.
- Casos construidos: busqueda de copias residuales con `grep` (ver Regla 4D); lectura de
  `git/estado.rs` para comprobar que `repo()` ahora aisla la config antes del `git init` y que el
  `gitconfig` firmado de `plan_blob_es_el_del_arbol_del_propio_commit` sigue sobrescribiendo al de
  `aislar` en la misma ruta que lee `entorno(&d)`.
- Mutacion (cada mutante se aplico sobre el clon, `cargo test -p tc2 --lib`, y se revirtio):

| Mutante | Fichero | Resultado |
|---|---|---|
| `EnMemoria::leer` devuelve `Ok(None)` | estado/prueba.rs | rojo, 36 fallos |
| `EnMemoria::rutas` devuelve vacio | estado/prueba.rs | rojo, 12 fallos |
| `estado()` guarda en `{id}/x.md` | estado/prueba.rs | rojo, 18 fallos |
| `motivo()` devuelve siempre `None` | estado/prueba.rs | rojo, 16 fallos |
| `texto()` devuelve vacio | estado/prueba.rs | rojo, 5 fallos |
| `de_rutas` altera la ruta | estado/prueba.rs | rojo, 5 fallos |
| `GIT_CONFIG_GLOBAL` apunta a otro fichero | git/prueba.rs | rojo, 6 fallos |
| `aislar` escribe config vacia | git/prueba.rs | rojo, 5 fallos |
| `sh` sin `.trim()` | git/prueba.rs | rojo, 3 fallos |
| `sh` ignora el fallo de git | git/prueba.rs | **sobrevive** (MEN-2) |
| quitar `GIT_CONFIG_NOSYSTEM` | git/prueba.rs | sobrevive: solo se nota si el host tiene config de sistema; no se considera hallazgo |
| `defaultBranch` develop -> main, `autocrlf` false -> true en `CONFIG` | git/prueba.rs | sobreviven; la `CONFIG` se movio literal, ya no estaba cubierta antes, no es regresion |

  (El mutante de cambiar el email en `CONFIG` no se cuenta: el `sed` tambien cambio la aserciones del
  test propio, asi que no era valido.)
- Medidas: no aplican.

## Revisado sin hallazgos

- Fuerza de los tests migrados: `revision/operacion.rs` pasa de `estado(String)` a `estado(&[("TASK-001", ..)])`
  con la misma ruta `TASK-001/tarea.md` y las mismas aserciones. `plan/operacion.rs` usa `EnMemoria::de_rutas`
  (semantica antigua de `estado(&[(ruta, ..)])`) sin cambiar rutas; los mutantes de `estado`/`de_rutas`
  los ponen en rojo. `plan/aprobar.rs`, `plan/guardar.rs`, `siguiente.rs`, `roadmap/mod.rs`:
  solo cambia el origen del `EnMemoria`.
- `estado/montaje.rs`: el cambio de `init.defaultBranch=main` a `develop` en la config aislada no afecta,
  porque el escenario fija `HEAD` con `symbolic-ref` y `update-ref` de forma explicita; la config ahora
  se aisla tambien para el `clone`.
- `git/estado.rs`: `repo()` ahora usa `aislar` + `sh`; deja de leer la config global del usuario en
  `git init` (mejora, sin cambio de fuerza).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/estado/montaje.rs` y `crates/tc2/src/git/estado.rs` completos alrededor de los hunks, y
  `grep` sobre `crates/tc2/src` para la comprobacion de 4D: necesario para verificar que no quedan copias.
