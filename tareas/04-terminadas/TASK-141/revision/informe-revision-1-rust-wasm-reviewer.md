# Informe de revision — TASK-141 (ronda 1)

- Commit revisado: 65a9b47af269e1b7ec3bd6ee34daace322bf0d0d
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: **aprobada-con-correcciones**. Sin CRITICO ni IMPORTANTE:
la refactorizacion es fiel, los unicos cambios de comportamiento son los
aceptados y la suite sale en verde. Quedan 4 MENOR (dos huecos de test que
dejan sobrevivir mutantes de la parte nueva, una duplicacion residual y un
`match` fragil), que se pueden corregir o aceptar documentandolos.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/alta.rs:116 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/lectura.rs:60 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/importar/mod.rs:247 | implementacion |
| MEN-4 | MENOR | abierto | crates/tc2/src/alta.rs:113 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `demasiados-criterios` en `import` no tiene test (mutante vivo)

La tarea acepta `causa=demasiados-criterios` en `new` **e** `import`, pero solo
`new` lo prueba (`tests/alta.rs`, caso de 13 criterios). Mutante: en
`alta.rs:116` cambiar `Err(("demasiados-criterios", _)) => Err(NoListo::DemasiadosCriterios)`
por `=> Ok(())`. Resultado: `--lib` 403 ok (sobrevive), `--test importar` 9 ok
(sobrevive); solo se pone rojo `--test alta c9`. Es decir, la mitad `import` de
la regla A3-08 para el tope de 12 no esta protegida. Falta una entrada de 13
criterios en `importar/fichero.rs` (unitario) o en `tests/importar.rs`.

### MEN-2 — el brazo `no-utf8` de `import` no tiene test (mutante vivo)

Mutante: en `lectura.rs:60` sustituir el `String::from_utf8(..)?` por
`String::from_utf8_lossy(&bytes).into_owned()`. `plan::tests::fichero_vacio_no_utf8_grande_y_ruta_saneada`
lo mata, pero `--test importar` sigue en 9 ok y ningun unitario de `import`
lo ejerce (solo aparece `no-utf8` en `importar/mod.rs:254`). Ya faltaba antes
de la tarea, pero ahora la lectura es compartida y la traduccion
`Motivo::NoUtf8 -> motivo=no-utf8` de `import` queda sin red. Un fichero con
`0xff` en `c7_limites_y_sin_red` lo cubre.

### MEN-3 — traduccion `Motivo` a informe repetida en `plan` e `import`

`plan/mod.rs:142-160` e `importar/mod.rs:247-260` repiten el mismo `match`
sobre `lectura::Motivo` (`NoSeLee`, `NoUtf8` y `Vacio` identicos salvo el
catalogo; `Grande` difiere) y la misma construccion del `Informe`
`uso motivo=.. ruta=..`; ademas los catalogos `plan::NO_SE_LEE/NO_UTF8/VACIO` e
`importar::NO_SE_LEE/NO_UTF8/VACIO` llevan el mismo texto. La lectura ya es
una sola; lo que queda es duplicacion residual de ~12 lineas (Regla 4D,
Duplicacion, MENOR). Aceptable si se documenta que cada orden mantiene su
catalogo (la tarea lo dice: "cada orden traduce"), o se unifica el mapeo
`Motivo -> (motivo, texto)` en `lectura`.

### MEN-4 — `match` por cadenas con comodin en `Entrada::listo`

`alta.rs:113-118` reinterpreta el `&'static str` que devuelve
`Enunciado::validar` y deja `Err(_) => CriterioVacio`. Hoy el comodin solo
alcanza `criterio-vacio` (los demas casos los frena antes `listo`), pero si
`validar` anade un motivo nuevo, `new`/`import` lo rotularian como
`criterio-vacio` sin que ningun test lo note. Mas robusto: enumerar
`"criterio-vacio"` y dejar `unreachable`-equivalente documentado, o que
`validar` devuelva un enum. Fragil, no un defecto hoy.

## Regla 4D

- Duplicacion: cumple con la salvedad MEN-3. La lectura acotada vive solo en `crate::lectura` (no quedan `take(..)/read_to_end` de `plan`/`importar`); `persona_y_config` sustituye a las dos secuencias de `approve` y no hay gemela (los otros `(e.quien)()` de `plan/mod.rs:376,434,456` no hacen la misma secuencia: sin rechazo de identidad ni `fetch`+config); `MAX_CRITERIOS` se usa por `crate::plan::heuristica::MAX_CRITERIOS` en el mensaje y en `validar`, sin segundo 12; `MAX_BYTES` unico en `lectura` (`importar::fichero::MAX_BYTES` es un `pub use`, `plan::guardar::MAX_BYTES` desaparece).
- Desviacion: cumple. Los unicos cambios de comportamiento son los aceptados (ver Reproduccion); la API publica solo pierde `plan::guardar::MAX_BYTES` (y `lectura` es `mod` privado).
- Diluvio: cumple. Un modulo de 65 lineas y una funcion generica `persona_y_config`; `alta_de_anotacion` es la mitad sin `Box` de `desde_anotacion`, pequena y necesaria para comparar la `Entrada`.
- Disfraz: cumple. `anotacion_ida_y_vuelta` y `esqueleto_de_los_roles_de_la_ronda` hacen asercion sobre valores reales (comprobado con mutantes de los propios autores y los mios); las fixtures ajustadas no pierden fuerza (ver abajo).

