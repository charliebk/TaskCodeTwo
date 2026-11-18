# Informe de revisión — TASK-081 (ronda 1)

- Commit revisado: 101e86a
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| M-1 | MENOR | abierto | crates/tc2/src/tablero.rs:187 | implementacion |
| M-2 | MENOR | abierto | crates/tc2/src/ciclo/mod.rs:248 | implementacion |
| M-3 | MENOR | abierto | crates/tc2/src/ciclo/liberar.rs:115 | implementacion |
| M-4 | MENOR | abierto | crates/tc2/src/ciclo/mod.rs:165 | implementacion |
| M-5 | MENOR | abierto | docs/plan/PLAN-TASKCODE-2.md:337 | implementacion |

**M-1.** El borde exacto del umbral en `status`/`board` no tiene test: `d >= umbral` → `d > umbral` deja verdes `--lib inactivas` y `--test tablero c9` (el unitario prueba 16 dias contra 7 y 17, nunca 16). C9 dice «supera N dias» y el codigo y las adendas «N dias o mas». Anadir `d == umbral` y alinear la redaccion de C9.

**M-2.** La rama `para_mi` de `paso_dejar` no tiene test en rojo: quitar `&& !para_mi` (linea 248) o `Ok(if false {` (linea 270) deja `--test liberar` 12/12. Sin la 270, `reasignar TASK-N <yo>` o un `liberar` repetido quitaria el worktree publicado de quien la lanza (no pierde datos). Test con el dueno que se reasigna a si mismo con worktree publicado.

**M-3.** D2a escrita dos veces: `liberar.rs:115` (`cuerpo.ultima_fecha().max(self.actividad_rama)`) y `situacion::actividad`; si cambia una copia, `status` y `liberar` discreparian. Ademas `ciclo::dias_sin_actividad` (umbral) y `situacion::dias_sin_actividad` (dias transcurridos) comparten nombre con significados distintos.

**M-4.** Una configuracion invalida hace caer el umbral a 7 sin avisar (`de_base(...).map_or(DIAS_POR_DEFECTO, ...)`), contra la precedente de `identidad::clave_plataforma`. Distinguir `sin-base`/`sin-config` (7) de configuracion invalida (rechazo o aviso).

**M-5.** El plan sigue diciendo «≤ 12 claves» en F2-7 (`PLAN-TASKCODE-2.md:337`); anadir una nota.

Observacion sin severidad, no reproducida: `comprobar_id_salvo` recibe la rama derivada del titulo cuando la tarea no tiene `rama:` (`toma/mod.rs:176`); improbable.

## Verificacion

`fmt --check` y `clippy -D warnings` limpios; `--lib` 237 ok; `--test liberar` 12/12 (dos veces); `--test tablero` 6/6; `--test config` 9/9; suite completa cortada por tiempo en `tests/init` (SIGTERM, sin fallos hasta ahi). C1-C12 demostrados. Mutaciones: `k < dias` → `k <= dias` rojo; `exige_red` false rojo en unitarios; sobreviven M-1 y M-2. Anotaciones y lineas coinciden con las adendas de F1-2, F1-3 y F1-9. `tests/tablero.rs` solo anade `c9_c10_inactivas_y_ya_no_tuyas`. `dias_sin_actividad` y `TOPE_CLAVES = 13` coherentes con el esquema y F1-11 (0, 366, -1, `"x"`, 7.5, `null` y 70000 dan error; 1 y 365 pasan).
