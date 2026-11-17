# Informe de revision — TASK-028 (ronda 4)

- Commit revisado: c9b549c33536f3809efaa1417675c5750db90d33 (codigo; el HEAD 8b09d62 solo añade ficheros de tareas/ y la peticion, el codigo es el mismo)
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-028 aprobada | aprobada-con-correcciones | cambios-solicitados)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:284 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/init/inventario.rs:51 | implementacion |
| MEN-6 | MENOR | corregido | crates/tc2/src/init/inventario.rs:314 | implementacion |
| MEN-7 | MENOR | corregido | crates/tc2/src/init/inventario.rs:439 | implementacion |
| MEN-9 | MENOR | abierto | crates/tc2/src/init/inventario.rs:352 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Notas sobre la tabla:
- MEN-5 no se ha revisado en esta ronda: queda abierto a proposito, para seguimiento (encargo).
- MEN-6 y MEN-7 se ponen `corregido` tras verificarlos (ver abajo), no por leer el diff.
- MEN-9 es nuevo: lo introduce la propia correccion (el aviso "no tiene nada en comun" ahora se afirma
  tambien cuando `git merge-base` falla). Es MENOR por proporcionalidad: no cambia lo que `init`
  escribe ni publica (en ambos casos no hay `git push`).
- Ningun CRITICO ni IMPORTANTE abierto.

### IMP-1 (corregido) — historia local sin relacion con origin

Verificado con la suite sobre c9b549c: `historia_local_sin_relacion_no_propone_publicar` pasa, y
comprueba `no-comprobado` (sin traer origin), `historia-sin-relacion` (origin/main traido, sin nada en
comun) y ninguna orden con `git push` en ambos casos. Mutacion en el clon: ver `## Reproduccion`.

### MEN-6 (corregido) — el aviso y el motivo afirman "sin relacion" sin comprobarlo

Caso de la ronda 3 (escenario D): `origin/main` sin ref local y su sha de `ls-remote` ausente del clon.
Con c9b549c, `init --simular` da `motivo=no-comprobado`, sin push, y el aviso
`AVISO_NO_COMPROBADO` ("no se pudo comprobar ... ejecuta git fetch origin y repite tc2 init"). El
aviso de "nada en comun" no sale. Lo cubre el primer tramo de
`historia_local_sin_relacion_no_propone_publicar` (tests/init.rs, la aserción `no-comprobado` y
`!s.stderr.contains("nada en comun")`). `Historia::NoComprobable` solo sale cuando falta en el clon
todo commit de alguna rama de `origin` (`falta_alguna`), que es lo que el texto dice.

### MEN-7 (corregido) — accion `persona` sin orden

`persona()` (inventario.rs:534) exige el argumento `orden: String`, y las cinco rutas de
`Decision::Persona` del bloque de `ramas()` pasan por ella o llevan orden propia (inventario.rs:439,
441, 446, 470, 495). La rama que la dejaba sin orden (antes en la linea 382-385 de la ronda 3) ya usa
`persona(a, m, comprobar)`. Lo cubre `rama_local_sin_relacion_lleva_su_orden`, que comprueba que toda
linea `persona` tiene orden.

### MEN-9 (abierto, MENOR, nuevo) — `merge-base` con error se toma por "sin ancestro comun"

- Lugar: `historia_relacionada()`, inventario.rs:350-362. `correr()` solo devuelve `Err` si no puede
  lanzar `git`; un codigo de salida distinto de 0 no es error para el. Entonces
  `if s.exito() && !s.stdout.trim().is_empty()` trata igual "no hay ancestro comun" (exit 1, el caso
  normal) y "merge-base fallo" (exit 128 u otro). Si no hay `falta_alguna`, sale `Historia::SinRelacion`
  y el aviso dice "no tiene nada en comun con la de origin".
- Efecto real: el texto afirma algo que no se midio. No cambia el plan: no hay `git push` en ninguna
  de las dos ramas (SinRelacion y NoComprobable), asi que es MENOR.
- Caso plausible, no construido: clon poco profundo (`--depth`), donde `merge-base` no ve el ancestro
  por encima del limite. Esto NO esta reproducido: es lectura del codigo. Arreglo sugerido: distinguir
  exit 1 de otros codigos (o tratar cualquier fallo como `NoComprobable`).

### MEN-5 (abierto, seguimiento) — comillas tipograficas en `comillas()`

No revisado en esta ronda, a proposito. Sigue como lo dejo la ronda 3.

## Alcance

