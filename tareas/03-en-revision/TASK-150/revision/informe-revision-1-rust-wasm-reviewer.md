# Informe de revision — TASK-150 (ronda 1)

- Commit revisado: 8347d882515db613e63d96e66e2e2f147fad004f
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/hallazgos/orden.rs:223 (`leer_entrada`) y crates/tc2/src/seleccion/orden.rs (`--rutas`, `--tarea`, ficheros de `paso_seleccion`) | implementacion |
| MEN-1 | MENOR | corregido | crates/tc2/tests/ (sin test de contrato) | implementacion |
| MEN-2 | MENOR | corregido | crates/tc2/src/escritura.rs:343-368 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

(El revisor propuso `aprobada-con-correcciones`; con un IMPORTANTE abierto el veredicto es `cambios-solicitados`, segun el flujo.)

### IMP-1 — El UTF-8 estricto se extendio a entradas que no eran informes: `hallazgos validar` falla con un diff que lleva bytes no UTF-8

La decision de la persona fue solo `informe=invalido` para informes no UTF-8. Pero `lectura::texto_con_tope` es estricto para todos sus llamadores. Antes de TASK-150, las dos `leer_con_tope` locales usaban `String::from_utf8_lossy`. Ahora `hallazgos::orden::leer_entrada` lee con el estricto la respuesta de la revisora, el diff (`TOPE_DIFF`), el contexto, el JSON y las rutas; `seleccion::orden` hace lo mismo con `--rutas`, `--tarea` y los demas ficheros de entrada. Un `git diff` de un repo con un fichero en Latin-1 o una linea binaria es un caso real.

Reproduccion con el binario de cada rama y el fixture `tests/fixtures/hallazgos/respuesta-mixta.md`, con `+// caf\xE9` anadido al final de `diff.txt`:
- `develop`: `hallazgos ok ... aceptados=3 rechazados=2 ...`, salida 0.
- Rama: `hallazgos error` con `no se puede leer ni escribir ...diff-latin1.txt: invalid utf-8 sequence of 1 bytes from index 909`, salida 1.

El contrato de salida de `hallazgos validar` cambia de `ok` a `error` para una entrada que no es informe. Correccion propuesta: estricto solo para los lectores de informe, o una variante lossy para diff, respuesta, contexto y rutas que conserve el comportamiento anterior; con un test con un diff con `0xE9`.

Aviso: con `{\xFF}` como informe, `develop` ya daba `informe=invalido` (el JSON lossy no parsea); la rama da lo mismo. Lo que cambia de contrato para informes es `sin-informe` → `invalido` para uno no UTF-8 y no JSON.

### MEN-1 — Ningun test de integracion fija que el diff/respuesta no UTF-8 siguen siendo validos

Los tests nuevos cubren `lectura` y `informe_revisor`, pero no `hallazgos validar` ni `seleccion` con una entrada no informe y no UTF-8; por eso IMP-1 pasa la suite.

### MEN-2 — `escritura::atomica` cambia ligeramente la semantica para los ficheros locales

- Antes `huella::escribir_local` reemplazaba un enlace por el fichero; ahora escribe el destino (decision documentada). Un enlace roto da error en `canonicalize` en vez de crear el fichero (borde).
- Se reintenta el `rename` hasta 300 ms ante cualquier error, tambien un destino que es una carpeta; el doc-comment lo cubre a medias.
- Los temporales pasan a `.tc2-tmp-<pid>`; un huerfano de una version anterior ya no se reconoce (sin consecuencias).

## Alcance

- Ficheros revisados: los 32 del diff, con atencion a escritura.rs, lectura.rs, proceso.rs, fecha.rs, verificacion/ejecutar.rs, actualizar_cli.rs, seleccion/orden.rs, hallazgos/orden.rs, desinstalar/mod.rs, confianza/mod.rs, frescura.rs, identidad.rs y huella.rs.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios.

## Reproduccion

- Clon `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-150-r1`, HEAD 52c5097 (incluye 8347d88), target propio, sin solape.
- `cargo fmt --check` limpio; `cargo clippy -p tc2 --all-targets --locked -- -D warnings` sin avisos.
- `cargo test -p tc2 --lib --locked`: 432 ok, 0 fallidos, 2 ignorados. Integracion en verde: `verificar` 16 (1 ignorado), `actualizar_cli` 12 (tras `cargo build -p tc2 --examples`), `seleccion` 2, `hallazgos` 9, `progreso` 12, `identidades` 12, `toma` 13, `desinstalar` 19, `identidad` 7, `frescura` 12, `revisoras` 6. Sin suite completa.
- Casos construidos: diff con `0xE9` en `hallazgos validar` en develop y en la rama (IMP-1); informe `{\xFF}` en las dos ramas.
- Mutacion (6, todos en rojo, revertidos con `touch`): sin reintento de `rename`; sin borrar el temporal; frontera `<=` por `<`; `matar_arbol` anulado; `from_utf8_lossy` en `texto_con_tope`; sin el brazo `InvalidData` en `informe_revisor`.
- Limpieza: sin procesos en marcha; clon borrado.

## Revisado sin hallazgos

- Escritura atomica: borrado del temporal en los tres fallos, reintento solo del `rename`, enlaces y permisos (test `cfg(unix)` no ejecutado en Windows).
- Tope de lectura: `take(tope.saturating_add(1))`, frontera correcta; `frescura` e `identidad` sin cambio; stdin de `seleccion` con el mismo mensaje.
- Espera con plazo: mata el arbol sin huerfanos; `verificacion` conserva `Fin::Codigo`/`Fin::Timeout`; divergencia de `try_wait` declarada y sin cambio de contrato.
- Fechas: una sola vez; `xtask` con su motivo.

## Regla 4D

- Duplicacion: cumple.
- Desviacion: IMP-1 (UTF-8 estricto mas alla de lo decidido).
- Diluvio: cumple.
- Disfraz: cumple; falta el test de contrato de MEN-1.

## Lecturas fuera del contexto permitido

- `crates/tc2/tests/fixtures/hallazgos/diff.txt`, `respuesta-mixta.md` y `crates/tc2/tests/hallazgos.rs`: para construir el caso de IMP-1.
- `crates/tc2/src/verificacion/ejecutar.rs` y `proceso.rs` via `git diff`: la peticion venia truncada.
