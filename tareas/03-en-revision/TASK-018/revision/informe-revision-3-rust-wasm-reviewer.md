# Informe de revision — TASK-018 (ronda 3)

- Commit revisado: 36d96aa
- Revisor: rust-wasm-reviewer (agente general-purpose, haiku)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R3-01 | MENOR | corregido | crates/tc2/src/estado/transaccion.rs (`duplicada` y `choque`, desde `intento`, `sin_red` y el bucle de pendientes) | implementacion |

R2-01 de la ronda 2: corregido y verificado (ver abajo).

### R3-01 — MENOR — El choque ya no ve colisiones dentro de la misma operación (ni con el ROADMAP.md generado)

`choque` recibe `vista.rutas()` tomado antes de superponer los cambios, así que no compara las rutas de un mismo lote entre sí; la versión anterior sí. También se perdió la comprobación sobre el `ROADMAP.md` generado (hoy `roadmap` es `None`: latente). Reproducción con una operación que devuelve `notas/A.md` y `notas/a.md` en un mismo `aplicar`, repo sin remoto:

- 36d96aa: `estado ok operacion=dos commit=bd2b2435bc72 intentos=1 tareas=al-dia` (publica las dos).
- `transaccion.rs` de 09e487e: `estado rechazada operacion=dos motivo=ruta-duplicada`.

Ninguna operación real emite dos rutas así en un lote: MENOR. Propuesta: comparar también las rutas de `cambios` entre sí y mantener la comprobación del `ROADMAP.md` generado.

## Estado tras la ronda 3 (lo anota quien implementa)

- R3-01 corregido: `choque` compara cada ruta con las de la rama y con las anteriores del mismo lote, y el `ROADMAP.md` generado vuelve a comprobarse antes de superponerse. Test `estado_bordes::dos_rutas_del_mismo_lote_que_chocan_por_mayusculas`, rojo con el mutante que no acumula las rutas del lote.

## Verificación de R2-01

- La pendiente `notas/Mayus.md` que choca con `notas/mayus.md` publicado va a rechazadas y las escrituras siguientes se publican; `refs/taskcode/rechazadas/000001` creada.
- Test `estado_bordes::pendiente_que_choca_por_mayusculas_no_bloquea_el_clon`, en verde.
- Mutante: quitar la guarda `Ok(cambios) if choque(...)` → falla en `estado_bordes.rs:345` (`left: None, right: Some("1")`). Restaurado.
- `clasificar_fallo_remoto`: sin fallos reproducibles; `Access is denied` de Windows no coincide con `access denied`.

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r3-rust` (36d96aa), `CARGO_TARGET_DIR` dentro; borrado al terminar.
- `cargo test --locked -p tc2`: 43 pasados (12 suites, 100,6 s). `cargo clippy --locked --all-targets -- -D warnings`: limpio.
- Prueba temporal de R3-01 en 36d96aa y en 09e487e; mutante de R2-01 rojo.

## Lecturas fuera del contexto permitido

`crates/tc2/src/estado/lectura.rs`, `crates/tc2/tests/comun/ops.rs` y las firmas de `crates/tc2/tests/comun/mod.rs`.
