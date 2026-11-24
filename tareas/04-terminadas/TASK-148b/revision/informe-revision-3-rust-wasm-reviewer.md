# Informe de revision — TASK-148b (ronda 3)

- Commit revisado: 17adf53be31de8a39f2613ab062babcf894b5128
- Revisor: general-purpose (haiku) con rust-wasm-reviewer
- Veredicto: aprobada

Veredicto propuesto: aprobada. IMP-2 queda corregido: el test `salida_rechaza_stderr_en_un_exito` ya no depende de un `sh` en el PATH, pasa sin `sh` y su panic procede del assert de stderr de `comun::salida`, no de otra causa. No hay hallazgos nuevos CRITICO ni introducidos por la correccion. El commit de codigo revisado es a0d9638 (HEAD de la rama al revisar): desde 759c0ca no cambia nada bajo `crates/`, solo la documentacion de tareas.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-2 | IMPORTANTE | corregido | crates/tc2/tests/soporte.rs:103 | implementacion |

Sin hallazgos nuevos. Ni CRITICO ni introducido por la correccion.

Evidencia de IMP-2 (corregido):

- Causa original reproducida en la ronda 2: sin `sh` en el PATH, el test fallaba con `NotFound` en `Command::new("sh")`.
- La correccion (759c0ca) sustituye el `sh` por `git -c alias.dos-salidas=!echo ok; echo aviso >&2 dos-salidas`. Git ejecuta el alias con su propio shell, asi que el test no necesita `sh` en el PATH.
- Con el PATH sin `C:\Program Files\Git\bin` ni `C:\Program Files\Git\usr\bin` (`Get-Command sh` no encuentra nada; `git` resuelve a `C:\Program Files\Git\cmd\git.exe`): `cargo test -p tc2 --locked --test soporte` da 9 passed, 0 failed.
- El test se queda en verde por la razon correcta: `cargo test ... salida_rechaza -- --nocapture` muestra `panicked at crates\tc2\tests\comun\lanzador.rs:68:9: stderr en un exito: aviso`, que es el mensaje esperado por el `should_panic`.
- Mutante: en el clon, el assert de stderr de `comun::salida` (`lanzador.rs:68`) cambiado por `let _ = &err;`. Resultado: `salida_rechaza_stderr_en_un_exito` FAILED con `note: test did not panic as expected at crates/tc2/tests/soporte.rs:103`. El resto (8) sigue verde. Asi, el test detecta la regresion que promete. Restaurado con `git checkout`.
- Revision del diff de la correccion (8 lineas en `soporte.rs`): el assert nuevo `assert!(o.status.success(), ...)` hace que un `git` que falle no se confunda con el stderr esperado; no abre fallos nuevos. La dependencia de `git` en el PATH ya la tienen los demas tests de soporte (`git_en_*`), no es nueva.

## Regla 4D

- Duplicacion: cumple.
- Desviacion: cumple.
- Diluvio: cumple.
- Disfraz: cumple. El test nuevo falla cuando se quita el assert de stderr que prueba (mutante de arriba).

## Alcance

- Ficheros revisados: `crates/tc2/tests/soporte.rs` (diff de 759c0ca) y `crates/tc2/tests/comun/lanzador.rs` (funcion `salida`, para el mutante).
- Contexto adicional pedido: ninguno aparte de lo anterior.
- Limite JS-WASM: sin cambios; son tests nativos de un CLI Rust, sin firmas exportadas con `wasm-bindgen`.
- IMP-1 (de la ronda 2) no se ha vuelto a verificar en esta ronda: la peticion acota la verificacion a IMP-2. Sigue como `corregido` segun el informe de la ronda 2.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148b-r3`, rama `fix/task-148-auditoria-4d-soporte-de-tests-compartido` en a0d9638. `CARGO_TARGET_DIR` dentro del clon (`target-r3`). Borrado al terminar; el clon no tenia commits propios.
- Repo original: sin checkout, switch ni commits.
- Build: `cargo test -p tc2 --locked --test soporte`, nativo (Windows), 1m 01s, sin errores.
- Formato y lints: no ejecutados (la peticion acota la verificacion al test).
- Suite: `cargo test -p tc2 --locked --test soporte`, PATH sin `Git\bin` ni `Git\usr\bin`: 9 ok, 0 FAILED. Linea base del test antes de la correccion (ronda 2): 8 ok, 1 FAILED en este mismo entorno.
- Casos construidos: `salida_rechaza_stderr_en_un_exito` sin `sh` en el PATH (IMP-2), y con `--nocapture` para ver el panic real.
- Mutacion: quitar el assert de stderr en `comun::salida` (`lanzador.rs:68`), sustituido por `let _ = &err;`: `salida_rechaza_stderr_en_un_exito` en rojo (FAILED, no panic). Restaurado.
- Medidas: no aplican.

## Revisado sin hallazgos

- `crates/tc2/tests/soporte.rs` completo (9 tests del binario `soporte`): pasan sin `sh`, y los `should_panic` panican por la causa que nombran.
- `comun::salida`: el assert de stderr en exito es el que protege el test nuevo; sin cambios en el commit de la correccion.

## Nota (MENOR, fuera de la tabla)

- El alias `dos-salidas` se pasa con `-c`, asi que no depende de la configuracion global de git. Un alias homonimo en el `.gitconfig` del usuario no puede interferir porque `-c` tiene prioridad sobre la configuracion de usuario. No abre hallazgo; se anota solo para que conste.