- Ficheros revisados (diff de c9b549c contra baef020, solo `crates/`):
  `crates/tc2/src/init/inventario.rs` (historia_relacionada, ramas, persona),
  `crates/tc2/src/init/mod.rs` (bucle de avisos, lineas 291-312),
  `crates/tc2/src/i18n/catalogo/init.rs` (AVISO_SIN_RELACION y AVISO_NO_COMPROBADO),
  `crates/tc2/tests/init.rs` (los tres tests nuevos o cambiados).
- Contexto adicional pedido: ninguno. Tambien mire `crates/tc2/src/git/mod.rs` (`correr`, `exigir`,
  `commit_de`, `es_ancestro`) para saber que hace `correr` con un codigo de salida distinto de 0.
- Limite JS-WASM: no aplica. `tc2` es un binario nativo; el diff no toca `#[wasm_bindgen]` ni tipos
  que crucen un limite.
- Codigo nuevo de `historia_relacionada` y de `ramas`: sin `unwrap`, `expect`, `panic!` ni `unsafe`.
  Los `.unwrap()` que aparecen en el diff estan en `tests/init.rs` (helper `develop`).
- `--simular` no escribe: `historia_relacionada` usa solo `rev-parse` (`commit_de`) y `merge-base`
  (`correr`). La palabra `fetch` aparece solo dentro de cadenas de `orden` (texto para la persona), no
  se ejecuta. El aviso que pide `git fetch` tampoco lo ejecuta `init`.
- No revisado en esta ronda: el texto en español del aviso `AVISO_SIN_RELACION` sobre la rama de
  estado ("si se publica"); el codigo de publicacion de la rama de estado queda fuera de este diff.
- `task-workflow/revision.md` del plugin no se leyo: el veredicto se deja en PENDIENTE.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r4-rust`, rama
  `feature/task-028-f3-1-taskctl-init-setup`. HEAD de trabajo `8b09d62` (el codigo es el de `c9b549c`).
  El clon se borra al terminar.
- Suite de integracion sobre el codigo limpio: `cargo test -p tc2 --test init --locked`, 20 passed,
  0 failed (resumen filtrado por rtk). No se repitio.
- Clippy sobre el codigo limpio: `cargo clippy --all-targets --locked -- -D warnings`, sin avisos.
- Mutacion MEN-6: en el clon, `if false && traida.is_none() && conocida.is_none()` en
  `inventario.rs:341`. Test `historia_local_sin_relacion_no_propone_publicar` en rojo:
  `left: Some("historia-sin-relacion")`, `right: Some("no-comprobado")`, en `tests/init.rs:852`. Revertido
  con `git checkout` en el clon, que quedo limpio.
- Incidente de ejecucion: una primera ejecucion de `cargo test` con la mutacion se lanzo sin `cd` y
  compilo el repo original (`C:\Users\nullcad2025\Documents\sources\Personal\TaskCodeTwo`), no el clon,
  por lo que dio verde falsamente. El repo original no tiene cambios de codigo: `git status` limpio.
  Solo se generaron artefactos en su `target/` (ignorado por git). La mutacion se repitio en el clon con
  `--manifest-path` absoluto y el resultado de arriba es el valido.
- Casos construidos: el escenario D de la ronda 3 (ref de `origin/main` ausente y sha fuera del clon)
  es el primer tramo del test `historia_local_sin_relacion_no_propone_publicar`; el caso de sha traido
  (IMP-3) lo cubre `ancestro_traido_por_sha_cuenta_como_relacionada`, que pasa.
- Medidas: no aplican.

## Revisado sin hallazgos

- Ausencia de `unwrap`, `expect`, `panic!` y `unsafe` en el codigo de produccion nuevo.
- `--simular` no escribe ni ejecuta `fetch` en la rama nueva de `historia_relacionada`.
- Bucle de avisos de `mod.rs`: cada motivo dispara su aviso con `{rama}` rellenado.

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-028/revision/informe-revision-3-rust-wasm-reviewer.md` y la peticion de
  la ronda 4: indicados en el encargo.
- `crates/tc2/Cargo.toml`: para confirmar que `tc2` es un binario nativo (sin objetivo wasm32).
- `crates/tc2/src/git/mod.rs`: para ver `correr`, `exigir` y `commit_de`.
- `tareas/03-en-revision/TASK-028/revision/informe-revision-3-code-quality-reviewer.md` y
  `informe-revision-4-code-quality-reviewer.md` (solo con grep, para no pisar el ID MEN-8 de ese
  revisor).
