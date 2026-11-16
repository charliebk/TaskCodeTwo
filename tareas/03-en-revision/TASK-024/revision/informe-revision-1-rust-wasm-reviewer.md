# Informe de revision — TASK-024 (ronda 1)

- Commit revisado: a6713930b5f17122ab4d850a1293fc185328c47d
- Revisor: general-purpose sonnet con rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/plataforma/github.rs:329 | implementacion |
| MEN-1 | MENOR | aceptado | crates/tc2/src/git/puerto.rs:137 | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/config/mod.rs:400 | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/lib.rs:82 | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/git/puerto.rs:236 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — Un «y» en español se cuela en el diagnostico en ingles (permiso de GitHub)

`estado_puertas` de GitHub pasa el permiso como `&'static str` (`"pull_requests:read y checks:read"`,
github.rs:329). `Permiso::from(&str)` lo trata como `Alcance`, «igual en todos los idiomas», y lo
inserta tal cual en `SIN_PERMISO`. Pero esa cadena lleva una palabra en español («y»), asi que el
mensaje en ingles queda mezclado. El test de guarda no lo ve porque solo busca `.diagnostico("`,
`format!` y `Mensaje::externo(`, y el literal no pasa por ahi.

Reproduccion (prueba desechable en el clon, mock HTTP de `tests/comun`: `GET /repos/o/r/pulls/12`
responde 403 `Resource not accessible by integration`; se llama a `GitHub::estado_puertas(Rol::Agente, 12, ..)`
y se imprime `diagnostico_en(Idioma::Es / Idioma::En)`):

```
ES: Some("el token de agente no tiene permiso: necesita pull_requests:read y checks:read (GitHub: «Resource not accessible by integration»)")
EN: Some("the agente token lacks permission: it needs pull_requests:read y checks:read (GitHub: «Resource not accessible by integration»)")
```

Salen dos restos en español en el texto en ingles: el «y» (defecto) y el nombre de rol «agente»
(`rol.nombre()`, que es el identificador de `rol=agente` de las claves; si es decision, basta
documentarlo). Arreglo posible: dos alcances separados («pull_requests:read» y «checks:read») unidos
con una entrada del catalogo («{a} y {b}» / «{a} and {b}»), o `Permiso::Descrito` con una entrada
`PERMISO_PUERTAS_GITHUB`. Conviene ademas que la guarda cubra los `Permiso::Alcance` con espacios.

### MEN-1 — `primera_linea_error()` sigue devolviendo «sin mensaje» en español

Limitacion que ya declara el implementador. Severidad: MENOR. Evidencia: `git/puerto.rs:137`
(`.unwrap_or("sin mensaje")`). Se usa en 10 sitios (`estado/montaje.rs:159,165,173`,
`estado/transaccion.rs:285,290,605`, `git/mod.rs:67`, `git/remoto.rs:180,181,233`) y todos acaban
en un diagnostico traducible, asi que con `TC2_IDIOMA=en` y git fallando con stderr vacio saldria
`git rev-parse failed: sin mensaje`. Solo ocurre con un fallo de git sin stderr (git muerto o
cortado, caso raro: git casi siempre escribe algo), no cambia stdout, claves, `motivo` ni codigo
de salida, y el espanol queda identico a develop. Ya existe la entrada `estado::SIN_MENSAJE`
(la usa `transaccion::denegada`, `transaccion.rs:439`), por lo que el arreglo es directo:
devolver `Option<String>` o un `Mensaje` y usar esa entrada. Aceptable dejarlo como limitacion
documentada.

### MEN-2 — Ningun test comprueba que todos los huecos `{nombre}` se rellenan

`catalogo_completo` comprueba que es y en llevan los mismos huecos, pero nada comprueba que cada
llamada los rellene. Un `.con(..)` olvidado deja el literal `{ruta}` en la salida.

