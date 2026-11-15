# Informe de revision — TASK-020 (ronda 1)

- Commit revisado: f2fdb154fbe7f9cef83906ff34b4c79fd56b290d
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: cambios-solicitados

(El revisor propuso `aprobada-con-correcciones`; con un IMPORTANTE abierto se registra `cambios-solicitados`.)

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/tarea/yaml.rs:opciones | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/tarea/yaml.rs:cadena | implementacion |
| MEN-3 | MENOR | corregido | crates/tc2/src/tarea/yaml.rs:partir | implementacion |
| MEN-4 | MENOR | corregido | crates/tc2/src/tarea/modelo.rs:escribir | plan |
| MEN-5 | MENOR | aceptado | crates/tc2/src/tarea/yaml.rs:partir | implementacion |
| MEN-6 | MENOR | corregido | crates/tc2/src/tarea/cuerpo.rs:es_cerca | implementacion |
| MEN-7 | MENOR | corregido | crates/tc2/src/tarea/modelo.rs:escape_ambiguo | implementacion |
| MEN-8 | MENOR | corregido | crates/tc2/src/tarea/modelo.rs:Tarea | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### Reproduccion de cada hallazgo

- **IMP-1.** El lector acepta booleanos y enteros con grafía YAML 1.1 y escalares entre comillas donde el esquema pide bool o entero (`Options::default()` deja `strict_booleans = false`). Con `tarea::leer` sobre formato 2: `plan_aprobado: yes`/`On`/`True` → true, `no` → false, `"false"` → false; `sprint: '7'` → 7, `0x10` → 16, `0o10` → 8, `1_0` → 10, `+5` → 5; `formato: "2"` aceptado. `serde_yaml` da `yes` y `1_0` como cadenas: CLI y parser estándar leen distinto `plan_aprobado`, la bandera de la puerta de aprobación (F1-8; F1-9 punto 5). Corregido en 53c942f: `strict_booleans`, `underscored_numbers_as_strings`, `leading_zero_numbers_as_strings` y una comprobación propia de la grafía canónica del texto crudo (`canonicos`); tests en `esquema_estricto_del_formato_2` y `formato_1_ambiguo_o_duplicado_falla`.
- **MEN-2.** U+FFFE y U+FFFF se aceptan y `cadena()` los emitía sin escapar; ni `serde_yaml` ni serde-saphyr los leen (no son `c-printable`). Corregido en 53c942f (escapados como `\uFFFE`/`\uFFFF`; caso en `ida_y_vuelta_con_parser_estandar`).
- **MEN-3.** Duplicadas con `entradas.iter().any`: coste cuadrático (40.000 claves, 2,28 s en release). Corregido en 53c942f con `HashSet`.
- **MEN-4.** La reescritura descarta los comentarios del frontmatter y un comentario como primera línea tras `---` se rechazaba. Corregido en 53c942f: el comentario inicial se admite y la doc de `escribir` dice que los comentarios no se conservan.
- **MEN-5.** Con CRLF el fichero reescrito mezcla finales (frontmatter LF, cuerpo CRLF) y el BOM se pierde. Aceptado: el modelo no recuerda el final de línea, la doc de `escribir` lo dice y ninguna tarea real tiene CRLF ni BOM (0 de 595). Si una tarea posterior reescribe tareas ajenas, que normalice el fichero entero.
- **MEN-6.** Cercas de código más laxas que CommonMark (`"## A\n````\n```\n## B\n````\n## C\n"` daba `["A","B"]`). Corregido en 53c942f (`cerca`/`abre`/`cierra`; test `cercas_como_commonmark`).
- **MEN-7.** Falsos positivos de H11: miraba comentarios, cadenas planas y de comilla simple (`titulo: C:\ruta`, `titulo: 'F2-5\ Modelo'`). Corregido en 53c942f: solo dentro de comillas dobles; casos en `formato_1_ambiguo_o_duplicado_falla`.
- **MEN-8.** `estimacion` y `desconocidas` públicos: `escribir` copiaba sin validar (`estimacion = Some("1h")` daba YAML roto). Corregido en 53c942f: privados, con accesores `estimacion()` y `claves_desconocidas()`.

## Lecturas fuera del contexto permitido

- `docs/adr/F1-9-*.md` (puntos 5 y 6, preguntas abiertas) y `docs/adr/F1-14-*.md` (fila serde-saphyr), pedidos en la tarea.
- `serde-saphyr` 2.1.1 `src/de/options.rs` en el registro de cargo, para los defaults de `Options`.
- Corpus de ToolsaWeb, OpenDockTrading y TaskCode (solo lectura).

## Alcance y reproduccion del revisor

- Límite JS-WASM: no aplica (CLI nativo). Sin `unsafe` ni `f64`.
- Divergencias de alcance (D3; `zonas`, `hito`, `roles` ampliado): aceptables. Un campo desconocido dentro de `roles` se rechaza (`deny_unknown_fields`), así que el `roles` ampliado de F1-15 romperá la lectura hasta que su tarea lo modele; decisión deliberada y documentada.
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-020-r1\repo`, borrado al terminar. `fmt` y `clippy -D warnings` limpios; `cargo test -p tc2 --locked`: 63 pasan, 1 ignorado; ignorado con `TC2_CORPUS`: 404 y 97 tareas, ok.
- Mutación: 6 mutantes sobre producción, todos detectados. Los tests de ida y vuelta no son circulares (`serde_yaml`).
- Sin `unwrap`, `expect` ni indexado sobre entrada externa en producción; dependencias según plan y F1-14; absorción de `alta.rs` byte a byte (test `DORADA`); corpus 0.16 de 94 + 404 + 97 tareas.
