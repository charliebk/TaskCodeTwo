# Informe de revisión — TASK-084 (ronda 1)

- Commit revisado: 4cc0c36
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| M1 | MENOR | abierto | crates/tc2/src/progreso/linea.rs:60 | implementacion |
| M2 | MENOR | abierto | crates/tc2/src/progreso/linea.rs:94 | implementacion |
| M3 | MENOR | abierto | docs/distribucion/instalar.md:95 | implementacion |
| M4 | MENOR | abierto | crates/tc2/src/lib.rs:353 | implementacion |

**M1.** La eleccion del tramo de la tabla (`lo_que_falta`) no esta sujeta: `.floor()` → `.round()` deja `--lib progreso` 18/18 verde (los tests solo usan 0, 20 y 70 min). Falta un caso a mitad de tramo (22 o 23 min).

**M2.** El umbral «sin actividad registrada» no se prueba en su frontera: `abierto > UMBRAL_MIN` → `> UMBRAL_MIN * 2.0` deja todo verde (unico caso, 127 min). Probar 45 y 46 min.

**M3.** La recomendacion de `.claude/settings.local.json` (`instalar.md:95-98` y `RUTA_STATUS_LINE` de `init/inventario.rs`) no tiene cita oficial: la pagina de status line solo dice «user settings» o «project settings». Citar `settings.md#where-settings-live`.

**M4.** La rama de `reasignar` en la lista de borrado de `de_una_tarea` no tiene test (`tests/progreso.rs` no la usa); `descartar` y `liberar` se borrarian igual por `refrescar`. Ademas `progreso/mod.rs:157` asume que la carpeta administrativa del worktree se llama como el ID: con sufijo de colision no se ve `logs/HEAD`.

## Verificacion

Clon (borrado). `--test progreso` 8/8, `--test estimacion` 11/11, `--lib` 280/280, frescura, identidad, init, desinstalar, tablero, toma y ciclo en verde (107). C1-C11 demostrados por sus tests (detalle en el texto del revisor). Linea conforme a L5 de F1-17 y su adenda. Sin segunda formula del restante: `calculo.rs` guarda la tabla con `estimacion::restante` (test que compara los 97 puntos). Migracion a `escribir_local` sin cambio de comportamiento (solo cambia el nombre del temporal de identidad, efimero). Ningun test existente debilitado. `instalar.md` y la propuesta de `init` coinciden con https://code.claude.com/docs/en/statusline (2026-11-18) salvo M3. Sin duplicacion real.

Ya documentados en la autorrevision: Ctrl+C sin codigo 0; liberar/reasignar desde otro clon; `desinstalar` y `settings.local.json`; ficheros de progreso de tareas cerradas con la 0.16 hasta F6-2/F7-1; `--seguir` con stdout cerrado solo termina al escribir.
