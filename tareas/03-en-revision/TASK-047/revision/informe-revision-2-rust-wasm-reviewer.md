# Informe de revision — TASK-047 (ronda 2)

- Commit revisado: 5f3521117b2005d83286d3bdf9cbd6d0c21ec125
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MENOR-1 | MENOR | corregido (5f35211) | crates/tc2/src/plataforma/credenciales.rs:274-282 y 252-265; conexion.rs:98-111 | implementacion |
| MENOR-2 | MENOR | corregido (5f35211) | crates/tc2/src/plataforma/credenciales.rs:510-526 (`id_valido`) | implementacion |
| MENOR-3 | MENOR | corregido (5f35211) | crates/tc2/src/plataforma/conexion.rs:112-116 (mapeo a sinred); tests de 404 y de margen existentes | implementacion |
| MENOR-4 | MENOR | corregido (5f35211) | crates/tc2/src/plataforma/app_github.rs:379-404 (`fallo_transporte`) | implementacion |
| MENOR-5 | MENOR | corregido (5f35211) | crates/tc2/src/git/puerto.rs:270-274, 461-490, 627-674 | implementacion |

Sin CRITICO ni IMPORTANTE abiertos. Los cinco MENOR de la ronda 1 estan corregidos y verificados. Ninguna correccion abre un fallo nuevo demostrable en el diff de esta ronda (detalle en `## Revisado sin hallazgos`).

Notas de verificacion sobre MENOR-3: la mutacion del mapeo a sinred (M6) deja en rojo `el_adaptador_traduce_sinred_al_acunar`. Las otras dos protecciones de la ronda 1 (M7, reinicio de la instalacion en 404; M12, `MARGEN`) tienen ahora test propio en la suite (`un_404_al_pedir_el_token_olvida_la_instalacion` y `el_token_que_vence_dentro_del_margen_se_renueva_y_el_de_fuera_no`); esta ronda solo comprobo que pasan, no las mute.

## Alcance
- Ficheros revisados: crates/tc2/src/git/puerto.rs, crates/tc2/src/plataforma/app_github.rs, conexion.rs, credenciales.rs, entorno_git.rs, github.rs, crates/tc2/src/i18n/catalogo/plataforma.rs, crates/tc2/tests/identidades.rs (con los tests que cubren cada hallazgo).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: no aplica. El crate es solo nativo y el diff no toca `#[wasm_bindgen]` ni tipos que crucen a JavaScript.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-047-r2-rust`, HEAD 8e556b5 (contiene la correccion 5f35211 y solo anade la peticion), `git status` limpio al final.
- Build: `cargo test -p tc2 --locked` compila sin errores (nativo; no se compilo `wasm32`, el crate no es WASM).
- Formato y lints: `cargo fmt --check -p tc2` sin diferencias; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` sin avisos.
- Suite: no se ejecuto la suite completa ni `cargo xtask`, por encargo. Se ejecutaron, sobre el commit revisado:
  - `cargo test -p tc2 --locked --test identidades`: 12/12 verdes.
  - `cargo test -p tc2 --locked --lib plataforma::`: 28/28 verdes (la ronda 1 tenia 27; el test nuevo es el de `id_valido`).
  - `cargo test -p tc2 --locked --lib git::`: 21/21 verdes.
  La peticion no traia verificacion en verde de la ronda, asi que esta es la linea base de la ronda.
