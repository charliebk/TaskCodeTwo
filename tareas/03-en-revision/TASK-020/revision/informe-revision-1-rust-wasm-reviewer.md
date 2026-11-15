# Informe de revision — TASK-020 (ronda 1)

- Commit revisado: f2fdb154fbe7f9cef83906ff34b4c79fd56b290d
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| # | Severidad | Fichero:linea | Hallazgo y evidencia reproducida | Origen | Estado |
|---|---|---|---|---|---|
| 1 | IMPORTANTE | `crates/tc2/src/tarea/yaml.rs` `opciones()` | El lector acepta booleanos y enteros con grafía YAML 1.1 y escalares entre comillas donde el esquema pide bool o entero (`Options::default()` deja `strict_booleans = false`). Reproducido con `tarea::leer` sobre formato 2: `plan_aprobado: yes`/`On`/`True` → true, `no` → false, `"false"` → false; `sprint: '7'` → 7, `0x10` → 16, `0o10` → 8, `1_0` → 10, `+5` → 5; `formato: "2"` aceptado. `serde_yaml` da `yes` y `1_0` como cadenas: CLI y parser estándar leen distinto `plan_aprobado`, la bandera de la puerta de aprobación (F1-8; F1-9 punto 5 «tipo incorrecto da error»). Sin test para este caso. | implementacion | corregido (53c942f: strict_booleans + grafia canonica propia; tests en esquema_estricto_del_formato_2 y formato_1_ambiguo_o_duplicado_falla) |
| 2 | MENOR | `modelo.rs` `titulo_valido`/`texto_valido` + `yaml.rs` `cadena()` | U+FFFE y U+FFFF se aceptan y `cadena()` los emite sin escapar; ni `serde_yaml` ni serde-saphyr los leen (no son `c-printable`). `alta` podría crear un `tarea.md` ilegible. El resto de casos hostiles da ida y vuelta correcta con ambos parsers. | implementacion | corregido (53c942f: escapados en cadena(); caso en ida_y_vuelta_con_parser_estandar) |
| 3 | MENOR | `yaml.rs` `partir()` (duplicada con `entradas.iter().any`) | Coste cuadrático en el número de claves de primer nivel: release 5.000 claves 45 ms, 10.000 180 ms, 20.000 519 ms, 40.000 2,28 s. Alias, profundidad y valores grandes bien contenidos. | implementacion | corregido (53c942f: HashSet en partir) |
| 4 | MENOR | `modelo.rs` `Tarea::escribir` | La reescritura descarta los comentarios del frontmatter, y un comentario como primera línea tras `---` se rechaza aunque es YAML válido. La doc no avisa de la pérdida. | plan | corregido (53c942f: comentario inicial admitido; la doc de escribir dice que los comentarios no se conservan) |
| 5 | MENOR | `yaml.rs` `partir()` + `modelo.rs` `escribir` | Un `tarea.md` con CRLF se reescribe con finales mezclados (frontmatter LF, cuerpo CRLF) y el BOM se pierde. Ninguna tarea del corpus tiene CRLF ni BOM (0 de 94). | implementacion | aceptado: el modelo no recuerda el final de linea; la doc de escribir lo dice y ninguna tarea real tiene CRLF ni BOM (0 de 595). Si F2/F3 escribe tareas de otros, que normalice todo el fichero |
| 6 | MENOR | `cuerpo.rs` `es_cerca` | Cercas más laxas que CommonMark: `~~~` cierra un bloque abierto con ```` ``` ````, ```` ```x``` ```` abre bloque, 4 acentos se cierran con 3, sangría de 4+ cuenta. `"## A\n````\n```\n## B\n````\n## C\n"` da `["A","B"]`. Bytes siempre conservados; 0 diferencias frente a CommonMark en 727 tareas reales. | implementacion | corregido (53c942f: cercas CommonMark; test cercas_como_commonmark) |
| 7 | MENOR | `modelo.rs` `escape_ambiguo` | Falsos positivos del chequeo de `\` del formato 1: mira también comentarios, cadenas planas y de comilla simple. `titulo: C:\ruta`, `titulo: 'F2-5\ Modelo'` y `dependencias: [TASK-010] # C:\x` dan H11. Conservador y seguro; 0 de 627 tareas reales lo disparan. | implementacion | corregido (53c942f: solo dentro de comillas dobles; casos en formato_1_ambiguo_o_duplicado_falla) |
| 8 | MENOR | `modelo.rs` campos `pub` `estimacion` y `desconocidas` | `escribir` copia sin validar: `estimacion = Some("1h")` produce YAML roto; una desconocida `("id", ...)` produce clave duplicada. La doc lo dice, pero los campos son públicos. | implementacion | corregido (53c942f: campos privados con accesores) |

## Alcance

- Ficheros revisados: `crates/tc2/Cargo.toml`, `src/alta.rs`, `src/lib.rs`, `src/tarea/{cuerpo,mod,modelo,yaml}.rs`, `tests/tarea.rs`; `Cargo.lock` solo para versiones.
- Lecturas fuera del contexto permitido: ADR F1-9 (puntos 5 y 6, preguntas abiertas) y F1-14 (fila serde-saphyr), pedidos en la tarea; `serde-saphyr` 2.1.1 `src/de/options.rs` para los defaults; corpus de ToolsaWeb, OpenDockTrading y TaskCode (solo lectura).
- Límite JS-WASM: no aplica (CLI nativo, sin `#[wasm_bindgen]`).
- Divergencias de alcance (D3; `zonas`, `hito`, `roles` ampliado): aceptables. Matiz: un campo desconocido dentro de `roles` se rechaza (`deny_unknown_fields`), así que el `roles` ampliado de F1-15 romperá la lectura hasta que su tarea lo modele; decisión deliberada y documentada.

## Reproduccion

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-020-r1\repo` (checkout del commit revisado), borrado al terminar.
- `cargo fmt --check` y `cargo clippy -p tc2 --all-targets --locked -- -D warnings` limpios. `cargo test -p tc2 --locked`: 63 pasan, 1 ignorado. Ignorado con `TC2_CORPUS`: 404 y 97 tareas, ok.
- Casos hostiles de títulos, frontmatter (anclas, alias, fusión, duplicadas anidadas, etiquetas, multidocumento, BOM, CRLF, anidación profunda) y escalares con tipo incorrecto: todo rechazado con error claro o aceptado con normalización idempotente, salvo hallazgos 1, 4 y 5.
- Mutación: 6 mutantes sobre producción, todos detectados (duplicadas `LastWins`, anclas permitidas, sin `escape_ambiguo`, desconocidas no escritas, claves con mayúsculas, sin escape de `\`). Los tests de ida y vuelta no son circulares (`serde_yaml`).

## Revisado sin hallazgos

- Sin `unwrap`, `expect` ni indexado sobre entrada externa en producción.
- Dependencias según plan y F1-14 (`default-features = false`, `serde_yaml` solo en dev), YAML tras `tarea::yaml`, lock versionado y `--locked`.
- Absorción de `alta.rs` byte a byte (test `DORADA`).
- Duplicadas del separador, claves no admitidas, desconocidas en orden, cuerpo byte a byte (también con CRLF), corpus 0.16 de 94 + 404 + 97 tareas.
