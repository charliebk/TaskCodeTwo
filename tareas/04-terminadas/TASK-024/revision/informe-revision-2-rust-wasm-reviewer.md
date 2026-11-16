# Informe de revision — TASK-024 (ronda 2)

- Commit revisado: 93e40c6be0c023cde0e0699da2d7873e1f3be9b8
- Revisor: general-purpose haiku con rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin hallazgos nuevos. IMP-1 queda `corregido` (verificado abajo). Los hallazgos MEN-1 a MEN-4 de la ronda 1 no se revisan en esta ronda, como pide la peticion.

## Verificacion de IMP-1 (estado-puertas de GitHub, permiso en ingles)

- Evidencia de codigo: `crates/tc2/src/plataforma/github.rs:330` pasa ahora `&t::PERMISO_PR_Y_CHECKS` (entrada `Texto` del catalogo, no un `&str`). `Permiso::from(&'static Texto)` lo deja como `Descrito`, y `Permiso::en` lo rellena con `con_mensaje`, es decir, se traduce segun el idioma.
- Catalogo: `crates/tc2/src/i18n/catalogo/plataforma.rs:163-165`: `es: "pull_requests:read y checks:read"`, `en: "pull_requests:read and checks:read"`. La version en ingles ya no lleva la conjuncion en español.
- Resultado esperado en ingles: «the token lacks permission: it needs pull_requests:read and checks:read ...». Solo queda `{rol}` con el nombre de rol (`agente`), que la ronda 1 ya comento como identificador y no se reabre.
- Busqueda de literales de permiso con conjuncion o espacios fuera del catalogo (`grep` sobre `crates/tc2/src`): ninguno en `plataforma/`. Solo aparecen en textos del catalogo y en otros ambitos (`carril.rs`, `config/`), que no son alcances de permiso.
- Alcance de la prueba: verificado leyendo el codigo y el catalogo. No se ejecuto el mock HTTP de `GitHub::estado_puertas` en esta ronda; la peticion lo permite.

## Revision del diff de la correccion (a6713930..93e40c6)

- `git/puerto.rs`: `ErrorGit::Prohibido` pasa a llevar un `Mensaje`. El `Display` sigue dando el texto en español, y el texto nuevo de `SIN_INDICE_TEMPORAL`, `CON_INDICE_TEMPORAL` e `INDICE_RELATIVO` coincide palabra por palabra con el de develop. No hay cambio en español.
- `i18n/mod.rs`: `debug_assert!(false, ...)` cuando un hueco queda sin valor. En debug (tests) revienta; en release deja `{nombre}` visible. Es una decision de diseño y no un fallo en el estado actual: la suite entera pasa y la revision de ronda 1 no encontro huecos sin rellenar. Sin hallazgo.
- `lib.rs` (test nuevo `new_y_actualizar_rama_en_ingles_mismo_stdout_y_codigo`): cubre las ramas `new` y `actualizar-rama` que pedia MEN-3. Pasa en verde.
- `plataforma/github.rs` y `i18n/catalogo/*`: cambios localizados, sin efectos colaterales sobre otras rutas de permiso (solo se usa `PERMISO_PR_Y_CHECKS` en `estado_puertas`).

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-024-r2\repo`, `git clone` del repo original y `git checkout 93e40c6be0c023cde0e0699da2d7873e1f3be9b8` (cabeza detached). El repo original no se toco. Clon borrado al terminar.
- Suite: `cargo test -p tc2 --locked`, una sola ejecucion sobre el commit revisado: exit 0, sin fallos. Sin `--ignored`. Sin `sinred` intermitente en esta ejecucion.
  - lib: 52 passed (incluido `i18n` y `contrato::tests::mismo_informe_dos_idiomas`).
  - alta 3, cli 3, config 8, estado_bordes 9, estado_casos 4, estado_concurrencia 3, git_binario 1, git_casos_limite 5, git_ciclo 1, git_conflicto 1, git_push 1, plataforma 4, plataforma_gitlab 3, tarea 5 (1 ignorado). Los e2e (`plataforma_e2e`, `plataforma_gitlab_e2e`) tienen 2 ignorados cada uno, como es habitual.
- Casos construidos: no se construyo un caso nuevo; la verificacion de IMP-1 es de lectura del catalogo y del camino de `Permiso::en`.
- Mutacion: no aplica en esta ronda (verificacion acotada).
- Medidas: no aplican.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/plataforma/errores.rs` (definicion de `Permiso` y `Permiso::en`): necesario para ver como se traduce el permiso de IMP-1.
- `crates/tc2/src/plataforma/conexion.rs` (firma de `leer`, que acepta `impl Into<Permiso>`): necesario para confirmar que el `&Texto` entra como `Descrito`.
- Busquedas con `grep` sobre `crates/tc2/src/plataforma/gitlab.rs`, `git/remoto.rs`, `carril.rs` y `config/`: solo para descartar otros literales de permiso con conjuncion.
