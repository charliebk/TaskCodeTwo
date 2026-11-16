# Informe de revision — TASK-077 (ronda 3)

- Commit revisado: 91f308772a3e21675c1a1cb6ce87fb26e7cb1a29
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE (escribelo con: taskctl veredicto TASK-077 aprobada | aprobada-con-correcciones | cambios-solicitados)

Veredicto que propone el revisor: **aprobada** (0 CRITICO, 0 IMPORTANTE, 0 MENOR nuevos; MEN-4 de la ronda 2 queda corregido y verificado).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-4 | MENOR | corregido | crates/tc2/src/git/remoto.rs:198 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin hallazgos nuevos: el diff de esta ronda no abre ningun CRITICO ni IMPORTANTE, ni un MENOR que la propia correccion introduzca.

### Verificacion de las correcciones de la ronda 2

- **MEN-4 (`traer_rama` clasificaba «rama inexistente» como «sin red»): corregido.** `traer_rama`
  (`remoto.rs:187-209`) llama a `clasificar_fallo_remoto` sobre el stderr de `git fetch`; si devuelve
  `RamaInexistente` (`remoto.rs:92`, patron `couldn't find remote ref`), devuelve
  `Informe::nuevo(op, Resultado::Rechazada)` con `motivo=sin-rama-base`, `rama=<rama>` y el mensaje
  `SIN_RAMA_BASE` (`i18n/catalogo/git.rs:73-76`, es y en). Cualquier otro fallo pasa a `fallo_fetch`, igual
  que antes. `fallo_fetch` (`remoto.rs:177`) sigue mapeando `RamaInexistente` a `sinred`, pero solo lo
  alcanza `traer` (el `fetch` completo), donde no puede aparecer esa clasificacion.
  - Caso construido: `origin` sin `refs/heads/develop` (test `origin_sin_rama_base_no_es_sin_red`,
    `tests/identidad.rs`). Salida esperada `identidad rechazada motivo=sin-rama-base rama=develop`, sin
    peticiones a la API y sin escribir la cache. Pasa en el commit revisado.
  - Mutacion: `if false && clasificar_fallo_remoto(...) == RamaInexistente` en `remoto.rs:198` →
    `origin_sin_rama_base_no_es_sin_red` pasa a FAILED (`tests/identidad.rs:349`, `left == right`).
    Restaurado con `git checkout` en el clon.
- **El `fetch` de siempre no cambia de comportamiento.** `traer` y `fetch` (`remoto.rs:169-171` y
  `:118-`) no han cambiado en el diff: solo se mueve el mapeo a `fallo_fetch`, que es el mismo. El test
  `origin_sin_rama_base_no_es_sin_red` cierra con `git::fetch(...)` sobre el mismo remoto sin `develop`
  y comprueba `es_exito()`.
- **Sin `unwrap`, `expect`, `panic!`, `unreachable!`, indexado ni `unsafe` en el codigo nuevo.** Las
  apariciones de `unwrap` en `remoto.rs` estan en el modulo de tests (desde la linea 595). El workspace
  tiene `unsafe_code = "forbid"` (ronda 2). La rama de `traer_rama` usa solo `Option`/`Result` sin panico.
- **Mensajes nuevos en español e inglés.** `SIN_RAMA_BASE` tiene `es` y `en`. `CUENTA_FUERA_DE_LISTA`
  (`i18n/catalogo/identidad.rs:57-60`) tiene `es` y `en`, con el nuevo `{base}` en ambos. El test
  `cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache` comprueba los dos idiomas y que no aparezca
  `tc2 identidad`.
- **Sin cambios en el limite JS-WASM.** No hay firmas `#[wasm_bindgen]` ni tipos exportados en el diff, asi
  que no se pidio llamador TypeScript ni `.d.ts`.

## Alcance

- Ficheros revisados (del diff de la ronda que casan con los patrones): `crates/tc2/src/git/remoto.rs`,
  `crates/tc2/src/identidad.rs`, `crates/tc2/src/i18n/catalogo/git.rs`,
  `crates/tc2/src/i18n/catalogo/identidad.rs`, `crates/tc2/tests/identidad.rs`.
