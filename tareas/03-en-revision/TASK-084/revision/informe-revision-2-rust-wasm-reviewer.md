# Informe de revisión — TASK-084 (ronda 2)

- Commit revisado: 5faa3cc
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R1-01 | IMPORTANTE | corregido | crates/tc2/src/progreso/mod.rs:339 | implementacion |
| R1-02 | MENOR | corregido | crates/tc2/src/tablero.rs:283 | implementacion |
| R1-03 | MENOR | corregido | crates/tc2/src/main.rs:60 | implementacion |
| R1-04 | MENOR | corregido | crates/tc2/src/progreso/fichero.rs:177 | implementacion |
| R1-05 | MENOR | corregido | crates/tc2/src/progreso/linea.rs:223 | implementacion |
| M1 | MENOR | corregido | crates/tc2/src/progreso/linea.rs:268 | implementacion |
| M2 | MENOR | corregido | crates/tc2/src/progreso/linea.rs:282 | implementacion |
| M4 | MENOR | corregido | crates/tc2/src/progreso/mod.rs:157 | implementacion |

Sin hallazgos nuevos. M3 fuera de la parte de este revisor.

**R1-01.** Reproducido con el binario: fuera de un repo, `timeout 30 tc2 status --seguir | head -1` imprime la linea y el proceso sale solo con 0 en 4 s (dos veces); sin `tc2` vivos despues. A fichero, 2 lineas en 7 s. Repetir la linea es aceptable: el consumidor de un pipe siempre lee la linea actual; sin escribir no hay forma de detectar el cierre sin `unsafe` (prohibido); documentado en `instalar.md`, F1-17 y `REPETIR_SEGUIR`; en terminal no repite. Quien deduplique lineas debe hacerlo el.

**R1-03.** `echo '{}' | tc2 estado --stdin | head -0` da rc 0 y stderr vacio.

**R1-02, R1-04, R1-05, M1, M2, M4.** Comprobados por sus tests nuevos (en verde); R1-04 no abre fallo (el anterior es solo respaldo); M4 resuelve `gitdir` relativos.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-084-r2-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios. `cargo test --locked --workspace`: 573 ok, 0 fallos. Mutaciones en rojo: anular la repeticion (R1-01, colgado 10 s), desactivar `es_status_line` (R1-03), buscar por nombre de carpeta (M4).

## Revisado sin hallazgos

`seguir` con `repetir = None` no repite; `n.max(1)`; `gitdirs_de_tareas` tolera carpetas ilegibles; sin `unwrap`, `expect` ni `unsafe` nuevos. Limite JS-WASM: no aplica.
