# Informe de revision — TASK-148b (ronda 2)

- Commit revisado: 6bb76aa675970f7b6451fa69b1e32ccac3d0d454
- Revisor: general-purpose (haiku) con rust-wasm-reviewer
- Veredicto: cambios-solicitados

Veredicto propuesto: cambios-solicitados. IMP-1 queda corregido, pero la correccion de MEN-2 deja un test nuevo (IMP-2) que rompe la suite en un entorno Windows sin `sh` en el PATH. La correccion es pequena.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/importar.rs:62 | implementacion |
| IMP-2 | IMPORTANTE | corregido | crates/tc2/tests/soporte.rs:105 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `publicado` duplicado (corregido)

Verificado en el clon, commit 6bb76aa:

- `grep "fn publicado"` en `crates/tc2/tests/`: solo quedan `Fixture::publicado` (`comun/remoto.rs:131`) y `Fixture::publicado_bytes` (`comun/remoto.rs:136`). Importar, revision y roadmap usan `f.publicado` o `roadmap_publicado`, que llama a `publicado_bytes`.
- `tc2` de roadmap (`roadmap.rs:21`) y `correr` de frescura (`frescura.rs:38`) van sobre `comun::lanzar`.
- `verify-commit` solo aparece en el ayudante `verificar_firma` (`aprobar.rs:117`), que usan `c3` y `f4_3`. Los dos bloques `Command::new("git")` de aprobar se han quitado.
- Residuales fuera de la duplicacion que corrige IMP-1, que existian antes y no estan en el diff: `revision.rs:494` (cierre `informe` con `Command::new("git") ... show`, devuelve `Option`, no pasa por `git_en`) y `estado_casos.rs:64` (`merge-base`). Se anotan como nota MENOR, no como hallazgo.

### IMP-2 — `salida_rechaza_stderr_en_un_exito` depende de un `sh` en el PATH (abierto)

Introducido por la correccion de MEN-2 (commit be874cb). El test hace `std::process::Command::new("sh")` y un `unwrap`:

- Reproducido con `cargo test -p tc2 --locked --test soporte` desde PowerShell, sin `C:\Program Files\Git\bin` en el PATH: 8 ok, 1 FAILED. El panic es `called Result::unwrap() on an Err value: Error { kind: NotFound, message: "program not found" }` en `soporte.rs:107`, no el mensaje esperado `stderr en un exito`, así que el `should_panic` no pasa.
- Con `C:\Program Files\Git\bin` delante en el PATH: 9 ok.
- Causa: el test necesita un ejecutable `sh` que la suite no declara ni busca. En esta sesion no estaba en el PATH. El usuario reporta 0 fallos, probablemente porque lo ejecuta con `sh` disponible; en un entorno sin él la suite queda en rojo.
- Correccion sugerida (no aplicada): que el stderr de un exito lo produzca `git`, p. ej. `git -c alias.x='!echo ok; echo aviso >&2' x` (git usa su propio `sh`), o declarar `sh` como prerrequisito y saltar el test si falta.

Severidad IMPORTANTE: comportamiento incorrecto en un entorno real de Windows, no de borde. No es CRITICO.

### MEN-1, MEN-2 y MEN-3 (de la ronda 1, no en la tabla)

Comprobados solo por mutacion, como evidencia del diff de la correccion (ver Reproduccion). Cada mutante lo mata un test nuevo.

## Regla 4D

- Duplicacion: cumple (IMP-1 corregido; los residuales de `revision.rs:494` y `estado_casos.rs:64` son nota MENOR).
- Desviacion: cumple.
- Diluvio: cumple.
- Disfraz: cumple. Los tres `should_panic` nuevos (MEN-1, MEN-2, MEN-3) fallan cuando se quita la proteccion que prueban.

## Lecturas fuera del contexto permitido

