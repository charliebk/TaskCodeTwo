# Informe de revisión — TASK-083 (ronda 1)

- Commit revisado: d5ced92
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | abierto | crates/tc2/src/estimacion/historico.rs:123 | implementacion |
| H2 | MENOR | abierto | crates/tc2/src/inventario.rs:606 | implementacion |
| H3 | MENOR | abierto | crates/tc2/src/estimacion/mod.rs:69 | implementacion |

**H1.** Coste cuadratico: `de_tarea` recorre los antepasados de `p1` (todo `develop`) por cada tarea terminada con merge, en cada `new`, `import`, `start` y `estimar`. Con un log sintetico de 100 000 commits y 300 merges, `Commits::de_log` y 300 `de_tarea` tardan 16,6 s en release (~55 ms por merge). Despreciable a la escala real; crece con tareas × commits. Calcular una vez el conjunto base o recorrer solo hasta el merge-base.

**H2.** `segundos_de` (`estimacion/real.rs:107`) acepta fechas que la PoC (`strptime`) rechaza: `2026-02-31T10:00:00Z`, `2026-04-31T...` y `...T10:00:60Z` dan instantes validos. Causa: `ms_de_iso` (`inventario.rs:606`) solo valida mes 1..12 y dia 1..31. La fila cuenta como valida en vez de ir a `filas_malas`; discrepa de la paridad declarada.

**H3.** `recalcular` sustituye el campo `estimacion` entero con `agente: None`: `start` borraria una estimacion del agente (D1a) ya escrita (reproducido con `agente: 25`). Hoy nadie la escribe (llega con F4-1). Conservar `agente` al recalcular.

Sin CRITICOS ni IMPORTANTES; sin `unwrap`, `expect` ni indexado que dispare panic con entrada alcanzable (indices por byte en fronteras de caracter).

## Reproduccion

Clon en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-083-r1-rust` (borrado). `fmt --check` y `clippy -D warnings` limpios. tests/estimacion.rs 9/9; toma, alta e importar 34/34; unitarios de estimacion 17/17.

Bordes probados sin panic: huecos de 2700, 2701 y 2699 s; `ini > fin`; extremos de `i64`; pausa sin reanudar; fechas ilegibles, ano 0000 y 9999; log con sha o padres vacios, `ct` no numerico o extremo, IDs que desbordan u64, `TASK-007ñ`, merges de tres padres; YAML con comillas, `\`, ñ, `n=usize::MAX`, 1e300, NaN, infinitos y negativos (saturados); percentiles con 0, 1 muestra y empates.

Mutaciones en rojo: sin tope de 45 min (C1); sin filtro C7 en `disponibles`; regla de `id_de_merge` relajada; sin `CERRADOS.contains` en `Rellenar::aplicar`.

## Revisado sin hallazgos

Aritmetica de tiempos y `ventanas_pausa`; transiciones desordenadas y `duracion-negativa`; cascada, `restante`, `error` y `minutos`; escritura y lectura de `estimacion`; integracion en `new`, `import`, `start` y `doctor` (una lectura y un `git log`; si falla el log, vacio sin romper); troceo con union-find. Limite JS-WASM: no aplica.
