# Informe de revision — TASK-018 (ronda 4)

- Commit revisado: b2f565d50585629c79df41974f1d7af5ee648930
- Revisor: rust-wasm-reviewer (agente general-purpose, haiku)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| R4-01 | MENOR | abierto | crates/tc2/src/git/remoto.rs | implementacion |

R3-01 de la ronda 3: corregido y verificado.

### R4-01 — MENOR — La marca genérica `connection` reclasifica fallos de credenciales y de disco

`transporte` incluye `connection` y anula también la regla de credenciales. Sonda en b2f565d frente a 36d96aa:

- `git@h: Permission denied, please try again.` / `Permission denied (password).` / `fatal: Could not read from remote repository.` / `Connection closed by 1.2.3.4 port 22`: 36d96aa `credenciales`; b2f565d `sin-red`.
- `error: cannot lock ref 'refs/heads/x': Permission denied` / `hint: connection lost` (construido): 36d96aa `error-local`; b2f565d `sin-red`.

Sin pérdida de datos; ningún test cubre el borde. Propuesta: quitar `connection`; `connect to host`, `failed to connect` y `could not resolve` cubren el caso que motivó la corrección.

## Lo verificado de la corrección

- `choque` acumula las rutas del lote: dos rutas que solo difieren en mayúsculas dan `ruta-duplicada`; test `dos_rutas_del_mismo_lote_que_chocan_por_mayusculas` en verde; duplicado exacto y sobrescritura no chocan.
- Comprobación del `ROADMAP.md` generado antes de superponer y de construir el commit (latente, verificada por lectura).
- Mutante (sin `vistas.push(&c.ruta)`): el test anterior en rojo (`estado_bordes.rs:429`). Restaurado.
- `transporte`: `red_bloqueada_con_permiso_denegado_es_sin_red` pasa.

## Estado tras la ronda 4 (lo anota quien implementa)

- R4-01 corregido: fuera la marca `connection` de `transporte`; test `red_bloqueada_con_permiso_denegado_es_sin_red` ampliado con los dos textos de la sonda (`Connection closed` tras rechazar la contraseña → credenciales; `cannot lock ref ... Permission denied` + `hint: connection lost` → local).

## Verificación ejecutada

- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-018-r4-rust` (26727c3; `crates/` idéntico a b2f565d), `CARGO_TARGET_DIR` dentro; borrado al terminar.
- `cargo test --locked -p tc2`: 45 pasados. `cargo clippy --locked --all-targets -- -D warnings`: limpio.
- Mutante de R3-01 rojo; sonda temporal del clasificador comparando 36d96aa y b2f565d.

## Lecturas fuera del contexto permitido

`crates/tc2/src/estado/lectura.rs` (fragmento de `Vista::rutas` y `superponer`).
