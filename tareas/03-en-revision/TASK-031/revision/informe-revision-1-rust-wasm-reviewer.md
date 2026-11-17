# Informe de revision — TASK-031 (ronda 1)

- Commit revisado: 8a54049c717b71525d0a180c8e00676686271db0 (HEAD de la rama: 9401218, solo anade la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/roadmap/mod.rs:112 | implementacion |
| MEN-1 | MENOR | abierto | crates/tc2/src/roadmap/mod.rs:263 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/roadmap/zona.rs:42 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — `roadmap --escribir` pisa en silencio la zona manual que otra persona publico antes

`Regenerar::aplicar` (mod.rs:108-114), con `editado = Some(t)`, devuelve siempre `Cambio::nuevo(ROADMAP, t)`
sin comparar con la base sobre la que la persona edito. Si un companero publico un cambio en la zona manual
entre medias, el `--escribir` de la segunda persona lo borra sin conflicto ni aviso.

Reproduccion (binario de la rama, remoto bare con dos clones A y B, ambos con `tareas/` montada en el mismo commit):
1. A edita `tareas/ROADMAP.md` (anade `NOTA DE A` bajo `## Hitos`) y ejecuta `tc2 roadmap --escribir`: `roadmap ok ... commit=c296b544254f`.
2. B (su carpeta sigue en el commit anterior) edita su copia (anade `NOTA DE B`) y ejecuta `tc2 roadmap --escribir`: `roadmap ok ... commit=d2c02d0063bb tareas=al-dia`.
3. `git show HEAD:ROADMAP.md` en B: solo contiene `NOTA DE B`; `NOTA DE A` ha desaparecido de lo publicado (queda solo en el historial de la rama).

Es el caso realista de equipo. Propuesta: en `aplicar`, si la zona publicada difiere de la base de la que parte la
edicion (el blob del HEAD de la carpeta montada) y de la de la persona, rechazar con un motivo propio
(`zona-divergente`) en vez de sobrescribir; o al menos avisarlo en la salida.

### MEN-1 — `--escribir` prepara (`git add`) antes de validar y deja el indice preparado si rechaza

`editado_en` (mod.rs:263 y siguientes) hace `git add -- ROADMAP.md` antes de comprobar la zona. Con zona rota
el comando devuelve `rechazada motivo=roadmap-roto` pero `tareas/` queda con `M  ROADMAP.md` preparado (el
propio test `zona_rota_no_bloquea_y_se_avisa` lo da por bueno al reescribir `" M "` a `"M  "`). No se pierde
contenido (el fichero de trabajo no se toca), pero la orden dice que no publica y deja el indice cambiado; lo
mismo si la transaccion falla despues (sin red, agotada). Mejor leer el fichero de trabajo, validar y solo preparar
al publicar, o restaurar el indice en el camino de error. Sin efecto en el resultado normal.

### MEN-2 — Las marcas de la zona manual se reconocen por prefijo, no por palabra completa

`es_marca` (zona.rs:42) usa `starts_with("<!-- manual:fin")`, de modo que una linea como
`<!-- manual:finalizado -->` o `<!-- manual:inicio2 -->` dentro de la zona de la persona cuenta como marca
(cierra o repite la zona). Es el comportamiento heredado de 0.16 y solo afecta a texto que la persona escribe
con esa forma; el resultado normal no cambia. Se anota por la promesa de "extraccion estricta". No reproducido
con el binario; se deduce del codigo.

## Alcance
- Ficheros revisados: crates/tc2/src/situacion.rs, siguiente.rs, roadmap/{mod,zona,generar,mermaid}.rs, estado/{mod,montaje,transaccion}.rs (cambio de firma de `Contexto::nuevo`), grafo.rs (`Hitos::del_roadmap`), `.gitattributes`, tests/roadmap.rs y la fixture `tests/fixtures/roadmap/referencia.md`.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni exporta tipos a JavaScript.

## Reproduccion
- Clon: C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-031-r1-rust, rama feature/task-031-f3-4-roadmap-la-vista-para-todos (HEAD 9401218). Borrado al terminar, junto con los repos de prueba `esc` y `esc2`.
- Build: nativo, con `cargo test` y `cargo clippy` (no hay objetivo wasm32 en este diff).
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` limpio, sin avisos.
- Suite: `cargo test -p tc2`: 27 binarios de test, 304 pruebas, 0 fallos.
- Casos construidos: (a) `tc2 new` con titulo `Con "comillas" ] --> %% <b>x</b> | barra` y con `<!-- manual:fin --> y #35; &amp;` mas backticks: tabla escapada (`\|`, `&lt;`), Mermaid con `#quot;`, `#gt;`, `#lt;`, `#35;`, `#96;`; `--comparar` da `ok` y `next` coincide. (b) `tc2 new` rechaza titulos con LF, CRLF o tab (`titulo-invalido`); un `tarea.md` con titulo de varias lineas o un `depende_de` hostil se lee como ilegible, asi que no llega al generador (prueba temporal en el clon, revertida): la segunda generacion es identica a la primera y la zona sigue extrayendose. (c) Edicion de la zona en `tareas/` y luego `new`: queda `tareas=con-cambios`, la edicion se conserva y un `--escribir` posterior la publica y deja la carpeta `al-dia`. (d) Dos clones (IMP-1).
- Mutacion: no se ejecuto `cargo mutants`; se leyeron los tests C2 (CRLF/LF, zona rota, idempotencia) y ejercitan el comportamiento, pero no se mutaron las protecciones.
- Medidas: no aplican (sin rendimiento ni limite WASM).

## Revisado sin hallazgos
- `situacion.rs` y `siguiente.rs`: `next` usa la misma `Situacion`; su salida solo gana `roadmap-roto=<motivo>` cuando la zona esta rota; con ROADMAP sano no cambia (los tests de `next` pasan).
- `zona.rs`: `extraer` recorre con `split_inclusive('\n')` y corta con `get(i..f)` sobre limites de linea (sin panico en UTF-8), devuelve la zona byte a byte con LF, CRLF y sin salto final; inicio sin fin, fin suelto, doble inicio y doble fin dan `Rota` distinta. `interior_tolerante` es coherente con `extraer` para un ROADMAP sano.
- Generador y Mermaid: `escapar` neutraliza `"`, `#`, `<`, `>`, backtick y caracteres de control (incluidos saltos de linea) dentro de etiquetas entre comillas; `id_de` normaliza a `[A-Za-z0-9_]`; limites de aristas y caracteres con nota; salida determinista (LF, sin fecha).
- `Contexto::nuevo` con generador: el montaje pasa `None` al inicializar, los demas llamadores pasan el generador; compila y los tests de new, import y escribir pasan.
- `--escribir`: tras publicar, `adelantar_editada` reescribe la carpeta con lo publicado, prepara y hace `merge --ff-only`; si falla, queda `con-cambios` sin perder contenido. No se encontro perdida de las ediciones de la propia persona (caso c).
- `.gitattributes`: `*.rs`, `*.toml` y `Cargo.lock` en LF; la fixture `referencia.md` con `-text` conserva su mezcla CRLF/LF (`git ls-files --eol` la da como `mixed` y `-text`).
- Sin `unwrap`, `expect`, `panic!`, `unreachable!` ni `unsafe` en el codigo de produccion de los ficheros del diff.
