# Informe de revisión — TASK-081 (ronda 1)

- Commit revisado: 101e86a
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | abierto | crates/tc2/src/git/rama.rs:98 | implementacion |
| H2 | MENOR | abierto | crates/tc2/src/ciclo/liberar.rs:116 | implementacion |

**H1 (hueco de test).** `ramas_ajenas` compara la rama propia por nombre exacto (correcto), pero ningun test lo distingue de un comparador mas laxo: comparar solo el slug (`split_once('/')`) o `!r.starts_with("feature/")` deja verdes `--lib rama` (14) y `la_rama_propia_no_debilita_ocupada`. Con la propia `feature/task-001-uno`, una ajena `fix/task-001-uno` o `feature/task-001-otro` dejaria de dar `ocupada`. Anadir esos dos casos al unitario de `git/rama.rs` (~linea 507).

**H2 (politica).** Una actividad con fecha futura cuenta como de hoy (`dias_sin_actividad` satura a 0) y `Dejar::permiso` usa el maximo de la ultima fila y el commit de la rama: un reloj adelantado bloquea liberar o reasignar la tarea de otra persona hasta esa fecha mas el umbral. Documentado y cubierto (`otra_persona_activa_o_inactiva`, caso `AHORA + 30*DIA`). Opcional: acotar la actividad a `ahora` o anotarlo como limitacion.

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-081-r1-rust` (borrado). `clippy -D warnings` y `fmt --check` limpios. `--test liberar --test tablero --test toma --test ciclo` en verde (15, 12, 6, 13); `--lib ciclo` 15. Mutaciones en rojo: sin umbral (`k < 0`), sin actividad de la rama, sin `requiere-persona`. Sobreviven las de `ramas_ajenas` (H1).

Bordes por lectura y tests: logins con espacios o control rechazados por `login_valido`; filas con fecha ilegible se saltan; worktree en uso `conservado en-uso`; sin publicar se rechaza antes de escribir (c8); heredado (c12); la rama nunca se toca y `quitar_worktree` exige HEAD contenido en `origin/<rama>`; sin red (c7). Carrera real entre dos clones sin test concurrente; por lectura, `aplicar` se reejecuta sobre el estado fresco y los `RefCell` se reinician por intento. Sin unwrap/expect/panic en produccion.

## Revisado sin hallazgos

`Dejar::permiso` (orden `activa` antes de `requiere-persona`, dueno heredado); anotaciones ida y vuelta y malformadas; `comprobar_id_salvo` en `start`; `ya_no_tuyas` e `inactivas` solo lectura con codigo 0; clave `dias_sin_actividad` (1-365, defecto 7). Limite JS-WASM: no aplica.
