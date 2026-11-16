# Informe de revision — TASK-026 (ronda 1)

- Commit revisado: 6609149e64b240c697b4eba4036d5e8006865279
- Revisor: rust-wasm-reviewer (general-purpose, sonnet)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | xtask/src/instaladores.rs:88-92 | implementacion |
| MEN-2 | MENOR | corregido | xtask/src/instaladores.rs:64-67 | implementacion |

### MEN-1: la prueba "sin exit" solo ve `exit` al principio de una línea

El test comprueba `l.trim_start().to_ascii_lowercase().starts_with("exit")`. Esa regla se salta cualquier `exit` que no abra la línea. Con `irm | iex` cualquiera de ellos cerraría la ventana de la persona.

Mutantes sobre `instalar/install.ps1`, con `cargo test -p xtask --locked instaladores`:

- `exit 1` como primera línea del cuerpo de `Install-Tc2`: detectado (la prueba falla en la línea 89 del fichero de tests).
- `[Environment]::Exit(1)` insertado tras `function Install-Tc2 {`: NO detectado (3 passed).
- `if ($false) { exit }` insertado igual: NO detectado (3 passed).

Además, `starts_with("exit")` daría falso positivo con un identificador como `exito`, aunque hoy no existe. No es un fallo real hoy: el `install.ps1` actual no contiene `exit` ni `Exit(`. Propuesta: buscar el token `\bexit\b` en cualquier posición de las líneas que no sean comentario, y añadir `[Environment]::Exit`.

### MEN-2: la prueba de alias compara conjuntos y no la asignación uname → alias

`la_tabla_de_alias_de_los_instaladores_es_la_de_p12` solo comprueba que el conjunto de alias del script sea el de `dist::TARGETS`. Si se cruzan las ramas de `detectar_alias`, el test sigue en verde.

- Mutante: en `install.sh`, `Linux/x86_64 ... echo linux-x64` pasa a `echo linux-arm64`, y `Linux/aarch64 ... echo linux-arm64` pasa a `echo linux-x64`. Resultado: `test result: ok. 3 passed`. NO detectado.
- Un Linux x64 bajaría entonces el binario arm64. Falla pronto, porque `--version` no arrancaría. Aun así, ningún test lo vería.

Propuesta: o se asume y se documenta como límite (la ejecución real de los instaladores lo cubre en los ensayos de la tarea), o se añade un test que mapee `uname -s/-m` a alias. Es MENOR porque `detectar_alias` se ejercita en otra parte de la tarea. No he revisado esa parte, que queda fuera de mi contexto.

## Alcance

- Ficheros revisados: `xtask/src/instaladores.rs`, `xtask/src/main.rs` (solo `#[cfg(test)] mod instaladores;`). Ver "Lecturas fuera del contexto permitido" para lo demás.
- Contexto adicional pedido: `instalar/install.sh`, `instalar/install.ps1` y `xtask/src/dist.rs` (`TARGETS`), necesarios para mutar y contrastar.
- Limite JS-WASM: no aplica. No hay `#[wasm_bindgen]` ni cambios de firma exportada.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-026-r1-rust`, rama `feature/task-026-f2-11-instaladores-de-una-linea`. HEAD del clon: a5ab364. Solo añade la petición y `tarea.md` sobre 6609149, sin cambios en `xtask` ni `instalar`. Borrado al terminar, sin nada que conservar (clon limpio tras cada mutante con `git checkout`).
- Build, formato y lints:
  - `cargo test -p xtask --locked` compila sin errores.
  - `cargo fmt --check -p xtask`: sin salida (limpio).
  - `cargo clippy -p xtask --all-targets --locked -- -D warnings`: "No issues found".
  - No se compiló a wasm32: xtask es nativo.
- Suite: `cargo test -p xtask --locked`: 11 passed, 0 failed.
- Mutaciones sobre los 3 tests nuevos:

| Mutante | Detectado |
|---|---|
| sh: alias linux-arm64 cambiado a arm65 | sí |
| sh: darwin-x64 cambiado a x65 | sí |
| sh: eliminada la rama linux-arm64 | sí |
| sh: alias extra `darwin-armx` | sí |
| ps1: win-arm64 cambiado a arm65 | sí |
| ps1: win-x64 cambiado a x65 | sí |
| ps1: comentario `# linux-x64` | sí (alias de más) |
| ps1: comentario con `Win-X64` en mayúsculas | no, y es correcto |
| carácter no ASCII en sh | sí |
| carácter no ASCII en ps1 | sí |
| CRLF en sh | sí |
| ps1: línea `exit 1` al inicio de la función | sí |
| ps1: `[Environment]::Exit(1)` | **no (MEN-1)** |
| ps1: `if ($false) { exit }` | **no (MEN-1)** |
| sh: comentario tras `main "$@"` | sí |
| sh: `main` sin `"$@"` | sí |
| sh: sin llamada final | sí |
| ps1: línea tras `Install-Tc2` | sí |
| ps1: `EXIT` al final | sí (falla por la llamada final) |
| sh: `main "$@"` con sangría | no (el `trim` lo tolera, comportamiento intencionado) |
| sh: intercambio de salidas linux-x64 y linux-arm64 | **no (MEN-2)** |

- Mutantes inválidos descartados: un primer intento de renombrar linux-arm64 no aplicó el `sed` (patrón mal escrito) y lo repetí bien. Uno de `exit` falló al escribir un fichero temporal y también lo repetí.

## Revisado sin hallazgos

- `alias_en`: respeta la palabra entera (`twin-x` no cuenta). El indexado `texto[..i]` no entra en pánico, porque `match_indices` devuelve índices en límite de carácter y se usa `next_back`. No hay `unwrap` ni `panic` fuera del `unwrap_or_else(panic!)` del helper de test, que lleva `#![allow(clippy::panic)]` justificado en el propio fichero.
- `leer` usa `CARGO_MANIFEST_DIR/../instalar`, válido en cualquier directorio de trabajo.
- Un comentario con un alias real en el ps1 cuenta como alias de más (probado), lo cual es un comportamiento estricto y aceptable.
- Gating `#[cfg(test)] mod instaladores;` en `main.rs` correcto: no entra en el binario.

## Lecturas fuera del contexto permitido

- `instalar/install.sh` e `instalar/install.ps1`: leídos enteros para construir los mutantes. Motivo: son el objeto que prueban los tests.
- `xtask/src/dist.rs` (primeras 80 líneas, `TARGETS` y `es_windows`): motivo, contrastar la tabla P12 que usa el test.
- `.cargo` y `CLAUDE.md`: no leídos. Se ejecutó por error `cargo test` una vez dentro de `tareas/.../revision` del repo original: no modificó nada (`git status` limpio, `target/` ignorado y ya existente).
