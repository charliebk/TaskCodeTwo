# Informe de revision — TASK-125 (ronda 1)

- Commit revisado: 41944471c24da878a2e28fc776ebb4a2b3aab7e5
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

sin hallazgos.

## Alcance
- Ficheros revisados: crates/tc2/src/ciclo/liberar.rs, crates/tc2/src/situacion.rs.
- Limite JS-WASM: sin cambios de firma exportada (solo se elimina una funcion publica de Rust nativo sin llamadores).

## Reproduccion
- Clon: %LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-125-r1-rust, checkout 4194447 (borrado al terminar).
- Suite: no repetida. `cargo xtask verificar --perfil rapida` en verde sobre d52d640 ("xtask: verificacion Rapida en verde", log verif-125.log); `git diff --stat d52d640 4194447 -- crates docs` vacio, codigo identico.
- Tests concretos: `cargo test -p tc2 --lib -- situacion tablero liberar` 17 ok; `cargo test -p tc2 --test liberar` 12 ok.
- Formato y lints: `cargo fmt --check` sin diferencias; `cargo clippy -p tc2 --all-targets` sin avisos.
- Doc: `cargo doc -p tc2 --no-deps` da 10 avisos en el commit revisado y 10 en develop; no hay avisos nuevos (el enlace `[crate::ciclo::liberar::actividad_conocida]` resuelve).
- Llamadores: `situacion::actividad` no tiene usos (grep sobre crates); `tablero.rs:190` y `liberar.rs:140` usan `actividad_conocida`.
- Comentarios: en situacion.rs ya no queda texto que diga que una fecha futura cuenta como de hoy (`dias_sin_actividad` explica que la resta satura dentro del `MARGEN_FUTURO`).
- Mutacion: en `actividad_conocida`, cambiar a que sin fila devuelva `None` (`conocida(ultima).and_then(...)`) pone en rojo `ciclo::liberar::tests::actividad_conocida_sin_fechas_del_futuro` (left None, right Some(rama)): la asercion nueva discrimina. Mutacion revertida (`git checkout`).

## Revisado sin hallazgos
- Criterio 1 (la funcion no existe y no queda comentario erroneo) y criterio 2 (tests en verde).
- El test `dias_sin_actividad_completos` conserva los bordes (6/7 dias, actividad anterior a la fila da 0).
- Los comentarios de liberar.rs ligados a MARGEN_FUTURO se dejan a proposito (plan D1a-D3a).

## Lecturas fuera del contexto permitido

- crates/tc2/src/tablero.rs (solo grep de la linea 190): comprobar que `actividad_conocida` es la regla unica de `board`.
