# Informe de revision — TASK-050 (ronda 1)

- Commit revisado: 90640c05dc759042baa565655060c3224b753c6f
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:820 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:409 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/hallazgos/mod.rs:641 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `plano` no neutraliza autoenlaces, menciones ni caracteres de formato Unicode

La doc de `plano` promete que el texto del modelo "no puede abrir tablas, enlaces, imagenes ni HTML" en el check. Reproduccion: respuesta con `descripcion` = `<U+202E>evil <U+200B>zero @user http://x.y $x$ :smile: ñ` validada con `--resumen`. La celda sale tal cual en la tabla. Quedan sin escapar: URL desnuda (GitHub la autoenlaza), `@usuario`, `$...$` (matematicas en Markdown de GitHub), `:codigo:` (emoji) y los controles de formato Unicode Cf (U+202E invierte la direccion del texto; U+200B es de ancho cero), porque `char::is_control` solo cubre Cc. Impacto bajo (no abre tablas, imagenes ni HTML; que una mencion en un check notifique no esta verificado), pero el modelo lee un diff de un PR ajeno y su texto es dato no confiable (F6-7). Sugerencia: descartar tambien los Cf (U+200B-U+200F, U+202A-U+202E, U+2066-U+2069, U+FEFF) y escapar `@`, `$` y `:` o romper `://`.

### MEN-2 — `Diff::leer` no decodifica rutas entre comillas con escapes octales de git

`ruta_nueva` quita las comillas pero deja los escapes: con `core.quotePath=true` (por defecto) git escribe la cabecera `+++ "b/src/\303\261.rs"` para `src/ñ.rs`. Reproduccion con ese diff y un hallazgo con `"fichero":"src/ñ.rs"` linea 2 (borde valido): `aceptados=0 rechazados=1 motivos=Q:linea-fuera-del-diff`. Solo se acepta si el modelo copia el texto escapado literal (probado: aceptado). Las instrucciones piden "la ruta tal como sale en la cabecera", asi que un modelo literal pasa, pero uno que escribe la ruta real (lo natural) se rechaza por un motivo que no es suyo. Afecta solo a ficheros con nombre no ASCII o con comillas o tabuladores. Sugerencia: decodificar los escapes (`\ooo`, `\"`, `\\`, `\t`, `\n`) o pasar `-c core.quotePath=false` en el `git diff` del workflow.

### MEN-3 — Hueco de test: el `trim` de `paso_rojo` no esta cubierto

Mutacion: `p.nombre == paso.trim()` por `p.nombre == paso.as_str()` en `validar_uno`: `cargo test -p tc2 --lib hallazgos` (7 ok) y `--test hallazgos` (3 ok) siguen en verde. Ningun test fija que `" clippy "` se acepte; `paso_rojo_se_contrasta_con_el_informe` solo prueba `clippy` y `fmt` exactos. Sugerencia: anadir ese caso al test.

## Alcance

