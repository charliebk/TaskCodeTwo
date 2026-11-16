# Informe de revision — TASK-077 (ronda 1)

- Commit revisado: eebf5f753da191740858ecfa9eea2af9c918b956 (la rama avanza un commit solo con la peticion, 7a8d6e5; mismo codigo)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto que propone el revisor: **aprobada** (0 CRITICO, 0 IMPORTANTE, 3 MENOR).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/plataforma/personas.rs:194 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/identidad.rs:218 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/identidad.rs:151 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `correos` se comparan sin tener en cuenta mayusculas no ASCII; `login` si

`personas.rs:183` compara logins con `to_lowercase()` (Unicode) y `personas.rs:194` y `:239` comparan `correos` con `eq_ignore_ascii_case`. Reproducido (test temporal en el clon): una lista con `correos: ["É@x"]` en una persona y `correos: ["é@x"]` en otra se acepta (`Ok(2)`), mientras que `login: "É"` y `login: "é"` se rechazan. La promesa del plan es que un correo de la 0.16 solo puede ser de una persona; con direcciones con letras no ASCII la unicidad no se cumple y `por_correo` devolveria la primera que case. Efecto pequeno (correos con tildes o eñes en la parte local son raros), por eso MENOR. Arreglo: usar el mismo plegado (`to_lowercase`) en los tres sitios.

### MEN-2 — la cache se lee entera sin tope de tamano ni de longitud del login

`identidad.rs:218` hace `read_to_string` sin limite y `Asignado::persona` / `login_valido` no acotan la longitud. Reproducido: una `identidad.yml` con un `login` de 50 000 000 de caracteres se lee como valida (`Some(50000000)`), se clona en `Cuenta` y acabaria escrito en `asignado_a`. Quien puede escribir la cache ya puede hacerse pasar por otra persona de la lista (riesgo aceptado en el plan), asi que no abre nada nuevo; es robustez: acotar a unos cientos de bytes el fichero y el login (GitHub limita a 39, GitLab a 255).

### MEN-3 — una cuenta que no es `User` (Organization, Mannequin...) sale como «es un bot»

`identidad.rs:151` rechaza todo `tipo != "User"` como `NoAsignable::Bot` y el mensaje `CUENTA_BOT` dice «es un bot». El rechazo es correcto (nunca se asigna), solo el texto y `cuenta=bot` no son exactos para los tipos que no son bot. Por lectura del codigo; no hay un caso de la API que lo dispare en la practica con un token de usuario normal.

## Alcance

- Ficheros revisados (los 12 del diff de dominio): `git/estado.rs`, `i18n/catalogo/{cli,identidad,mod}.rs`, `identidad.rs`, `lib.rs`, `plataforma/{mod,personas}.rs`, `tarea/{mod,modelo}.rs`, `tests/identidad.rs`, `tests/tarea.rs`.
- Contexto adicional pedido: ninguno aparte de lo anotado abajo.
- Limite JS-WASM: sin cambios; el diff no toca nada exportado con `#[wasm_bindgen]` ni hay objetivo `wasm32` en estos crates.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-077-r1-rust`, rama `feature/task-077-f2-13-identidad-de-la-persona-desde-la-p` (HEAD 7a8d6e5). Borrado al terminar.
- Build: `cargo test -p tc2 --locked` compila sin errores ni avisos.
- Formato y lints: `cargo fmt --check` sin diferencias; `cargo clippy --all-targets --locked -- -D warnings` sin avisos.
- Suite: `cargo test -p tc2 --locked`: 21 suites, 153 tests, 0 fallos (incluidos `tests/identidad.rs` y `tests/tarea.rs`). No hubo cuelgues de 127.0.0.1 que obligaran a repetir (la suite `actualizar_cli` tardo 176 s, ya era asi de lenta). En las pasadas de mutacion un test de `identidad` tardo 17 s, coherente con el cuelgue conocido; no afecta al resultado.
- Casos construidos (test temporal `tests/rev_extra.rs` en el clon, no commiteado):
  - Cache: vacia, claves duplicadas, campo de mas, `id: 0`, `verificado` negativo, ancla YAML, binario, no UTF-8, `login` de 50 MB, ruta de la cache que es un directorio, padre de la cache que es un fichero. Todas dan `None` (se trata como «no hay ninguna guardada») o un `Err` de `guardar` sin `panic` y sin dejar el temporal (`restos: ["identidad.yml"]`). Salvo el login de 50 MB (MEN-2).
  - Ida y vuelta de la cache con `id = u64::MAX` y login con `"`, `\` y `é`: igual.
  - `personas.yml`: `id` y `login` repetidos (nombra las dos entradas), `id` 0, flotante, ancla que repite una entrada, documento multiple, nombre en blanco: rechazados. `id` = `u64::MAX` y lista vacia: aceptados. Correo repetido sin distinguir mayusculas ASCII: rechazado. Correos con mayusculas no ASCII: aceptados (MEN-1).
  - `Heredado` no se puede construir fuera del modulo: `Asignado::Heredado(Heredado("x".into()))` desde un test externo da `E0423 cannot initialize a tuple struct which contains private fields`.
  - Worktree: `git rev-parse --path-format=absolute --git-common-dir` desde un worktree enlazado devuelve la carpeta `.git` del clon principal, asi que la cache es compartida.
