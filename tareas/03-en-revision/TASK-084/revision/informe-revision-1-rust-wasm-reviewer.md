# Informe de revisión — TASK-084 (ronda 1)

- Commit revisado: 4cc0c36
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R1-01 | IMPORTANTE | abierto | crates/tc2/src/progreso/mod.rs:282 | implementacion |
| R1-02 | MENOR | abierto | crates/tc2/src/tablero.rs:279 | implementacion |
| R1-03 | MENOR | abierto | crates/tc2/src/main.rs:47 | implementacion |
| R1-04 | MENOR | abierto | crates/tc2/src/progreso/fichero.rs:1102 | implementacion |
| R1-05 | MENOR | abierto | crates/tc2/src/progreso/linea.rs:80 | implementacion |

**R1-01 (IMPORTANTE).** `status --seguir` no termina al cerrarse stdout si la linea no cambia: `seguir` solo detecta el cierre al escribir y solo escribe si el texto cambia (`progreso/mod.rs:292-296`). Sin tarea en curso la linea es fija y el proceso queda huerfano consultando cada 2 s, contra el comentario de `main.rs:38-40`, el docstring de `seguir` y C9. Reproduccion: fuera de un repo, `timeout 10 tc2 status --seguir | head -1`: `head` sale y tc2 sigue vivo hasta el `timeout` (`PIPESTATUS[0]`=124). El test `Cerrado` pasa porque su primera escritura siempre falla. Arreglo: detectar el cierre de otra forma o escribir un keepalive; o documentar que solo termina con Ctrl+C y quitar la promesa.

**R1-02.** `sanear` deja pasar U+061C (control bidi), U+00AD, U+2060 (y U+2061-2064, U+180E). Anadirlos al filtro de `tablero.rs:279`.

**R1-03.** `tc2 estado --stdin` con stdout cerrado escribe `tc2: no se pudo escribir la salida` en stderr y sale con 1 (`main.rs:54-62`; reproducido 3/3). Esa orden deberia ignorar el error de escritura.

**R1-04.** Un fallo al escribir `.anterior.json` (`fichero.rs:1102-1105`, con `?`) impide escribir el progreso principal. Intentar el anterior y escribir el principal igualmente.

**R1-05.** Pasados 480 min la tabla se queda en el ultimo punto sin avisar de que esta saturada (`lo_que_falta`).

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-084-r1-rust` (borrado). progreso 8, frescura 12, identidad 7, desinstalar 19, toma 13, estimacion 11, unitarios de progreso y huella 25: todos verdes.

Casos con el binario real sin panic y con rc 0: stdin vacio, BOM, no UTF-8, `null`, 5 MB, 100 000 `[` anidados; ficheros de progreso con version 2, mas de 64 KB, `id` null, vacio, BOM, directorio con nombre de fichero, fechas futuras, `u64::MAX`, `1e308`; `.git` fichero con `gitdir:` vacio, inexistente, `\r`, `commondir` roto, vacio o no UTF-8. Sin `unwrap`/`expect`/`panic!` fuera de tests.

Mutaciones en rojo: `escribir_local` sin reintento; sin tope de 45 min en `transcurrido`; aceptar `version >= VERSION`.

## Revisado sin hallazgos

`escribir_local` (crea carpeta, reintento acotado ~300 ms, borra el temporal; temporal con pid); migracion de identidad y frescura; lectura con tope; `calcular`, `restante` y `linea` sin NaN que escapen y tabla alineada; `de_una_tarea`/`con_progreso` no cambian la transicion; `start` no duplica el refresco; `init` sin tocar `settings.json`. Limite JS-WASM: no aplica.