- Ficheros revisados: crates/tc2/src/hallazgos/mod.rs, crates/tc2/src/hallazgos/orden.rs, crates/tc2/src/i18n/catalogo/cli.rs, crates/tc2/src/i18n/catalogo/hallazgos.rs, crates/tc2/src/i18n/catalogo/mod.rs, crates/tc2/src/lib.rs, crates/tc2/tests/hallazgos.rs, xtask/src/workflows.rs (el test; el YAML solo se ojeo para saber como se obtiene el diff y donde se publica el resumen).
- Contexto adicional pedido: fixture crates/tc2/tests/fixtures/hallazgos/diff.txt y, de .github/workflows/revision-ia.yml, solo las lineas de generacion del diff y publicacion del resumen (para juzgar MEN-1 y MEN-2).
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]`.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-050-r1-a\repo`, rama feature/task-050-f6-4-busqueda-de-errores (HEAD a50ce03, contiene 90640c0), `CARGO_TARGET_DIR` dentro de esa carpeta. Carpeta borrada al terminar.
- Build: `cargo build -p tc2 --locked` nativo, ok. El diff no tiene objetivo wasm32.
- Formato y lints: `cargo clippy -p tc2 -p xtask --all-targets --locked -- -D warnings` sin avisos.
- Suite: `cargo test -p tc2 --test hallazgos` 3 ok; `cargo test -p tc2 --lib hallazgos` 7 ok; `cargo test -p xtask` 29 ok. No se lanzo la suite completa (indicado).
- Casos construidos (binario `tc2 hallazgos validar`): bloque json dentro de un cerco exterior de 4 acentos (`bloque-repetido`, todo rechazado: estricto pero coherente), cercos `~~~` (`sin-bloque`), `JSON` en mayusculas, CRLF y cerco dentro de lista (aceptados), JSON truncado, linea 4294967295 / 4294967296 / -1 / 5.0 / "5" (fuera-del-diff o hallazgo-invalido, sin panic), Unicode (ñ, emoji, combinante, bidi, ancho cero: ver MEN-1), ruta con espacio y tabulador final (ok), ruta entre comillas con escapes octales (MEN-2), diff sin prefijos (ok), cabecera de trozo 4294967295,4294967295 (sin desbordamiento), linea de contexto vacia sin espacio inicial (el trozo se corta; `git diff` siempre pone el espacio, no se reporta), mutante con `linea_original` distinta por espacios internos o por mayusculas (`mutante-no-coincide`), `mutacion` que solo cambia espacios internos (aceptada, defendible), mutante sobre linea de contexto (aceptado), `paso_rojo` sin informe (`paso-rojo-no-esta`). Tamanos: 1000 hallazgos validos (162 KB) dan 100 aceptados y `#100:fuera-de-tope` en 0,12 s; 100000 corchetes anidados dan `json-invalido` (limite de recursion de serde_json, sin desbordar la pila); cadena de 300 KB da `error` (supera el tope de 256 KiB; el workflow corta antes a 64 KiB); 100000 elementos `3` dan 101 rechazos y la linea `motivos=` queda acotada; diff de 6 MB da error de tamano; respuesta binaria o vacia da `sin-bloque`.
- Mutacion (sobre `hallazgos/mod.rs`, lib e integracion): quitar el `trim` de `linea_original` (roja), quitar mutante-sin-cambio (roja), quitar borde-sin-diferencia (roja), `Rojo` por `true` (roja), quitar `linea == 0` (roja), quitar id-repetido (roja), duplicar el tope (roja), cambiar el escape de `|` (roja). Sobrevive el `trim` de `paso_rojo` (MEN-3). Los tests de integracion `--test hallazgos` no matan ninguna de esas mutaciones por si solos (dependen de los de la lib); es aceptable porque cubren de extremo a extremo los criterios 1 y 2.
- Medidas: no aplican (sin bucles calientes; todo es lineal y acotado por topes de entrada).

## Revisado sin hallazgos

- `bloque_json`, `validar`, `evidencia_completa`, `Diff::leer` (conteo de trozos, `++` dentro de un trozo, ficheros borrados, diff cortado) y los topes: sin panics ni indexados peligrosos (`&l[1..]` y `t[graves..]` cortan en limite de caracter porque el primer byte es ASCII).
- `orden.rs`: argumentos, topes de lectura con `take(tope + 1)`, codigos de salida (ok con rechazos, 2 uso, 1 lectura), `--informe` ilegible degradado a `sin-informe|grande|invalido` sin romper.
- Catalogo i18n y `lib.rs`: cableado correcto, `USO` actualizado en es y en.
- `xtask/src/workflows.rs`: el test pasa y sus mutaciones nuevas dan el error esperado; sin hueco evidente.
- Nota sin hallazgo: un hallazgo rechazado por evidencia ya consume su id, y otro posterior con ese id sale `id-repetido`; es consistente con el orden de comprobaciones documentado.