- Casos construidos y lectura de cada hallazgo:
  - MENOR-1: `en_gitlab_la_app_cede_al_token_del_mismo_rol` pasa. Por lectura: `atar_github` solo se llama desde `GitHub::nuevo` (github.rs:336), asi que un `SinAtar` nunca llega a `obtener` en GitHub; el fallback al token del mismo rol solo actua en destinos que no son GitHub. Sin token del rol, `app-solo-github` nombra `TC2_TOKEN_<ROL>` (test de `de_entorno_con`, nunca el token de otro rol).
  - MENOR-2: `el_id_de_app_numerico_no_admite_ceros_a_la_izquierda` pasa. Por lectura: un id solo de digitos se acepta solo si `parse::<u64>()` lo reproduce tal cual (`0`, `123` si; `0123`, `00`, `99999999999999999999999` no). Client ids `Iv1.*` no cambian de forma.
  - MENOR-4: `fallo_transporte` cubre todas las variantes de `FalloHttp` (el `match` es exhaustivo y compila). Por lectura, `Tls` sale como `causa=tls` con el texto que manda al almacen de certificados; no se provoco un fallo TLS real.
  - MENOR-5: por lectura, el token viaja solo en `GIT_CONFIG_VALUE_<n>` del entorno del proceso hijo. `comando` solo registra `sub` y `args` en el registro (no el entorno), y `-c` no recibe el token. `numerar_config` toma `previas` de `GIT_CONFIG_COUNT` del proceso, que el hijo hereda (no esta en `ENTORNO_QUITADO`), asi que la numeracion coincide con lo que git lee. `con_entorno` y `con_entorno_extra` ya acumulan en el mismo orden, y el test `las_entradas_de_config_se_acumulan_y_se_numeran_tras_las_previas` lo comprueba en ambos ordenes.
- Mutacion:
  - M6 (MENOR-3): `if causa == causas::SINRED` -> `if false && causa == causas::SINRED` en conexion.rs:112. `cargo test -p tc2 --locked --test identidades`: rojo en `el_adaptador_traduce_sinred_al_acunar` (11 pasan, 1 falla). Revertido con `git checkout -- crates/tc2/src/plataforma/conexion.rs`; comprobado despues con grep y `git status` (limpio).
  - M7 y M12 de la ronda 1: no mutadas en esta ronda (ver nota en `## Hallazgos`).
- Medidas: no aplican (sin bucles calientes ni limite WASM en el diff).

## Revisado sin hallazgos
- Token fuera de argv y de registro (MENOR-5): el token solo va en variables de entorno del hijo; el registro guarda `git <sub> <args>`.
- `id_valido` (MENOR-2): el cambio es estricto y no rechaza ids validos reales.
- Permisos del token (`permiso_que_falta`, `permissions` opcional con `serde(default)`): un `null` o un campo ausente se acepta como antes; si GitHub devuelve permisos y falta uno pedido, sale `permisos-no-concedidos`. `write` cubre `read` como antes.
- PEM con BOM y texto antes de `-----BEGIN` (`leer_pem`): el BOM se quita, las lineas previas se saltan y, sin `BEGIN`, sale `clave-ilegible` (no hay panic). Los tests de la suite de identidades lo cubren.
- `repo_valido` movido a `github.rs` y reutilizado por `app_github.rs`: una sola regla, sin cambio de comportamiento para `GitHub::nuevo`.
- Sin `unwrap`/`expect`/indexado sobre datos de entrada en el codigo de produccion del diff; `unsafe` no hay.
- Dependencias: sin crates nuevos en el diff de la correccion.

## Lecturas fuera del contexto permitido
- crates/tc2/src/plataforma/http.rs: buscada la lista de variantes de `FalloHttp` (grep de nombres) para verificar el `match` de MENOR-4.
- crates/tc2/src/plataforma/conexion.rs y crates/tc2/src/git/puerto.rs: ademas leidos por rangos (ya permitidos).

## Notas fuera de la tabla (no abren ronda)
- Semantica de `con_entorno` (MENOR de documentacion): ahora acumula, como `con_entorno_extra`. Una llamada con `GIT_CONFIG_COUNT=0` y sin entradas ya no anula las `GIT_CONFIG_*` heredadas del proceso (antes si, porque se copiaba la variable tal cual). Ningun caller del repo usa `GIT_CONFIG_COUNT=0` para aislar; el doc de `con_entorno` ya dice que acumula, pero conviene una linea que lo diga para este caso.
- `tests/estado_casos.rs:518` pasa `GIT_CONFIG_COUNT`/`KEY_*` a traves de un camino de config que puede pasar por `con_entorno`; no se ejecuto (fuera del encargo), pero el cambio de numeracion lo afecta si el entorno del proceso ya trae `GIT_CONFIG_COUNT`. Recomendado correr `--test estado_casos` en la verificacion de la rama antes de cerrar.
