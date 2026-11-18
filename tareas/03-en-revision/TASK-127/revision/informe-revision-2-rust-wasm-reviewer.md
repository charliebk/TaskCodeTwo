# Informe de revision — TASK-127 (ronda 2)

- Commit revisado: 0d318d505400cf903b38867e6846e8355967785c
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

sin hallazgos. Ningun CRITICO ni IMPORTANTE abierto ni nuevo. La correccion
(`con_entrada` reutiliza `git_fallo`, `shas_completos` y el modulo
`tests_stdin`) no introduce ningun fallo que se haya podido demostrar. Nota
no bloqueante, sin tabla: la diferencia de texto del diagnostico cuando git
falla sin stderr (ahora `sin mensaje` por `primera_linea_error`, antes
cadena vacia) no afecta a la logica y no se considera hallazgo.

Se ejecuto para llegar a esta conclusion:
- Verificacion de la ronda: `cargo xtask verificar --perfil rapida` en verde
  (exit=0) sobre 4589b99. El diff de `crates` y `docs` entre 4589b99 y
  0d318d5 es vacio (`git diff --stat` sin salida), asi que el codigo revisado
  es identico. Log: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\verif-127-r2.log`
  (termina con "xtask: verificacion Rapida en verde"). No se repitio la suite.
- Tests concretos: `cargo test -p tc2 --lib --locked tests_stdin` sobre el
  clon en 0d318d5: 2 passed, 0 failed (`shas_completos_exige_uno_por_fichero`,
  `con_entrada_da_error_si_git_falla`).
- `cargo clippy -p tc2 --all-targets --locked`: exit 0, sin avisos.
- `cargo fmt --check`: exit 0.
- Mutacion: no ejecutada. Se razona que `shas_completos` queda cubierto por
  los casos `None` del test (`"a\n"` con n=2, `"a\nb\nc\n"` con n=2) y que
  `con_entrada` por el `expect_err` de los tres casos de error. Se deja
  constancia de que es lectura de las aserciones, no mutacion ejecutada.

## Alcance

- Ficheros revisados: `crates/tc2/src/git/estado.rs` (diff de la correccion:
  `con_entrada`, `shas_completos`, `construir_commit`, modulo `tests_stdin`).
  El cambio en `docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md` es
  solo texto y no casa con los patrones de esta skill.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada con `#[wasm_bindgen]` ni de
  tipos que crucen el limite. El diff no toca codigo que se exporte a
  WebAssembly, asi que no se pide llamador ni `.d.ts`.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-127-r2-rust`, en
  0d318d505400cf903b38867e6846e8355967785c (borrado al terminar la revision).
- Build: `cargo test --locked` de tc2 compila limpio en nativo. No se compilo
  para `wasm32` porque el diff no toca el crate de WebAssembly.
- Formato y lints: `cargo fmt --check` exit 0; `cargo clippy -p tc2
  --all-targets --locked` exit 0.
- Suite: la de la ronda (`cargo xtask verificar --perfil rapida`, exit 0 sobre
  4589b99, codigo identico). Linea base: la misma pasada, citada arriba.
- Casos construidos: las aserciones de `tests_stdin` ejercitan git real en un
  repo vacio: `hash-object --stdin-paths` sobre una ruta inexistente (error con
  `git=hash-object`), `update-index` con entrada mal formada (error con
  `git=update-index`), `hash-object` sobre un fichero existente (un sha de 40
  caracteres) y `shas_completos` con uno, dos y cero shas esperados.
- Mutacion: no ejecutada (ver arriba).
- Medidas: no aplican al diff.

## Lecturas fuera del contexto permitido

- `tareas/task-workflow/revision.md` (en la skill, `skills/task-workflow/revision.md`): reglas comunes de la revision y formato del veredicto.
- `crates/tc2/src/git/mod.rs` (lineas 55-85): definicion de `git_fallo` y `correr`/`exigir`, para verificar la firma que la correccion reutiliza.
- `crates/tc2/src/git/puerto.rs` (`primera_linea_error`, lineas 140-150): para verificar el texto por defecto del diagnostico que cambia.
- `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\verif-127-r2.log` (cola): para citar el resultado de la suite de la ronda.
