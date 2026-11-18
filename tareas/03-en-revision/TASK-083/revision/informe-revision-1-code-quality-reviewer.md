# Informe de revisión — TASK-083 (ronda 1)

- Commit revisado: d5ced92
- Revisor: code-quality-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R1-01 | MENOR | abierto | crates/tc2/src/git/mod.rs:143 | implementacion |
| R1-02 | MENOR | abierto | crates/tc2/src/estimacion/mod.rs:75 | implementacion |
| R1-03 | MENOR | abierto | crates/tc2/src/estimacion/estimar.rs:267 | implementacion |
| R1-04 | MENOR | abierto | crates/tc2/src/estimacion/mod.rs:48 | implementacion |
| R1-05 | MENOR | abierto | crates/tc2/tests/estimacion.rs:707 | implementacion |
| R1-06 | MENOR | abierto | crates/tc2/src/toma/mod.rs:134 | implementacion |

**R1-01.** `git::historia` (formato de `git log`, eleccion de `origin/develop` o `develop`) no la ejercita ningun test con commits reales: cambiar `FORMATO_HISTORIA` a `"--format=%H"` deja `--test estimacion` 9/9 verde. Anadir un E2E con un commit `TASK-N` en el `develop` del remoto que cambie un real.

**R1-02.** `recalcular` sobrescribe `agente` con `null` (reproducido con `agente: Some(300)`). Conservar el `agente` previo.

**R1-03.** `restante` desbordada no da la cota de F1-17 punto 5 (mediana de las fases siguientes): con imp 10, 20, 50, 60, 70, 80, rondas 10, cierre 5 y 45 min transcurridos da `mediana=25` y la cota es 15; el test solo comprueba `n` y `desbordada`. Alinear el calculo con el ADR y fijar el valor en el test.

**R1-04.** `commits_de` traga el fallo de `git log` (`unwrap_or_default`): estimaciones distintas sin aviso. Minimo, un aviso o una clave en la salida.

**R1-05.** «Un solo `git log`» solo se prueba en `estimar TASK-NNN`, no en `new`, `import` ni `start`; no hay prueba de que `import` calcule el historico una vez para el lote.

**R1-06.** `start` calcula el historico completo dos veces (sondeo `op.aplicar(&vista)` y transaccion). Solo coste.

## Verificacion

Clon (borrado). `fmt --check` y `clippy` limpios. `cargo test -p tc2` completo: 515 ok, 6 ignorados, 0 fallos. C1-C7 demostrados salvo R1-01 y R1-05 (C1 sumado a mano: 85). Fidelidad a F1-17 (tope, pausas, cascada, umbral 5, p80, forma del punto 7 byte a byte); divergencias en la adenda salvo R1-02 y R1-03. Paridad no circular: compara contra el fixture JSON generado con `poc-f1-17.py` (el script generador `t083-paridad.py` no esta en el repo: aviso de trazabilidad). Cambios en `tests/toma.rs`, `tests/comun` y unitarios de `toma` solo anaden la estimacion. Sin duplicacion nueva. Salida coincide con F1-4 (sin test de `estimar` sin red: no verificado).