- Commit: la peticion indica 91f3087 como HEAD revisado. El HEAD actual de la rama es 3b8bf77; entre
  91f3087 y 3b8bf77 solo cambian ficheros de `tareas/` (`git diff 91f3087..HEAD -- crates/` vacio). El
  codigo revisado es el de 91f3087.
- Contexto adicional pedido: ninguno. Se leyeron en el repo original los puntos de llamada de `traer_rama`
  (`identidad.rs:471`) y los mensajes de `identidad.rs:120-140`, para ver como sale el `Rechazada`.
- Limite JS-WASM: sin cambios de firma exportada con `#[wasm_bindgen]`; no hay llamador TypeScript que revisar.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-077-r3-rust`, rama
  `feature/task-077-f2-13-identidad-de-la-persona-desde-la-p`, HEAD 3b8bf77 (codigo = 91f3087). Sin cambios
  ni commits en el clon salvo la mutacion, que se revirtio. Borrado al terminar.
- Build: `cargo test -p tc2 --locked` compila sin errores.
- Formato y lints: `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0.
- Suite: `cargo test -p tc2 --locked --no-fail-fast`: exit 0. Lib 83 pasan; `tests/identidad.rs` pasa,
  incluido `origin_sin_rama_base_no_es_sin_red`; el resto de binarios pasa, con los ignorados habituales
  (2 + 2 + 1 ignorados en binarios de tarea y git, sin fallos).
  - Incidencia: en una pasada previa (sin `--no-fail-fast`), `protecciones_y_bordes` de
    `tests/plataforma.rs:641` fallo una vez (`left == right`). Fichero fuera del diff. Repetido solo, pasa
    (`cargo test --test plataforma protecciones_y_bordes`: 1 passed), y pasa en la pasada completa
    `--no-fail-fast`. Se trata como inestabilidad del entorno (127.0.0.1 lento en esta maquina), no como
    fallo de la rama. No se ha investigado mas, porque el fichero no esta en el diff.
  - Linea base: la ronda 2 dejo la suite en verde para 2fae3e4; el codigo de esta ronda solo cambia
    `remoto.rs`, `identidad.rs`, los catalogos y los tests del diff.
- Casos construidos:
  - `origin` sin `develop` (remoto `bare` con `update-ref -d refs/heads/develop`): salida
    `rechazada motivo=sin-rama-base rama=develop`, sin peticiones a la API, cache sin escribir (MEN-4).
  - `git::fetch` sobre el mismo remoto sin `develop`: exito (camino de `fetch` de siempre).
- Mutacion:
  - MEN-4: `if false && clasificar_fallo_remoto(...) == RamaInexistente` en `remoto.rs:198` →
    `origin_sin_rama_base_no_es_sin_red` FAILED. Revertido.
- Medidas: no aplican.

## Revisado sin hallazgos

- `traer_rama` (`remoto.rs:187-209`): el patron de `RamaInexistente` y la rama de `Rechazada` no abren
  nuevos caminos de error; el resto de fallos siguen en `fallo_fetch` como antes.
- `fallo_fetch` y `traer` (`remoto.rs:169-181`, `:118-`): sin cambios de comportamiento.
- `i18n` (`catalogo/git.rs`, `catalogo/identidad.rs`): textos es/en completos; `{base}` se pasa desde
  `identidad.rs:137-140` (`config::BASE`).
- Tests del diff (`tests/identidad.rs:318-365`): los dos nuevos aseveran la linea de contrato y los
  textos en ambos idiomas, y el segundo prueba que el `fetch` normal no cambia.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/identidad.rs` lineas 120-140 y 455-480 (fuera de los tramos del diff): para ver el
  `Rechazada` de `traer_rama` y el paso de `config::BASE` al mensaje de fuera de lista.
- `crates/tc2/src/git/remoto.rs` lineas 80-215 y `crates/tc2/src/git/remoto.rs` (busquedas de `RamaInexistente`):
  para ver el clasificador y los llamadores de `traer_rama`.
- Busqueda en `crates/tc2/src` de `traer_rama|SIN_RAMA_BASE|sin-rama-base|RamaInexistente`: para confirmar
  que solo `identidad.rs:471` llama a `traer_rama`.
