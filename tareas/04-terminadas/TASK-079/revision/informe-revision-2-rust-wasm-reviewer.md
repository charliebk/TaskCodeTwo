# Informe de revisión — TASK-079 (ronda 2)

- Commit revisado: bf653ca
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | corregido | crates/tc2/src/estado/frescura.rs:89 | ronda 1 |
| H2 | MENOR | corregido | crates/tc2/src/estado/frescura.rs:159 | ronda 1 |
| H3 | MENOR | corregido | crates/tc2/tests/frescura.rs:190 | ronda 1 |
| H4 | MENOR | corregido | crates/tc2/src/estado/lectura.rs:188 | ronda 1 |
| H5 | MENOR | aceptado | crates/tc2/tests/tablero.rs:527 | ronda 1 |

Sin hallazgos nuevos.

## Verificacion

- H1: `traer_estado` sin `-q`; `clasificar_traida` separa `Reescrita` y `Bloqueada`; con `LC_ALL=C` (`puerto.rs:203`) el texto de git no depende del idioma. Lectura: rama reescrita `remoto=rechazado`, ref bloqueada `remoto=error-local`, codigo 0. Escritura (transaccion y `montar`): `error` (1) con `motivo=rama-reescrita` o `motivo=error-local`, sin pendiente. Correcto: un pendiente por rama reescrita nunca se publicaria y reintentar no lo arregla; la ref bloqueada es un fallo del clon como `FalloRemoto::Local`. Sin force-push en el codigo; el cerrojo se libera por `Drop` (concurrencia 5/5).
- H2: `leer_en(ahora)` descarta horas futuras. H3: C2 y C3 borran la marca y comprueban que se recrea. H4: `con_reintento` solo con ref bloqueada (200 y 500 ms). M1-M4 cubiertos (C9, C7, C4) y adendas coherentes. Nota de H5 honesta.
- Commit 68f054e: incluye el refactor de `montaje.rs` y `tablero.rs` bajo un mensaje de tests; el codigo es correcto (solo higiene del mensaje).

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-079-r2-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios. `--lib` 204 ok; frescura 12, tablero 5, alta 12, ciclo 15, estado_concurrencia 5, estado_bordes 10, estado_casos 6, siguiente 6, roadmap 13, importar 9.

| Mutacion | Test en rojo |
|---|---|
| quitar filtro `s <= ahora` | `marca_del_futuro_es_nunca` |
| quitar reconocimiento de `[rejected]` | `clasifica_rechazo_y_ref_bloqueada` |
| volver a `-q` | `c10_rama_reescrita_en_origin_no_es_sin_red` |
| transaccion trata `Reescrita` como `sin_red` | `c10` |
| quitar `marca.registrar` | `c2`, `c3` |
| `con_reintento` sin bucle | `reintenta_solo_la_ref_bloqueada` |
| `leer_fresco` sin `con_reintento` | ninguno (nota) |

## Notas (no abren ronda)

- El cableado del reintento en `leer_fresco` no lo protege ningun test: el unitario cubre `con_reintento` y `c11` usa un `.lock` huerfano que sale `error-local` con o sin reintento. Un test determinista exigiria un `.lock` soltado por otro proceso a mitad de la lectura.
- C1 no protege que la lectura renueve la marca (si C2, C3 y C4).
