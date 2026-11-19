# Informe de revision — TASK-044 (ronda 1)

- Commit revisado: c9a8550 (HEAD de la rama; el codigo es el de cf29ab0, c9a8550 solo anade la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| # | Gravedad | Donde | Hallazgo |
|---|---|---|---|
| 1 | MENOR | crates/tc2/src/verificacion/informe.rs:75 (`leer`), :152 (`seccion_revisor`) | Ni el lector ni el esquema acotan el tamano: sin `maxItems` en `pasos` ni limite de bytes. Medido: un JSON de 96 MB con 2 000 000 de pasos validos se lee y produce una seccion de 36 MB en 5,5 s (debug). Sin panic y lineal; el coste es solo contexto/tokens del revisor IA si F6-3 llama a `leer` con un fichero hostil sin limite propio. |

Detalle del 1. Caso: `pasos` repetido 2 000 000 de veces con `{"nombre":"a","resultado":"verde","segundos":1}`.
Propuesta, no bloqueante: `maxItems` en el esquema y un tope en `leer` (por ejemplo 64 pasos y 64 KiB de texto), o anotar en TASK-049 que el llamador limita la lectura. Es MENOR porque `leer` aun no tiene llamador y TASK-049 ya esta prevista como consumidor.

Observaciones sin gravedad: fechas solo validadas de forma (`9999-99-99T99:99:99Z` se acepta, igual que el esquema); no se comprueba `fin >= inicio`, ni nombres de paso duplicados, ni "tras el primero en rojo no hay mas". Todo ello coherente con "el JSON nunca decide".

## Alcance
- Ficheros revisados: crates/tc2/src/lib.rs, crates/tc2/src/verificacion/informe.rs, crates/tc2/src/verificacion/mod.rs, xtask/src/main.rs, xtask/src/workflows.rs (diff de la peticion), mas el esquema y los fixtures para el contraste.
- Contexto adicional pedido: ninguno fuera de lo permitido salvo `crates/tc2/esquemas/informe-verificacion.schema.json` y los fixtures `crates/tc2/tests/fixtures/informe-verificacion/*.json`, leidos para contrastar lector, escritor y esquema.
- Limite JS-WASM: sin cambios; nada del diff lleva `#[wasm_bindgen]`.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-044-r1-a`, commit c9a8550, `core.autocrlf=true`. El fixture `xtask.json` esta en CRLF (comprobado con `od -c`).
- Build: implicito en los tests (nativo). No se compilo a `wasm32` (el diff no lo toca).
- Formato y lints: `cargo clippy --locked -p tc2 -p xtask --all-targets -- -D warnings`: sin avisos.
- Suite: `cargo test --locked -p xtask`: 25 pasados, 0 fallos (incluye `el_informe_es_el_fixture_de_tc2` con el fixture en CRLF y `el_informe_se_sube_siempre_y_solo_desde_ejecutar`). `cargo test --locked -p tc2 --lib verificacion::`: 2 pasados. Se acepta la verificacion completa en verde de la peticion como linea base; no se corrio la suite entera ni `cargo xtask verificar`.
- Casos construidos (test temporal en el clon, ya borrado): BOM al inicio (error), clave duplicada (error), `tests: null` (aceptado como ausente), texto tras el JSON (error), `segundos` flotante (error), `u64::MAX` (aceptado), `u64::MAX + 1` (error), nombre de 65 caracteres (error), fecha con digito Unicode (error), `commit` en mayusculas (error por el test del PR), JSON con 2 000 000 de pasos (hallazgo 1). No hay camino a panic: ni `unwrap` ni indexado en `leer`/`seccion_revisor` (solo en `#[cfg(test)]`, con `allow`).
- Inyeccion de Markdown en `seccion_revisor`: cada campo interpolado esta acotado antes de llegar al texto. `perfil` y `resultado` son enum o lista cerrada; `commit` son 40 hex; `inicio` y `fin` son digitos con la forma fija; `nombre` cumple `^[a-z0-9][a-z0-9._-]{0,63}$` (sin espacios, `|`, backticks ni saltos); `segundos` y `tests` son `u64`. Unico campo libre: `artefacto`, que lo pasa el llamador (no viene del JSON) y no se valida; conviene que TASK-049 lo construya a partir de `informe-verificacion-<perfil>` y no del contenido del PR.
- Escritor de xtask: JSON a mano sin escapado alguno, correcto por construccion: `nombre` son `&'static str` constantes, `perfil` sale de un `match`, `commit` solo se emite si son 40 hex minusculas, las fechas son numericas. Los dos informes generados son byte a byte los fixtures que lee tc2.
- Fechas UTC: `fecha_utc` (xtask/src/main.rs:430) contrastada con `datetime` de Python en UTC sobre 300 valores pseudoaleatorios hasta el ano 2233 mas bordes (0, 86399, 86400, 29-feb-2000, 1-mar-2100 y similares): 0 diferencias; `u64::MAX` no hace panic ni overflow.
- Errores de xtask: `verificar` (main.rs:258) no tiene `unwrap`/`expect`; el fallo al escribir el informe en una ejecucion verde devuelve `Err`, y en rojo se muestra por `eprintln!` sin enmascarar el error real. `segundos_unix` y `commit_actual` degradan a `0`/`None`.
- Mutacion (una): anulada la validacion `es_nombre_de_paso` en `leer` (informe.rs:106). Resultado: `lo_que_el_contrato_no_admite_es_error` se pone rojo con "«fmt | x» se acepto". Revertida con `git checkout`; el clon queda limpio.
- Medidas: la del hallazgo 1. Sin cambios de tamano de `.wasm` (no aplica).

## Revisado sin hallazgos
- `deny_unknown_fields` y `deserialize_with = "Option::deserialize"` para `commit` obligatorio pero nulable: quitar la clave da error y `null` se acepta (comprobado).
- Version distinta de 1, perfil desconocido, `verde` con paso en rojo y `fallidos > total`: errores con mensaje util y sin panic.
- Tests de `informe.rs`: el caso de mutaciones normaliza CRLF a LF y el fixture se lee bien con CRLF; el test de xtask tambien normaliza. Pasan en este clon con autocrlf.
- `workflows.rs`: la subida `actions/upload-artifact` se rechaza en clase privilegio y sin SHA; el test lo demuestra.
