# Informe de revisión — TASK-033 (ronda 1)

- Commit revisado: ded3359
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

### MENOR-1: `start` sale `rechazada` (3) cuando la toma ya esta publicada y la carpeta del worktree esta ocupada

`crates/tc2/src/toma/mod.rs`, `paso()`: la toma se publica en la transaccion y despues `crear_rama_y_worktree` rechaza con `carpeta-ocupada`. Reproducido en un clon temporal con `.taskcodetwo/worktrees/TASK-001/basura.txt`: salida `start rechazada id=TASK-001 rama=feature/task-001-primera motivo=carpeta-ocupada estado=50ee790bfd05`, codigo 3; en el remoto la tarea queda `en-curso` asignada y sin rama; repetir da el mismo rechazo. Sin perdida de datos (`estado=` avisa), pero la carpeta ocupada se puede comprobar antes de publicar.

### Observacion (no es hallazgo)

`comprobar_id` rechaza con `ocupada` cualquier rama local `*/task-nnn-*` de una tarea planificada, aunque sea propia; coherente con F1-3 y falla antes de escribir.

## Reproduccion

- Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-033-r1-rust` (borrado), `CARGO_TARGET_DIR` dentro.
- `cargo test --locked -p tc2 --test toma`: 11 pasan; `--lib toma`: 8 pasan; `fmt --check` y `clippy -D warnings` limpios.
- Mutaciones: anular la comprobacion de dependencias pone rojos `rechazos_por_estado_dependencias_y_formato` y `c4_...`; anular el atajo «ya es mia y empezada» pone rojos su unitario y `c2_...`.

## Revisado sin hallazgos

`Tomar::aplicar` sin unwrap/expect/indexado y `RefCell` reiniciados por intento; `partes_de_rama` ida y vuelta y rechazo de IDs con sufijo; `anadir_transicion` conserva el cuerpo (sin tabla, sin seccion, CRLF); refactor de `identidad::resolver`/`destino_de`/`adaptador` y `tablero` sin cambio de comportamiento; `desde_anotacion` sin panico con anotaciones malformadas. Limite JS-WASM: no aplica.
