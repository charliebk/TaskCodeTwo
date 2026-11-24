# Informe de revision — TASK-148f (ronda 1)

- Commit revisado: fd776e6274ef35d6a465d6b19761918159920678
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: **aprobada** (un unico hallazgo MENOR, heredado de antes del diff, no bloqueante).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/identidad.rs:744 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — el contador de `Falsa` en `identidad.rs` no esta protegido por ningun test

El test `bot_nunca_asignado_y_sin_tty_es_agente` solo compara `llamadas` antes y despues de la llamada sin terminal
(`assert_eq!(ana.0.llamadas.get(), antes, ...)`), y tras la llamada con terminal no comprueba que el contador
subio. Reproduccion (clon propio, mutante sobre `Falsa::identidad`):

- Mutante C1: quitar `self.llamadas.set(self.llamadas.get() + 1);` (el contador deja de contar) -> `cargo test -p tc2 --lib identidad::` sigue en verde (6 ok).
- Mutante C3: ademas, hacer que `quien_ejecuta` llame a `plataforma.identidad(Rol::Agente)` sin terminal, con el contador roto -> sigue en verde.
- Mutante C2 (codigo llama a `identidad` sin terminal, contador intacto) -> rojo. Es decir, el contador solo vale si cuenta, y nada impide que deje de contar.

No es regresion de la migracion: el contador y su asercion se conservaron tal cual estaban antes del diff. Es una
debilidad heredada que la migracion mantiene. Arreglo barato: tras la llamada con terminal, `assert_eq!(ana.0.llamadas.get(), 1)`.

## Regla 4D

- Duplicacion: cumple. `git grep -nE 'impl(<[^>]*>)? Plataforma for' -- crates` da solo `github.rs:401`, `gitlab.rs:527` y los dos `Doble<P>` (`plataforma/prueba.rs:61`, `tests/comun/plataforma.rs:58`); `git grep -n sin_uso -- crates` vacio; no queda otro doble con relleno. La copia de `Parcial`/`Doble` en dos crates esta justificada (frontera de crate y orfandad) y documentada en el plan.
- Desviacion: cumple. API publica sin cambios: lo unico nuevo en `src` es `#[cfg(test)] pub(crate) mod prueba;` y dos comentarios de `plataforma/mod.rs` (quitan «adaptadores de prueba»); lo demas vive en `tests/` y bajo `#[cfg(test)]`. Sin cambios en `#[wasm_bindgen]` ni en tipos que cruzan el limite JS-WASM (el diff no toca nada de eso).
- Diluvio: cumple. Un trait y un envoltorio que sustituyen a cinco dobles con ~100 lineas de relleno; el diff es -165/+269 con 108+89 de soporte nuevo y 2 tests.
- Disfraz: cumple. Los dos `#[should_panic(expected = "merge-pr")]` se ponen en rojo con el mutante que quita el `panic!` (K y K2); ningun `#[test]` descarta resultados sin asercion.

## Alcance

- Ficheros revisados: los 10 de la peticion (identidad.rs, plataforma/mod.rs, plataforma/prueba.rs, revision/mod.rs, tablero.rs, tests/comun/{mod,plataforma}.rs, tests/{revision,soporte,tablero}.rs).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada; no aplica (crate CLI, sin `#[wasm_bindgen]` en el diff).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148f-r1`, HEAD desacoplado en fd776e6, `CARGO_TARGET_DIR` dentro del clon. Borrado al terminar.
- Build/lints: `cargo clippy -p tc2 --all-targets --locked -- -D warnings` sin avisos; `cargo fmt --check` limpio. wasm32 no aplica.
- Suite: linea base citada de la peticion (antes 781; despues 48 binarios, 783 ok, 0 fallos, 13 ignorados); no repetida entera. Ejecutado por mi: `cargo test -p tc2 --lib` 400 ok / 2 ignorados; `--test revision` 5 ok; `--test tablero` 6 ok; `--test soporte` 13 ok.
- Mutacion (todos revertidos tras cada uno):

| Mutante | Resultado |
|---|---|
| A: `Doble` (src) no reenvia `estado_puertas` | rojo (`revision_ia_de_estado_puertas`) |
| D: `Doble` (src) no reenvia `identidad` | rojo (`bot_nunca_asignado_y_sin_tty_es_agente`) |
| E: `Doble` (src) no reenvia `prs_abiertos` | rojo (2 tests de `tablero`) |
| B: `revision_ia` pide `estado_puertas` con otro rol (activa el `assert_eq!(rol, Agente)` de `Puertas`) | rojo |
| C2: `quien_ejecuta` llama a `identidad` sin terminal | rojo (contador intacto) |
| C1/C3: contador de `Falsa` no cuenta | VERDE, ver MEN-1 |
| F, G, H, I, J: `Doble` de tests no reenvia `leer_revision`, `estado_puertas`, `entorno_push`, `prs_abiertos`, `abrir_pr` | rojo en todos (`tests/revision` y `tests/tablero`) |
| M: `review` pide `leer_revision` con otro rol (activa el `assert_eq!(rol, Agente)` de `Apuntes`) | rojo (2 tests) |
| K, K2: el defecto `merge_pr` de `Parcial` ya no hace `panic!` (tests/comun y src) | rojo (los dos `should_panic`) |
| L2: `sin_dato` hace `panic!` | rojo: `review_guarda_el_informe_de_la_revision_ia` llega a `sin_dato("prs-abiertos")`, asi que la rama «sin dato» se ejerce |
| L: `sin_dato` devuelve `Ok` en vez de `Error` | verde: esa rama solo da el `Err(..)` de un `Paso`, cuyo `Resultado` interior no se asierta; no cambia la conducta que se prueba y era igual antes con `sin_uso`, no se reporta |

## Revisado sin hallazgos

- `plataforma/prueba.rs` y `tests/comun/plataforma.rs`: las 10 operaciones reenviadas con la firma correcta, nombres del `panic!` iguales a los del contrato; `Doble` no reenvia `proveedor`, `leer_plan`, etc., que siguen con el defecto del trait, como dice el plan.
- Dobles migrados: conservan contadores (`llamadas`, `lecturas`, `urls`, `prs`), `assert_eq!(rol, Agente)` y las respuestas «sin dato» de `tests/revision.rs` (`prs-abiertos`, `estado-puertas`, `leer-revision` con constantes `puertas::ESTADO_PUERTAS` y `LEER_REVISION`).
- `impl Parcial for Rc<Apuntes>` sustituye a `Compartida` sin perdida de comportamiento.
- Los `panic!("board no ...")` de `tests/tablero.rs` se sustituyen por el mensaje comun con el nombre de la operacion: no se pierde diagnostico.

## Lecturas fuera del contexto permitido

ninguna
