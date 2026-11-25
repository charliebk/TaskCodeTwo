# Informe de revision — TASK-150 (ronda 2)

- Commit revisado: 81155ba1361f4d574148d86fbdebc38567c8d55d
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/hallazgos/orden.rs:223 (`leer_entrada`) y crates/tc2/src/seleccion/orden.rs (`--rutas`, `--tarea`, ficheros de `paso_seleccion`) | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — corregido (verificado, no dado por bueno)

- La correccion anade `lectura::texto_con_tope_lossy` (mismo `con_tope`, mismo tope, `String::from_utf8_lossy`). La usan `hallazgos::orden::leer_entrada` (respuesta, diff, contexto, JSON, rutas), `leer_validados` y los tres puntos de `seleccion::orden` (`--rutas`, `--tarea` y `--rutas` de `paso_seleccion`). El lector de informe (`texto_con_tope` en `informe=`) sigue estricto.
- Caso de la ronda 1 repetido con el binario de la rama: `hallazgos validar --respuesta respuesta-mixta.md --diff <diff.txt + "+// caf\xE9"> --revisora x` da `hallazgos ok revisora=x aceptados=3 rechazados=2 motivos=H4:sin-evidencia,H5:linea-fuera-del-diff`, salida 0 (en la ronda 1 daba `error`, salida 1).
- Informe no UTF-8 sigue dando `informe=invalido`: con `{"a":"caf\xE9"}` y con `\xFF` como `--informe`, salida 0. Se conserva la decision 3.
- Mutacion: volver `leer_entrada` a `texto_con_tope` estricto pone en rojo `diff_y_respuesta_no_utf8_se_leen` (9 ok, 1 fallido); revertido con `git checkout`.
- Diff de la correccion (3c1d524, 9ce49a2): sin fallo nuevo. El tope se aplica igual (`con_tope` conserva `take(tope+1)`). El stdin de `seleccion` (`-`) sigue estricto, como antes de TASK-150. El doc de `escritura::atomica` (MEN-2) es solo documentacion.

## Alcance

- Ficheros revisados: crates/tc2/src/lectura.rs, crates/tc2/src/hallazgos/orden.rs, crates/tc2/src/seleccion/orden.rs, crates/tc2/src/escritura.rs (solo doc), crates/tc2/tests/hallazgos.rs.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios.

## Reproduccion

- Clon `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-150-r2`, HEAD 2d9b55b (incluye 81155ba), target dentro del clon. Se espero a que terminaran dos `cargo.exe` ajenos antes de lanzar cargo.
- Build, formato y lints no repetidos (ronda acotada).
- `cargo test -p tc2 --lib --locked`: 432 ok, 0 fallidos, 2 ignorados. `--test hallazgos --test seleccion`: hallazgos 10 ok, seleccion 2 ok. Sin suite completa.
- Casos construidos: diff con `+// caf\xE9`; informes no UTF-8 (`{"a":"caf\xE9"}` y `\xFF`).
- Mutacion: 1, en rojo, revertida.
- Limpieza: sin procesos; clon borrado.

## Revisado sin hallazgos

- `texto_con_tope_lossy`: frontera del tope, `None` por exceso, bytes invalidos a `U+FFFD`.
- `leer_validados` en lossy: sigue exigiendo `version == VERSION`.
- Doc de `atomica`: coincide con lo descrito en MEN-2 de la ronda 1.

## Regla 4D

- Duplicacion: cumple. `texto_con_tope_lossy` reutiliza `con_tope`.
- Desviacion: cumple. El estricto solo se aplica a informes.
- Diluvio: cumple.
- Disfraz: cumple. El test nuevo se pone en rojo al revertir la correccion.

## Lecturas fuera del contexto permitido

- `crates/tc2/tests/fixtures/hallazgos/diff.txt` y `respuesta-mixta.md`: para construir los casos.
- `crates/tc2/src/hallazgos/orden.rs` (280-300): para localizar la lectura de `--informe`.

## Nota (no es hallazgo)

`seleccion --rutas -` (stdin) sigue con `String::from_utf8` estricto y da error ante un byte no UTF-8; era asi antes de TASK-150. Se anota por si se quiere unificar con `--rutas <fichero>`, que ahora es lossy.