- Mutacion (cada una sobre `identidad.rs`, restaurado despues):
  - quitar `u.tipo != "User"`: se ponen rojos `bot_nunca_asignado_y_sin_tty_es_agente` y `cuenta_fuera_de_lista_bot_o_web_flow_no_toca_la_cache`.
  - quitar `personas.contiene(u.id)` en `asignable`: los mismos dos.
  - quitar la caducidad en `vigente`: `cache_vigencia_y_rechazos` y `sin_red_no_toca_la_cache_y_sin_cache_se_pide_tc2_identidad`.
  - quitar la comprobacion de proveedor de la cache: se pone rojo `cache_vigencia_y_rechazos` (el test de integracion no lo cubre, pero el unitario si).
- Medidas: no aplican.

## Revisado sin hallazgos

- `Asignado`: `Heredado` solo lo construye el lector (campo privado, comprobado); `Persona{id,login}` pasa por `Asignado::persona` (id > 0 y login valido) tanto al construir como al leer; serializacion con `yaml::cadena` escapa comillas, barras, controles y separadores; ida y vuelta byte a byte probada en `tests/tarea.rs` con `u64::MAX`; la lectura rechaza claves de mas, `id` como texto, ceros a la izquierda, vacios.
- Comparacion por `id`: `la_lleva` y `misma_persona` solo miran `id`; probado mismo login con otro id y login renombrado con el mismo id.
- Cache: escritura en temporal con pid y `rename` (el temporal se borra si el `rename` falla); TTL de 7 dias con margen de 5 min para relojes adelantados y `saturating_*` (sin desbordes con `ahora = 0` o `verificado = u64::MAX`); proveedor guardado y comparado; el login sale de `personas.yml` si la persona esta; sin cache, caducada, de otra plataforma o fuera de lista se rechaza con `cache=<estado>` y el mensaje pide `tc2 identidad`. No guarda token alguno (solo `proveedor`, `id`, `login`, `verificado`).
- `quien_ejecuta`: con TTY consulta la API y devuelve `Persona`; sin TTY no toca la red (el test cuenta peticiones: 1 de la orden, 0 del agente) y devuelve `Agente` con `decidido_por: agente`.
- Nunca un `Bot`: `asignable` rechaza `tipo != "User"`, `web-flow`, fuera de lista y login no valido antes de construir `Cuenta`; la unica via a `Asignado` desde una cuenta pasa por ahi.
- Duplicados en `personas.yml`: `id`, `login` (sin distinguir mayusculas, Unicode) y `correos` (ASCII, ver MEN-1) rechazados nombrando las dos entradas con numero, login e id.
- Lectura de la base: `personas.yml` y `config.yml` salen de `origin/develop` con `ls-tree` + `show`, nunca del worktree; sin base o sin fichero, rechazo `sin-personas`; `config` ausente se trata como «detectar por host», una config invalida falla.
- Sin `unwrap`, `expect`, `panic!`, indexado ni `unsafe` en produccion de `identidad.rs` y `personas.rs` (el unico `unwrap_or` es seguro); el workspace tiene `unsafe_code = "forbid"` y `unwrap_used`/`expect_used`/`panic` en `warn`, con clippy `-D warnings` limpio.
- Tokens: el token solo vive en `Credenciales` (puerto `Plataforma`); la salida lleva `rol`, `id` y `login`; los errores pasan por `errores::sanear`; la URL de `origin` no se repite en ningun mensaje de `detectar`; la cache no tiene secretos.
- `url_origin` usa `git ls-remote --get-url origin` (sin red); un `origin` ausente (devuelve `origin`) se trata como sin remoto.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/config/mod.rs` (`de_base`, `texto_de_base`): para comprobar que `config.yml` se lee de `origin/develop`.
- `crates/tc2/src/git/estado.rs` (`dir_comun`, `leer_blob`): para la carpeta comun y la lectura del blob.
- `crates/tc2/src/plataforma/deteccion.rs` y `gitlab.rs` (`UsuarioApi`): para ver que la URL de `origin` no se repite en los mensajes y como GitLab marca un bot.
- `Cargo.toml` de la raiz: lints del workspace.

## No verificado

- Deteccion de TTY: `lib.rs:226` usa `std::io::stdin().is_terminal()`. En Git Bash con mintty (la consola que se usa en esta maquina) la entrada es una tuberia y `is_terminal` de `std` puede dar `false`; entonces `tc2 identidad` rechazaria a la persona con «necesita terminal». No se ha podido comprobar desde aqui (el revisor no tiene un terminal mintty); conviene que la persona lo pruebe a mano una vez. Tambien solo se mira la entrada estandar, no la salida.
- `origin/develop` local se usa sin `fetch`: una persona anadida por un PR recien fusionado no aparece hasta que se haga `fetch`, y el mensaje solo sugiere «pide que la anadan con un PR». Es por diseno (la orden no habla con el remoto salvo la API), solo se deja anotado.