- `crates/tc2/tests/comun/lanzador.rs`: para ver `salida` y `lanzar`, que la correccion de MEN-2 y de IMP-1 usa.
- `crates/tc2/tests/revision.rs` (linea 494) y `crates/tc2/tests/estado_casos.rs` (linea 64): solo con grep y lectura puntual, para buscar copias de `git` fuera de `comun/`.

## Alcance

- Ficheros revisados: los 8 tests del diff de la correccion (`aprobar.rs`, `comun/git.rs`, `comun/remoto.rs`, `frescura.rs`, `importar.rs`, `revision.rs`, `roadmap.rs`, `soporte.rs`) y `comun/lanzador.rs`.
- Contexto adicional pedido: ninguno aparte de lo anterior.
- Limite JS-WASM: sin cambios (tests nativos de un CLI Rust, sin firmas exportadas con `wasm-bindgen`).

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\TaskCodeTwo\task-148b-r2`, commit 6bb76aa, con `CARGO_TARGET_DIR` dentro del clon (`target-r2`). Borrado al terminar el informe.
- Build: `cargo test -p tc2 --locked --no-run` sobre los seis binarios (`soporte`, `importar`, `revision`, `roadmap`, `frescura`, `aprobar`): ok, 1m56s.
- Formato y lints: no ejecutados en esta ronda (la peticion acota la verificacion a los tests).
- Suite (sin `sh` en el PATH, como en la sesion de PowerShell):
  - soporte: 8 ok, 1 FAILED (IMP-2).
  - importar: 9 ok.
  - revision: 5 ok.
  - roadmap: 13 ok.
  - frescura: 12 ok.
  - aprobar: 10 ok.
  - Con `C:\Program Files\Git\bin` en el PATH, soporte da 9 ok.
  - La suite completa (48 binarios, 776 ok) no se ha repetido: se cita la de la peticion.
- Casos construidos: grep de `fn publicado`, `Command::new("git")` y `verify-commit` sobre `crates/tc2/tests/` (IMP-1); el test `salida_rechaza_stderr_en_un_exito` con y sin `sh` en el PATH (IMP-2).
- Mutacion (cada mutante, un fichero, restaurado con `git checkout` en el clon; el clon queda limpio):
  - M1: quitar `GIT_CONFIG_NOSYSTEM` de `git_salida`: mata `git_en_fija_locale_y_prompt` (MEN-1).
  - M2: quitar `LC_ALL=C` de `git_salida`: mata `git_en_fija_locale_y_prompt` (MEN-1).
  - M3: `o.status.success()` por `true` en `git_salida`: mata `git_en_con_un_git_que_falla_hace_panic` (MEN-1).
  - M4: `fila.ends_with(...)` por `true` en `con_fila`: mata `con_fila_rechaza_una_fila_de_otra_fase` (MEN-3).
  - M5: quitar el `assert!` de stderr vacio en `salida`: mata `salida_rechaza_stderr_en_un_exito` (MEN-2).
- Medidas: no aplican.

## Revisado sin hallazgos

- Migracion de `aprobar.rs` (`verificar_firma` sobre `git_salida`, usado en `c3` y `f4_3`): mismas expectativas de firma y mismo entorno.
- `frescura.rs` (`correr` sobre `comun::lanzar`, idioma por `tc2::i18n::VARIABLE`): los doce tests pasan.
- `importar.rs` y `roadmap.rs` (uso de `f.publicado` y `roadmap_publicado`): los tests pasan y no quedan copias.
- `comun/git.rs` (`git_salida` y `git_en`): los asserts de `GIT_CONFIG_GLOBAL`, `LC_ALL` y `GIT_CONFIG_NOSYSTEM` se ejercitan.
- `comun/remoto.rs` (`publicado_bytes`, `con_fila`) y `comun/lanzador.rs` (`salida`): sin cambios de comportamiento fuera de lo ya citado.

## Nota (MENOR, fuera de la tabla)

- `revision.rs:494` y `estado_casos.rs:64` siguen llamando a `git` directamente sin `git_en`, aunque el criterio A8-20 centraliza esa llamada. Ni existian en el diff de la ronda 1 ni los toca esta correccion.
