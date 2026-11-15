# Informe de revision — TASK-018 (ronda 1)

- Commit revisado: 4f80989fc93337d12966d31c9be87260371a8ddd
- Revisor: rust-wasm-reviewer (agente general-purpose, sonnet)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| H1 | MENOR | corregido | crates/tc2/src/estado/montaje.rs | implementacion |
| H2 | MENOR | corregido | crates/tc2/tests/estado_casos.rs (ramas de crates/tc2/src/estado/transaccion.rs) | implementacion |
| H3 | MENOR | corregido | crates/tc2/src/git/estado.rs (`ruta_valida`) | implementacion |
| H4 | MENOR | corregido | crates/tc2/src/estado/transaccion.rs | implementacion |

Sin CRITICO ni IMPORTANTE. Sin `unwrap`, `expect`, `panic!` ni indexado sobre datos de entrada en producción. `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` y la suite, limpios.

### H1 (MENOR) — `montar` no toma el cerrojo: dos `montar` simultáneos del mismo clon dan `error`

Cuatro hilos llaman a `estado::montar` a la vez sobre un clon con el remoto ya inicializado: 1 da `montar-estado ok`, 3 dan `montar-estado error git=worktree | git worktree fallo: fatal: '.../clon/tareas' already exists`. No se pierde nada, pero `error` (1) dice «no se reintenta»; lo correcto sería `ok` o `sin-cambios`. Propuesta: cerrojo en `montar`, soltándolo antes de la transacción `Inicializar`; test con varios hilos.

### H2 (MENOR) — `tareas=con-cambios`, `tareas-con-cambios` y `diverge` sin test; el mutante que las falsea sobrevive

M4 (`adelantar` siempre `AlDia`) deja la suite verde. Comprobado a mano que el comportamiento es correcto: con remoto, `estado ok ... tareas=con-cambios` (publicado, fichero de la persona intacto); sin remoto, `estado rechazada operacion=anotar motivo=tareas-con-cambios`. Propuesta: test permanente con las dos variantes. `Diverge` es casi redundante (M9 sobrevive): probarlo o quitarlo.

### H3 (MENOR) — `ruta_valida` acepta nombres no portables

En Windows, `notas/aux.md`, `notas/a?b.md`, `notas/fin.` dan `error git=update-index ... Invalid path` (limpio); `notas/Mayus.md` y `notas/mayus.md` dan `ok` las dos, y chocarían al hacer checkout en Windows o macOS. Riesgo latente (hoy las rutas salen de IDs validados). Propuesta: conjunto portable (sin `<>:"|?*`, reservados de Windows, final `.` o espacio) y rechazar duplicados que solo difieren en mayúsculas.

### H4 (MENOR) — Una pendiente cuya re-ejecución da `Fallo::Error` bloquea todas las escrituras

`return Err(error(d))` aborta la transacción y se repite mientras la ref siga ahí, sin salida desde el CLI. Propuesta: tratarla como `Destino::Rechazada`. No reproducido empíricamente (riesgo de diseño).

## Estado tras la ronda 1 (lo anota quien implementa)

- H1 corregido: `montar` toma el cerrojo (lo suelta para la transacción de creación y vuelve a entrar); test `estado_bordes::montar_a_la_vez_desde_varios_hilos_del_mismo_clon` (1 ok y 3 sin-cambios).
- H2 corregido: test de `con-cambios`, `diverge` y `tareas-con-cambios`, con y sin remoto. `Diverge` se queda: es lo que detecta un commit a mano en `tareas/`.
- H3 corregido: `ruta_valida` portable y `rechazada motivo=ruta-duplicada` por mayúsculas; filas nuevas en el test unitario y test de integración.
- H4 corregido: una pendiente con `Fallo::Error` va a rechazadas; test `estado_bordes::pendientes_que_ya_no_proceden_o_fallan_van_a_rechazadas_sin_reutilizar_numero`.

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r1-rust\repo`, `CARGO_TARGET_DIR` dentro; borrado al terminar.
- `cargo test -p tc2 --locked`: verde (15 unitarios, cli 2, estado_casos 4, estado_concurrencia 3, git_binario 1, git_casos_limite 5, git_ciclo 1, git_conflicto 1, git_push 1).
- Mutantes: M1 cerrojo quitado ROJO; M2 contención como denegación ROJO; M3 sin borrar pendiente ROJO; M4 `adelantar` siempre `AlDia` SOBREVIVE; M5 non-ff como éxito ROJO; M6 ignorar borrado de ref temporal SOBREVIVE (ruta difícil de forzar); M7 `guardar` sin CAS SOBREVIVE (el cerrojo ya serializa); M8 `sin_red` ignora `exige_red` ROJO; M9 sin `Diverge` SOBREVIVE; M10 rechazada como publicada ROJO; M12 `.ok()` en `resolver` SOBREVIVE (no forzable).
- Puerto Git (`update-ref` acotado, índice temporal), cerrojo y portabilidad de los tests: sin hallazgos.

## Lecturas fuera del contexto permitido

`git_binario.rs`, `git_ciclo.rs` y `git_conflicto.rs` solo se ejecutaron con la suite, sin leer su código.