Mutante (en el clon): en `config/mod.rs:400` quitar `.con("ruta", RUTA)` de
`i.diagnostico(diag.con("ruta", RUTA).con_mensaje("arreglo", arreglo))`.
Comando: `cargo test -p tc2` completo. Resultado: **0 tests en rojo** (el mutante sobrevive; los
tests de config solo hacen `contains` de un fragmento). Hoy no hay ningun hueco sin rellenar:
lo comprobe con un script de analisis estatico sobre las 216 entradas y sus usos (las 30 alertas
resultaron ser ayudantes que rellenan el hueco en otro sitio: `con_valor`, `con_cita`, `Permiso::en`,
`forma_inesperada`). Es una red que falta, no un defecto actual. Sugerencia: un test que dispare
los mensajes mas comunes y afirme que la salida no contiene `{`, o un `debug_assert!` en
`Mensaje::texto` cuando un hueco queda sin valor.

### MEN-3 — El camino en ingles de `new` y `actualizar-rama` no tiene test

`tests/cli.rs` solo prueba el ingles con una orden desconocida y con `guardar`; el test unitario
`mismo_informe_dos_idiomas` prueba `salida_en` en general, pero no el cableado de cada rama de
`ejecutar_en`.

Mutante: en `lib.rs:82` cambiar `orden_new(resto).salida_en(json, idioma)` por
`orden_new(resto).salida(json)`. Resultado: `cargo test -p tc2` completo, **todo en verde** (el
mutante sobrevive; `tc2 new` ignoraria `TC2_IDIOMA`). Los mutantes equivalentes sobre `guardar` y
sobre el borde si se detectan (ver Reproduccion). Sugerencia: anadir a `tc2_idioma_en_...` un
`tc2_en("en", &["new"])` y `["actualizar-rama", "x"]`.

### MEN-4 — `ErrorGit::Prohibido` lleva texto en español dentro del argumento

`git/puerto.rs:236,255,261,311` construyen `Prohibido(format!("{} sin indice temporal", ..))`,
`"{} con indice temporal"`, `"{} con indice relativo"` y `"{} {a}"`. Ese texto es el hueco
`{argumento}` de `ARGUMENTO_PROHIBIDO`, asi que en ingles saldria
`argument not allowed for the layer: restore sin indice temporal`. Son comprobaciones de
invariantes de la propia capa Git (solo se disparan por un fallo de programacion, no por entrada
de la persona), por eso MENOR. Aun asi son literales de cara al diagnostico fuera del catalogo y
la guarda no los detecta.

## Alcance
- Ficheros revisados: los 38 ficheros `.rs` de la peticion (src de `alta`, `config`, `contrato`,
  `estado`, `git`, `i18n`, `plataforma`, `lib.rs`, `main.rs` y `tests/cli.rs`, `tests/comun/ops.rs`,
  `tests/config.rs`). `Cargo.toml` y `Cargo.lock` no cambian.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: no aplica; el diff no toca ningun `#[wasm_bindgen]` ni tipos que crucen a JS.
  Es un crate nativo (CLI).