## Alcance

- Ficheros revisados: los 16 de la peticion (alta.rs, i18n/catalogo/importar.rs, importar/fichero.rs, importar/mod.rs, lectura.rs, lib.rs, plan/aprobar.rs, plan/guardar.rs, plan/mod.rs, y tests alta/estimacion/importar/progreso/toma), mas `plan/heuristica.rs` (`validar`, `MAX_CRITERIOS`) como referencia de la regla.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]` ni tipos que crucen.
- Observacion fuera de alcance: otras lecturas acotadas del crate (`hallazgos/orden.rs:219`, `seleccion/orden.rs:282`, `inventario.rs:369`, `lib.rs:591`) siguen con su propio `take(..)`; la tarea las deja a TASK-150 (decision 4), asi que no es hallazgo de esta ronda.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-141-r1\repo`, rama de la tarea (HEAD 0705338, que solo anade la peticion sobre 65a9b47), `CARGO_TARGET_DIR` dentro de esa carpeta.
- Formato y lints: `cargo fmt --check` exit 0; `cargo clippy --locked -p tc2 --all-targets -- -D warnings` exit 0.
- Suite: no se repite entera; linea base de la peticion (antes 787; despues 48 binarios, 789 ok, 0 fallos, 13 ignorados). Ejecutados por mi cuenta: `--lib` 403 ok / 2 ignorados; `--test alta` 12 ok; `--test importar` 9 ok; `--test aprobar` 10 ok (390 s); `--test plan` 7 ok. Sin intermitencias.
- Fixtures: el diff de `tests/` solo anade `--objetivo o` / `> o` a entradas que antes no lo tenian (las mismas aserciones, ahora con el enunciado que `plan` exige) y recalcula lineas y longitudes; el unitario `entradas_malas_rechazan_el_lote_entero` pasa de linea 59 a 88 y de longitud 39 a 51 de forma consistente, y anade el caso `objetivo-vacio`. No se debilita ninguna asercion.
- Casos construidos (test temporal en el clon, ya revertido): `NoListo::DemasiadosCriterios.mensaje()` en es = "tiene mas de 12 criterios de aceptacion: partela en varias tareas" (el `{max}` se rellena con `MAX_CRITERIOS`); 13 criterios con objetivo = `Err(DemasiadosCriterios)`, 12 = `Ok`; criterios sin objetivo = `Err(SinObjetivo)` con y sin `exige_criterios`; borrador solo con titulo = `Ok` con `listo(false)` (se sigue admitiendo) y `Err(SinCriterios)` con `listo(true)`.
- Mutacion (cada una sobre el clon limpio, revertida con `git checkout`):

| Mutante | Resultado |
|---|---|
| `lectura::fichero` sin el rechazo del vacio | rojo: `plan::tests::fichero_vacio_no_utf8_grande_y_ruta_saneada` e `importar c7_limites_y_sin_red` |
| `lectura::fichero` con tope +1 (`bytes.len() > tope + 1`) | rojo en `plan::tests` (`grande.md`); en `import` sigue verde, porque `fichero::leer` vuelve a comprobar el tamano (equivalente a nivel de informe de `import`, protegido por `plan`) |
| `lectura::fichero` sin UTF-8 estricto (`from_utf8_lossy`) | rojo en `plan::tests` (`binario.md`); **vivo en import** (MEN-2) |
| `Entrada::listo` sin llamar a `validar` | rojo: `importar::fichero::tests::entradas_malas_rechazan_el_lote_entero` y `alta c9` |
| `listo` con `demasiados-criterios => Ok(())` | rojo solo en `alta c9`; **vivo en `--lib` y `--test importar`** (MEN-1) |
| quitar `sanear` de `lectura.rs` | no lo repeti: el implementador lo da en rojo (`ruta_que_no_se_lee_saneada`); el test comprueba `\n` y `\u{1b}` fuera del diagnostico, lo que se ve leyendo el test |

- Equivalencia de `persona_y_config` y `de_la_orden`: por lectura del diff, `approve --hito` conserva el orden de claves (`hito=` primero, sin `operacion`/`id`), `pendiente=no` en `sinred` y el rechazo de identidad antes de `fetch`; `--test aprobar` en verde.

## Revisado sin hallazgos

- `lectura::fichero`: lee como mucho `tope + 1` bytes, `saturating_add` evita desbordar con `usize` grande, sin `unwrap`/`panic`; ruta y error saneados una vez. Un fichero que solo lleva BOM no cuenta como vacio (`trim` no quita U+FEFF) y cae despues en `sin-entradas`: comportamiento anterior, sin cambio.
- `alta.rs`: `desde_anotacion` documenta por que basta el titulo (`exige_red`) y el test comprueba `nombre`, `anotacion` y `exige_red` de la reconstruida mas la `Entrada`; el mutante `Alta::nueva("otro")` lo pone en rojo (lo dan los autores; la asercion sobre `Entrada::nueva("con | barra")` lo respalda).
- `plan/aprobar.rs`: `esqueleto_de_los_roles_de_la_ronda` usa roles reales (`brainstorm-arquitectura` y `brainstorm-rendimiento`) y exige `false` sin rutas, que es la rama de `juegos_de_rondas`.
- `lib.rs`: `mod lectura` privado; no amplia la API publica.
