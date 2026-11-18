# Informe de revision — TASK-127 (ronda 3)

- Commit revisado: fa39e430c433bf9411d29cb9b108ee2daf7b55df
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Sin hallazgos. Las dos correcciones de la ronda 2 (IMP-1 y el test que dependia de `shas_completos`) quedan verificadas en el diff `eb0eb3d..fa39e43`, que no contiene cambios de codigo fuera de `crates/tc2/src/git/estado.rs`.

- IMP-1 (hash-object sin un sha por fichero) queda resuelto: `hashear` (`crates/tc2/src/git/estado.rs`, linea 306) devuelve `HASH_OBJECT_INCOMPLETO` cuando `hash-object --stdin-paths` no devuelve exactamente `n` shas, y `construir_commit` lo usa en lugar de la antigua `shas_completos`.
- Reproduccion del caso que rompe: con la comprobacion de conteo anulada (`if true || shas.len() == n`), el test `hashear_exige_un_sha_por_fichero` falla con panic en la linea 527. Con el codigo de la correccion, pasa. La mutacion se revirtio en el clon.
- No se encontro ningun fallo nuevo introducido por la correccion: el conteo se compara antes de emparejar rutas con shas, el error de git se propaga con `?` antes de la comprobacion y no hay `unwrap` ni `panic` en codigo no-test.

## Alcance
- Ficheros revisados: `crates/tc2/src/git/estado.rs` (funcion `hashear`, llamador `construir_commit` y test `hashear_exige_un_sha_por_fichero`).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada. El diff no toca ningun `#[wasm_bindgen]` ni tipo que cruce el limite (`hashear` es `fn` privada).

## Reproduccion
- Clon: `$LOCALAPPDATA/pruebas-claude/TaskCodeTwo/task-127-r3-rust`, checkout de `fa39e430c433bf9411d29cb9b108ee2daf7b55df`. Borrado al terminar la revision.
- Build: no se ejecuto un build aparte; `cargo test` y `cargo clippy` compilaron el crate `tc2` en el clon con `--locked`. Objetivo `wasm32` no aplica al diff (no toca codigo de WebAssembly).
- Formato y lints: `cargo fmt --check` exit 0; `cargo clippy --locked -p tc2 --all-targets` exit 0, sin avisos.
- Suite: no se repitio. Se cita la verificacion de la ronda 3 en verde, log `$LOCALAPPDATA/pruebas-claude/TaskCodeTwo/verif-127-r3.log` con `.fin` = `exit=0` y la linea final `xtask: verificacion Rapida en verde`. El diff de codigo `eb0eb3d..fa39e43` sobre `crates` y `docs` esta vacio, asi que es el mismo codigo verificado.
- Tests dirigidos: `cargo test --locked -p tc2 --lib tests_stdin`: 2 passed, 0 failed, exit 0.
- Casos construidos: un fichero con `n=1` (sha de 40 caracteres); `n=0` y `n=2` con un solo fichero (ambos deben devolver `HASH_OBJECT_INCOMPLETO`, el test lo comprueba con el diagnostico en `Idioma::Es`).
- Mutacion: anular la comprobacion de conteo en `hashear` (`if true || shas.len() == n`) pone el test `hashear_exige_un_sha_por_fichero` en rojo (panic linea 527, exit 101). Revertido con `git checkout` en el clon.
- Medidas: no aplican (la correccion no toca rendimiento).

## Revisado sin hallazgos
- Correccion de `hashear` frente a la version anterior `shas_completos`: mismo criterio de conteo, ahora con error de git propagado antes y diagnostico unico.
- Test `hashear_exige_un_sha_por_fichero`: cubre el caso feliz y los dos casos de conteo distinto, y comprueba que falla si se revierte la comprobacion.
- Test `con_entrada_da_error_si_git_falla` y el test de `hash-object` real (que ahora asserta longitud 40 directamente sobre `shas.trim()`): correctos y sin cambios de comportamiento.
- Ownership y errores: sin `unwrap` ni `panic` nuevos fuera de tests; el `expect` del test tiene mensaje.

## Lecturas fuera del contexto permitido
- `tareas/03-en-revision/TASK-127/revision/peticion-revision-3-rust-wasm-reviewer.md`: peticion de la ronda, obligatoria para el encargo.
- `tareas/03-en-revision/TASK-127/revision/informe-revision-3-rust-wasm-reviewer.md`: plantilla que pide la peticion.
- `$LOCALAPPDATA/pruebas-claude/TaskCodeTwo/verif-127-r3.log` y su `.fin`: para citar la verificacion en verde del commit, sin repetir la suite.
- `crates/tc2/src/git/estado.rs` lineas 269-370 leidas con `grep`/`sed` para ver `con_entrada` y el llamador `construir_commit`; el fichero esta dentro del contexto permitido.
