# Informe de revision — TASK-127 (ronda 4)

- Commit revisado: e7646219e90763898c5c0fb75a8298e8391f4486
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Sin hallazgos. Los dos puntos pendientes de la ronda 3 (la comprobacion de conteo de `hash-object` y su test) estaban ya `corregido` y siguen verificados. La correccion de esta ronda solo anade codigo de test:

- `Git::con_binario` (`crates/tc2/src/git/puerto.rs`, lineas 238-245) esta bajo `#[cfg(test)]` y `pub(crate)`: no entra en el binario de produccion ni en el limite JS-WASM, y no cambia el comportamiento de `Git::nuevo()`.
- `construir_commit_rechaza_shas_incompletos` y el git falso `git_que_pierde_shas` (`crates/tc2/src/git/estado.rs`, lineas 540-600) comprueban que, con dos ficheros y un solo sha, `construir_commit` devuelve `HASH_OBJECT_INCOMPLETO` y no llega a `commit-tree`. El prefijo que se busca en el registro (`git commit-tree`) coincide con el nombre real del subcomando (`Subcomando::CommitTree` = `"commit-tree"`), asi que la aserción negativa no es vacua.
- Guion de Windows (`.cmd`): `%5` es `hash-object` porque los argumentos son `-c color.ui=never -c core.quotepath=false <sub>`. Funciona: el test pasa.
- Guion de unix (`.sh`): comprobado a mano con `sh` en el clon (mismos argumentos): `hash-object` devuelve el sha y el resto se delega en `git "$@"`. El test no se ha ejecutado en unix (ver Reproduccion).
- Un fallo de escritura en el stdin del git falso (el `.cmd` no lee la entrada) se ignora en `lanzar_con_entrada` con `let _ =`. Eso es anterior a esta ronda, no lo introduce la correccion, y no afecta al test porque la entrada cabe en el buffer del pipe. No se abre hallazgo.

## Alcance
- Ficheros revisados: `crates/tc2/src/git/puerto.rs` (`Git::con_binario`) y `crates/tc2/src/git/estado.rs` (`git_que_pierde_shas`, `construir_commit_rechaza_shas_incompletos`, y el llamador `hashear`/`construir_commit` para el caso que rompe).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada. El diff de esta ronda no toca ningun `#[wasm_bindgen]` ni tipo que cruce el limite. No hay llamador TypeScript que revisar.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\TaskCodeTwo\task-127-r4-rust` (la misma carpeta aparece como `...\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Local\pruebas-claude\...` en la salida de cargo), checkout de `e7646219e90763898c5c0fb75a8298e8391f4486`. Se borra al terminar la revision.
- Codigo verificado: `git diff --stat cdbd5ba e764621 -- crates docs` vacio (mismo codigo que el verificado en la suite).
- Build: `cargo test` y `cargo clippy` compilaron `tc2` con `--locked` en el clon. Objetivo `wasm32`: no aplica al diff. Objetivo unix: no instalado en esta maquina (`rustup target list --installed` solo muestra `x86_64-pc-windows-msvc`), asi que el codigo de unix no se compila aqui; el guion `.sh` se valido a mano.
- Formato y lints: `cargo fmt --check` exit 0 (con `-p tc2` y con `--all`); `cargo clippy -p tc2 --all-targets` exit 0, sin avisos.
- Suite: no se repitio. Se cita la verificacion rapida en verde sobre `cdbd5ba`, log `$LOCALAPPDATA/pruebas-claude/TaskCodeTwo/verif-127-r4b.log` con `.fin` = `exit=0` y la linea final `xtask: verificacion Rapida en verde`. El log contiene `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` y `cargo test --workspace --locked`, y el test `construir_commit_rechaza_shas_incompletos` aparece `ok`.
- Tests dirigidos: `cargo test -p tc2 --lib --locked tests_stdin`: 3 passed, 0 failed (`hashear_exige_un_sha_por_fichero`, `con_entrada_da_error_si_git_falla`, `construir_commit_rechaza_shas_incompletos`), exit 0.
- Casos construidos: dos ficheros con un solo sha devueltos por el git falso (`construir_commit_rechaza_shas_incompletos`); `n=0` y `n=2` con un fichero (`hashear_exige_un_sha_por_fichero`).
- Mutacion: en `hashear` se cambio `if shas.len() == n {` por `if shas.len() >= 1 {` (acepta un sha incompleto). Resultado: `hashear_exige_un_sha_por_fichero` FAILED y `construir_commit_rechaza_shas_incompletos` FAILED (exit 101). La mutacion se revirtio con `git checkout` en el clon y el arbol quedo limpio.
- Medidas: no aplican (la correccion no toca rendimiento).

## Revisado sin hallazgos
- `Git::con_binario`: solo compila en tests, `..Self::nuevo()` deja el registro vacio y el entorno fijo igual que en produccion.
- Git falso en Windows y en unix: los dos guiones leen los mismos argumentos (`$5` / `%5`) y delegan en el git real en cualquier otra llamada.
- Aserciones del test: el diagnostico se compara con `Idioma::Es` contra el de `HASH_OBJECT_INCOMPLETO`, la ausencia de `commit-tree` se comprueba en el registro real.
- Ownership y errores en el codigo no-test de la correccion: no hay cambios de produccion, sin `unwrap`/`panic` nuevos fuera de tests; los `unwrap` del test estan en la fixture, no en el camino de produccion.

## Lecturas fuera del contexto permitido
- `tareas/03-en-revision/TASK-127/revision/informe-revision-3-rust-wasm-reviewer.md`: se leyo como ejemplo de como rellenar la plantilla; no es parte del diff revisado.
- `$LOCALAPPDATA/pruebas-claude/TaskCodeTwo/verif-127-r4b.log` y su `.fin`: se citaron como verificacion de la ronda, pedidos por la peticion.
- `crates/tc2/src/git/puerto.rs` y `crates/tc2/src/git/estado.rs` se leyeron fuera de los rangos del diff (`lanzar_con_entrada`, `comando`, `Subcomando::nombre`, `hashear`, `con_entrada`) para entender el llamador y el registro; son ficheros del contexto permitido.
