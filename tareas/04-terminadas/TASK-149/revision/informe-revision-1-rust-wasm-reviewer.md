# Informe de revision — TASK-149 (ronda 1)

- Commit revisado: 02d8a4d06b04453d55e45765ce24186700d13434
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | aceptado | crates/tc2/src/estado/dueno.rs (`AnotacionConDueno::leer`, `escribir`) | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

**Sin hallazgos CRITICOS ni IMPORTANTES.**

**MEN-1 (MENOR).** `AnotacionConDueno` sirve a `approve-hito`, donde `id` es el numero del hito. `escribir` lo admite, pero `leer` lo rechazaria por `git::es_id`. Hoy no hay lector de esa anotacion (exige red y no queda pendiente), asi que no hay efecto observable. Si alguien anade un `desde_anotacion` para `approve-hito` reutilizando `leer`, fallaria en silencio. Basta una linea en el doc de `id` que lo diga. No se ha construido un caso que falle porque el lector no existe. (Aceptado por quien orquesta: sin efecto observable; el veredicto es `aprobada` y una linea de doc en codigo tras el commit revisado obligaria a otra ronda; queda en el `## Resultado` para la segunda auditoria.)

**Equivalencia de contrato.** Comprobada por lectura del diff mas ejecucion; no se compilo develop para comparar salidas.
- Los tests de integracion de `crates/tc2/tests/` no estan tocados en la rama (`git diff origin/develop..HEAD -- crates/tc2/tests` vacio) y pasan todos. Fijan el orden de claves, los motivos y los codigos de salida.
- `como_orden` sustituye a `de_start`, `de_review`, `de_la_orden` de ciclo y `de_la_orden` de plan con el mismo comportamiento (comando, quita `operacion`, `id` y la clave, inserta la clave primero y pone `pendiente=no` solo en un `sinred` sin `pendiente`).
- `Fallo::informe` equivale a los `match` retirados en `review`, `start`, `descartar`, `roadmap`, `siguiente`, `tablero`, `seleccion`, `estimacion` e `importar` (`simular=si` en la posicion 0 en importar; `id=` primero solo si Rechazada en estimacion; `asignado=` solo si Rechazada en review).
- Lectores de anotaciones: `review`, `descartar`, `pausa`, `liberar` y `reasignar` aceptan y rechazan lo mismo que antes. `start` es la unica divergencia, la declarada en el Resultado (mas estricta).
- Escritura: `escribir` produce `orden|id|cuenta|login[|resto]`, el mismo formato que los `format!` retirados.

## Regla 4D

- Duplicacion: cumple. Consolida 4 copias del informe de la orden, 9 de `Fallo` a `Informe`, 2 de `de_base_o_suelo`, 9 de `escribir`/`con_dueno`, 9 del preambulo con dueno y 4 de `falta-valor`.
- Desviacion: cumple. Las firmas publicas del CLI y el contrato no cambian; la unica desviacion de comportamiento es la de `start`, declarada.
- Diluvio: cumple. Funciones y un struct pequeno, sin traits nuevos.
- Disfraz: cumple. Los tests nuevos hacen aserciones reales (ver mutacion).

## Alcance

- Ficheros revisados: los 25 de la peticion, con foco en `estado/dueno.rs`, `estado/transaccion.rs`, `contrato.rs`, `config/mod.rs` y los lectores `desde_anotacion` de `ciclo/operacion.rs`, `ciclo/liberar.rs`, `revision/operacion.rs` y `toma/operacion.rs`.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios; el diff no toca `#[wasm_bindgen]`.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-149-r1`, HEAD eeb339e (commit revisado mas peticion, sin cambios de codigo), `CARGO_TARGET_DIR` propio. Sin otros cargo en marcha al empezar.
- Build nativo con `cargo test`; sin `wasm32` ni `--release`. fmt y clippy no ejecutados.
- `cargo test -p tc2 --lib`: 427 ok, 0 fallidos, 2 ignorados.
- Integracion en verde: `alta` 12, `aprobar` 10, `ciclo` 15, `estimacion` 11, `hallazgos` 9, `importar` 9, `liberar` 12, `plan` 7, `revision` 5, `roadmap` 13, `toma` 13. Sin suite completa del workspace.
- Mutacion (aplicada en el clon y revertida con `git checkout`):
  1. `como_orden` sin quitar el `id=`: `informe_de_la_transaccion_como_de_la_orden` rojo.
  2. `AnotacionConDueno::leer` sin `es_id`: `anotacion_con_dueno_ida_y_vuelta` rojo.
  3. `tarea_propia` sin limpiar `dueno`: `tarea_propia_ajena_heredada_e_inexistente` rojo.
  4. `Fallo::Error` como `Rechazada`: `fallo_como_informe_de_la_orden` rojo.
  5. `de_base_o_suelo` sin tratar `sin-config` como suelo: `config_de_la_base_o_suelo_a_nombre_de_la_orden` rojo.
- Limpieza: clon borrado, sin procesos cargo ni rustc.

## Revisado sin hallazgos

- `valor_de` copia el `String` igual que el `cloned()` anterior; en `alta.rs`, `let v = &valor_de(...)?` extiende la vida del temporal sin problema.
- `Entorno::escribir` (toma) es el mismo cuerpo que el `escribir` retirado de ciclo y plan.
- `config::de_base_o_suelo` conserva la politica M-4.
- El `*dueno = None` de `tarea_propia` cubre todos los llamadores; en `PedirCambios` sigue limpiandose `ronda` aparte.

## Lecturas fuera del contexto permitido

- `git diff origin/develop..HEAD` de `crates/tc2/tests` (para confirmar que no esta tocado).
- `tarea.md`, `## Resultado` / `### Divergencias`, para contrastar la divergencia declarada de `start`.