- Cambios de tipo en la API publica de la biblioteca (`Fallo`, `ErrorConfig.mensaje`,
  `Informe.diagnostico`, `FalloHttp::Rechazada(&'static Texto)` y la variante nueva `CuerpoGrande`,
  `Base::nueva -> Result<_, &'static Texto>`): se miraron los usos dentro del repo; solo los
  consumen `src/` y `tests/` del propio crate y compilan y pasan.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-024-r1`, `git clone` del repo y `git checkout a6713930b5f17122ab4d850a1293fc185328c47d` (cabeza detached). El repo original no se toco. Borrado al terminar.
- Build: `cargo test -p tc2` compila el crate y los tests en nativo. No hay objetivo `wasm32`.
- Formato y lints: `cargo fmt --check -p tc2` = 0 salida, exit 0. `cargo clippy -p tc2 --all-targets --locked -- -D warnings` = exit 0.
- Suite: `cargo test -p tc2` (una vez, sin `--ignored`), todo en verde: 51 tests de lib (incluidos `i18n::tests::catalogo_completo`, `resolucion_de_idioma`, `huecos_y_sustitucion`, `i18n::guarda::ningun_literal_fuera_del_catalogo`, `contrato::tests::mismo_informe_dos_idiomas`), cli 3/3 (incluido `tc2_idioma_en_da_el_diagnostico_en_ingles_con_el_mismo_codigo`), config 8/8, estado_bordes 9/9, y el resto de tests de integracion en verde. Sin `sinred` intermitente en esta ejecucion. Las suites posteriores a cada mutante se corrieron aparte (ver abajo).
- Casos construidos:
  - Binario `target/debug/tc2.exe` con `TC2_IDIOMA=es` y `en` fuera de un repo y con las ordenes `guardar`, `guardar --json m`, `guardar m f.txt`, `actualizar-rama`, `actualizar-rama x`, `actualizar-rama --json`, `new`, `new --json`, `new a b`, `new Titulo`, orden desconocida y sin orden: codigo de salida igual en los dos idiomas en todos; stdout identico byte a byte en los no `--json`; con `--json` solo cambia el campo `diagnostico` (`comando`, `resultado`, `codigo`, `claves` identicos, p. ej. `"claves":{"motivo":"sin-rutas"}`). Resolucion por entorno: `LANG=en_US.UTF-8` da ingles; `LC_ALL=es_ES` con `LANG=en_US` da español.
  - Comparacion de textos en español contra `origin/develop` (1d7a69b): script que extrae las 216 entradas `es:` del catalogo y comprueba que cada una tiene un literal igual en develop, ignorando los huecos. Sobre 11 sin coincidencia directa, las 11 se revisaron a mano y son recomposiciones identicas (`FALTA_LA_ORDEN`, `ORDEN_DESCONOCIDA`, `DIAGNOSTICO_CON_CLAVE`, `FORMA_O_FORMA`, `NUNCA_TODO_EL_ARBOL`, las tres `DENEGADA_*`, y tres de git/estado.rs). Sentido inverso: 19 literales de develop sin equivalente, todos recomposiciones (los dos de `head_cambiado` y `sin-commits` ahora llevan `cita` vacia). Resultado: **el español no cambia** respecto a develop.
  - Esquema JSON: `tests/config.rs::esquema_publicado_es_el_generado` pasa; `Regla::texto` usa `en_idioma(Idioma::Es)`.
  - Busqueda de literales de cara a la persona fuera del catalogo (script sobre `src/` sin los modulos de tests): solo quedan los de IMP-1, MEN-1 y MEN-4, las descripciones del esquema JSON (español por decision) y los mensajes de `tarea/` (`ErrorTarea`), que no pasan por el CLI en este commit.
  - Huecos sin rellenar: script estatico sobre las 216 entradas (ver MEN-2): ninguno real.
- Mutacion (cada mutante sobre el clon, `git checkout -- .` despues):
  - `SIN_IDS` con `en: ""` -> `catalogo_completo` en rojo: «falta el texto en ingles».
  - `SIN_IDS` con `es: ""` -> `catalogo_completo` en rojo: «falta el texto en espanol».
  - `MONTA_PRIMERO` con `{carpet}` en vez de `{carpeta}` en el ingles -> en rojo: «los huecos no coinciden entre idiomas».
  - `SIN_IDS` con un hueco `{x}` extra solo en el ingles -> en rojo (huecos).
  - `SIN_IDS` con el ingles igual al español -> en rojo (sin traducir).
  - `Mensaje::externo("hola")` en una funcion nueva de `alta.rs` -> guarda en rojo («usa `Mensaje::externo("`» y «usos distintos de EXTERNOS»).
  - `Mensaje::externo(format!(..))` partido en dos lineas, y `Mensaje::externo(x.to_string())` -> guarda en rojo por el recuento (`EXTERNOS`).
  - `.diagnostico(format!("hola"))` nuevo -> no llega a la guarda: no compila (`Mensaje: From<String>` no existe), que es mejor red que el test.
  - `de_entorno` con el orden `[TC2_IDIOMA, LANG, LC_ALL, LC_MESSAGES]` -> `resolucion_de_idioma` en rojo; `"es" => Some(Self::En)` -> en rojo.
  - `main.rs` con `let idioma = Idioma::Es;` -> `tests/cli.rs` en rojo (`tc2_idioma_en_...`).
  - `orden_new(..).salida(json)` en vez de `salida_en` -> suite entera en verde (MEN-3).
  - Quitar `.con("ruta", RUTA)` del diagnostico de config -> suite entera en verde (MEN-2).
- Medidas: no aplican.

## Revisado sin hallazgos
- `i18n/mod.rs`: `Mensaje::texto` no vuelve a sustituir lo que traen los valores, un `{` suelto o un nombre con caracteres no validos se deja tal cual, el recorte por indices cae siempre en `{`/`}` (ASCII), no hay `unwrap`, `expect` ni indexado que pueda dar panic; los mensajes anidados (`con_mensaje`) se traducen en el mismo idioma y no hay recursion infinita posible (el arbol de argumentos es finito).
- `i18n/guarda.rs`: funciona (ver Mutacion). Observacion menor, sin hallazgo: recorta el fichero en `#[cfg(test)]\nmod tests`, asi que un literal anadido despues de ese modulo no se mira; no es un caso realista en este repo.
- `contrato.rs`: `json()`/`salida()` conservan el español por defecto, `json_en`/`salida_en` solo traducen el diagnostico; stdout, claves, `motivo` y codigo no dependen del idioma (comprobado con el binario).
- Tipos cambiados: `Fallo::Rechazada`/`Error` (`de_fallo` conserva `«{motivo}: {diagnostico}»` en español), `ErrorConfig` (el orden y el contenido de `config rechazada` en español coinciden con develop; `0|1` errores sin sufijo, `n` errores con « (y n-1 error(es) mas)»), `FalloHttp` (la variante nueva `CuerpoGrande` mantiene `motivo=respuesta-invalida` y el mismo texto en español), `Permiso` (salvo IMP-1), `ErrorGit` (`Display` sigue dando el texto en español del catalogo).
- Las traducciones al ingles de los 216 textos se leyeron entera: sin huecos que no cuadren, sin frases incorrectas, sin codigos de salida ni claves traducidas. Matiz sin hallazgo: «the agente token» / «the persona one» / «a derivado of tipo genera» mantienen el nombre de rol o de clave de la configuracion (identificadores), coherente con las claves que no cambian.

## Correcciones de la ronda 1 (quien implementa)

- IMP-1 corregido: el permiso pasa a la entrada `plataforma::PERMISO_PR_Y_CHECKS` («pull_requests:read and checks:read» en ingles).
- MEN-2 corregido: `Mensaje::texto` lleva un `debug_assert!` si un hueco queda sin valor. El mutante de `config/mod.rs:400` (quitar `.con("ruta", RUTA)`) deja 2 tests de `tests/config.rs` en rojo.
- MEN-3 corregido: test `new_y_actualizar_rama_en_ingles_mismo_stdout_y_codigo` en `lib.rs`.
- MEN-4 corregido: `ErrorGit::Prohibido` lleva un `Mensaje`, y las tres llamadas con indice usan las entradas `git::SIN_INDICE_TEMPORAL`, `CON_INDICE_TEMPORAL` e `INDICE_RELATIVO`.
- MEN-1 aceptado: solo sale cuando git falla con stderr vacio. Arreglarlo exige cambiar a `Mensaje` las ~15 firmas por las que hoy pasa el detalle de git como `String` (`Intento::Fallo`, `Intento::Local`, `error_local`, `denegada`, `sin_red`...). Queda anotado como limitacion conocida en el Resultado.
