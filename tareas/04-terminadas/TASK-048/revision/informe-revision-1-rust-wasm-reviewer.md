# Informe de revision — TASK-048 (ronda 1)

- Commit revisado: a30688c63e099c28376a9d6dcd3f9ffd50f0a95d
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | aceptado (divergencia ya anotada en la adenda F6-2; pushurl distinto es raro y se trata con F6-3) | crates/tc2/src/revision/mod.rs:765 | implementacion |
| MEN-2 | MENOR | aceptado (seguimiento: doble que falle abrir_pr; orden correcto verificado por lectura) | crates/tc2/tests/revision.rs:63 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — la cabecera se limita a la URL de `remote.origin.url`, no a la de push

`url_origin` usa `git ls-remote --get-url origin`, que devuelve `remote.origin.url` y
no `remote.origin.pushurl`. `entorno_push` fija `http.<url>.extraheader` para esa URL.
Si un repo tiene `pushurl` distinta (https en otra forja, o ssh), git empuja a la
`pushurl`, la cabecera del agente no aplica y el push sale con la credencial local de la
persona: se pierde la identidad del agente sin aviso. No es un caso de la tarea (el
test usa `pushurl` justamente para redirigir al mock y pasa porque el doble inyecta la
cabecera a mano). Dado que el requisito es «push con la identidad del agente», lo
proporcionado es rechazar `review` (o usar `git remote get-url --push`) si hay `pushurl`
distinta. No lo reproduje contra una forja real; el razonamiento sale de la semantica de
git (`http.<url>.*` se casa con la URL a la que se empuja).

### MEN-2 — sin test de que un fallo de `abrir_pr` deja el estado en `en-curso`

El doble de `tests/revision.rs` siempre abre el PR bien. El orden push -> PR ->
transaccion esta cubierto para el fallo del push (`el_push_de_review_lleva_la_cabecera_del_agente`:
sin PR y con `en-curso`), pero ningun test falla si se mueve la transaccion antes de
`abrir_pr`. El codigo es correcto (`publicar` devuelve `Err(pr)` antes de la
transaccion, revision/mod.rs:637-646 y 770-777); es solo red de tests.

## Alcance
- Ficheros revisados: los 17 del dominio Rust de la peticion (git/puerto.rs, i18n/catalogo/*, lib.rs, plataforma/{entorno_git,github,gitlab,mod}.rs, revision/{mod,operacion}.rs y los tests).
- Contexto adicional pedido: `git/remoto.rs` (push, publicar) y `git/estado.rs` (url_origin), para seguir la ruta del push; ver abajo.
- Limite JS-WASM: sin cambios (no hay `#[wasm_bindgen]` en el diff).

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-048-r1-rust` en 999d68b (rama de la tarea; el diff de codigo es el de a30688c).
- Build: compilado por los propios tests (nativo); sin wasm32 (el crate es CLI).
- Formato y lints: `cargo clippy -p tc2 --all-targets --locked -- -D warnings` sin avisos. `cargo fmt` no ejecutado (la verificacion completa de la peticion ya lo cubre).
- Suite: `--test revision` 3/3, `--lib revision::` 4/4, `--lib plataforma::` 29/29, `--test identidades` 12/12, `--test toma` 13/13. La verificacion completa y los E2E de GitHub/GitLab se toman de la peticion (no reejecutables).
- Casos construidos: lectura del flujo de errores de `publicar` y `paso`; no se construyo una forja real para MEN-1.
- Mutacion: no ejecutada (proporcionalidad). Por lectura, el test del push fallido ya fallaria si se invirtiera push/transaccion; el de `abrir_pr` fallido no existe (MEN-2).
- Medidas: no aplican.

## Revisado sin hallazgos
- Token del agente: solo viaja por `GIT_CONFIG_VALUE_n` del entorno del hijo (`Git::derivado` -> `comando`), nunca en argv; el registro del puerto de la transaccion no lo ve (`derivado` reinicia el registro) y el test lo comprueba. Los diagnosticos de push usan stderr de git, que no imprime `extraheader`.
- Cabecera limitada a la URL del remoto; `entorno_push` solo acepta `https://` sin caracteres de control ni espacios, y rechaza antes de pedir el token (test con ssh, http, ruta local y `https://`).
- Orden push -> `abrir_pr` -> transaccion: el estado nunca pasa a `en-revision` sin PR; si la transaccion falla, la linea lleva `pr=` y repetir converge (`sin-cambios` con `Revisar` idempotente).
- Errores: sin `unwrap`/`expect`/`panic` en codigo de produccion del diff; los fallos salen como `Informe`. `Git::derivado` clona binario, entorno y config y acumula la numeracion de `GIT_CONFIG_*` como `con_entorno_extra`.

## Lecturas fuera del contexto permitido

- crates/tc2/src/git/remoto.rs (push/publicar/reintentos) y crates/tc2/src/git/estado.rs (url_origin): para comprobar que el push usa el puerto derivado y de que URL sale la cabecera.
- crates/tc2/src/git/puerto.rs `comando` y `separar_config`/`numerar_config`: para confirmar que el token va por entorno y no por argv.
