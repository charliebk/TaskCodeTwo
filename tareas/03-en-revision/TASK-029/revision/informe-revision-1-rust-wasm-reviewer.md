# Informe de revision — TASK-029 (ronda 1)

- Commit revisado: 80f72ac09fe9e660606b55f88dc82c61307d5b56
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/importar/fichero.rs:182 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/importar/mod.rs:297 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/alta.rs:455 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — Cabecera `---` con un espacio final (o una linea en blanco antes) se ignora sin avisar

`cabecera` solo reconoce la cabecera si la primera linea es exactamente `---`. Con `--- ` (espacio
final, tipico de un editor) o una linea en blanco antes, la cabecera pasa a ser preambulo y se
ignora: se pierden `lote:` y `sprint:` sin error. Reproducido con una sonda sobre
`fichero::leer` (borrada despues):

- `"--- \nlote: x\nsprint: 3\n---\n### F1 Uno..."` -> OK, `nombre=None`, `sprint=None`.
- `"\n---\nlote: x\n..."` -> OK, `nombre=None`, `sprint=None`.
- `"---\nlote: x\nsprint: 3\n---\n..."` -> OK, `nombre=Some("x")`, `sprint=Some(3)`.

Efecto: el lote sale del nombre del fichero y el sprint es 0, y eso se escribe en los `tarea.md`.
Es un caso de borde con consecuencia acotada y visible (`--simular`), por eso MENOR. Arreglo
sugerido: comparar con `trim_end()` y/o rechazar con `cabecera-invalida` un `lote:` o `sprint:`
suelto en el preambulo.

### MEN-2 — Dos ficheros con el mismo nombre y sin `lote:` comparten lote

Sin cabecera, el lote es el slug del nombre del fichero (`importar/mod.rs:297`). Dos ficheros
`plan.md` en carpetas distintas con claves iguales (`T1`, `F3-2`) dan el mismo `lote/clave`: el
segundo import trata sus entradas como ya importadas y no crea nada. Sale en `existentes=` y no se
pierde nada, pero puede sorprender. Documentarlo o pedir `lote:` explicito cuando hay claves que
colisionan.

### MEN-3 — Una `tarea.md` ilegible o con `lote` invalido no cuenta como importada

`alta::tareas` descarta con `filter_map` las tareas que no se leen. Si una tarea importada se
edita a mano y su `lote:` deja de ser valido, un import repetido no la reconoce y la vuelve a
crear. Requiere edicion manual erronea; `doctor` ya avisa de los `tarea.md` ilegibles. MENOR.

## Alcance
- Ficheros revisados: los 13 de la peticion (`alta.rs`, `estado/lectura.rs`, catalogos i18n,
  `importar/fichero.rs`, `importar/mod.rs`, `lib.rs`, `tarea/{cuerpo,mod,modelo}.rs`,
  `tests/alta.rs`, `tests/importar.rs`).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: no aplica. El crate `tc2` es nativo (CLI) y el diff no toca ninguna firma
  `#[wasm_bindgen]`.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-029-r1-rust`, rama
  `feature/task-029-f3-2-new-e-import-en-bloque-con-dependen` (HEAD b96b694, que solo anade la
  peticion sobre el commit revisado 80f72ac). Borrado al terminar.
- Build: `cargo test` y `cargo clippy` compilan sin errores; no se compilo a `wasm32` (no aplica).
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings` limpio. No se ejecuto
  `cargo fmt --check`.
- Suite: `cargo test -p tc2` en verde (120 tests unitarios y todos los de integracion, 0 fallos).
  No hubo `LNK1104`.
- Casos construidos (sonda temporal en `tests/zz_sonda.rs`): cabecera con espacio final y con
  linea en blanco previa (MEN-1); `\r` suelto como fin de linea (`sin-entradas`, error
  explicito); ciclo de 200 entradas (`ciclo`); cadena de 200 sin ciclo (OK, iterativo, sin
  desbordar la pila); entrada de 1 MB (OK); `Dependencias:` con caracteres multibyte en el limite
  de `PALABRA.len()` (sin panic, `linea-suelta`); titulo con U+202E (se acepta; viene de
  `titulo_valido`, no del diff).
- Mutacion: no se hizo de forma sistematica; no hay `cargo mutants`.
- Medidas: no aplican.

## Revisado sin hallazgos
- Parser (`importar/fichero.rs`): los indices de bytes (`l[n..]`, `l[PALABRA.len()..]`) caen en
  limites de caracter porque `#` es ASCII y `get(..)` + `eq_ignore_ascii_case` garantizan el corte.
  El tope de bytes se aplica antes del BOM, `MAX_ENTRADAS` se aplica al cerrar cada entrada, y
  CRLF y BOM se tratan bien. Claves: duplicadas, auto-dependencia y ciclos se rechazan. No hay
  YAML en el parser: la cabecera se lee a mano, sin recursion ni expansion.
- `Importar::aplicar`: atomico, porque devuelve todos los cambios de una vez o `Fallo` antes de
  escribir nada. Un reintento recalcula `mapa`, `ids` y `plan` sobre el estado fresco, y el plan
  solo se lee si salio bien. Dos clones a la vez: el segundo en publicar ve las claves ya
  existentes y las cuenta en `existentes`. Idempotencia por `lote/clave` correcta; sin entradas
  nuevas devuelve cambios vacios (`SinCambios`).
- `alta::altas` compartido con `new`: la rama se comprueba por entrada, las parecidas se
  deduplican y los IDs son consecutivos. `reservar` usa `checked_add`.
- `Vista::vacia`: `commit = None` y estado vacio; `simular` solo lo usa con `sin-rama-de-estado`.
- `#[allow(clippy::large_enum_variant)]` en `Documento`: lleva comentario que lo justifica y
  la lectura es puntual; aceptable.
- `referencia_a_uso`: solo remapea `Rechazada` con `motivo=referencia-inexistente`, en `import`,
  `new` y `--simular`.
- Produccion sin `unwrap`, `panic` ni `unsafe` en el diff; los `unwrap_or*` son con valor por
  defecto.
- `Cuerpo::con_enunciado`: parte la plantilla antes de sustituir y escapa lineas que abririan
  seccion o cerca; `&linea[..sangria]` es seguro (solo espacios ASCII).
- Rendimiento: la busqueda lineal de `rutas` por dependencia es cuadratica pero acotada
  (200 entradas), sin efecto medible.
