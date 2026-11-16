# Informe de revision — TASK-026 (ronda 2)

- Commit revisado: dedddf7567e00cff3b26cb454c4e8b9ac67e7822 (HEAD de la rama ahora es ff07e6a; su diff contra dedddf7 solo toca `tareas/`, sin codigo)
- Revisor: rust-wasm-reviewer (general-purpose, haiku)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Sin hallazgos. Ningun hallazgo nuevo de CRITICO ni introducido por la correccion, y la verificacion de los dos tests nuevos (mutaciones) cazo los fallos que tenia que cazar.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-026-r2-rust`, con `git clone` local y checkout de `dedddf7`. Borrado al terminar.
- Suite: `cargo test -p xtask --locked` → `cargo test: 13 passed (1 suite, 0.01s)` (en verde sobre dedddf7).
- Clippy: `cargo clippy -p xtask --all-targets --locked -- -D warnings` → `cargo clippy: No issues found`, salida 0.
- Revision del diff de la correccion en `xtask/src/instaladores.rs`:
  - `DETECCION`, `cada_sistema_detectado_da_su_alias`: cada clave es una sola linea en su script (`Linux/x86_64`, `Linux/aarch64`, `Darwin/x86_64`, `Darwin/arm64` en `install.sh`; `'AMD64'`, `'ARM64'` en `install.ps1`), y se compara el alias exacto de esa rama. No hay indices de string que puedan entrar en panico: los cortes de `alias_en` y de `tiene_palabra` caen siempre en limites de caracter (prefijos y palabra ASCII, `to_ascii_lowercase` preserva los bytes).
  - `tiene_palabra`: la comprobacion de palabra entera distingue `exit` de `$exito` y `Exit_Code`, y acepta `[Environment]::Exit(1)`. Test propio presente y verde.
  - Test de exit reescrito: filtra comentarios (`#` al inicio) y busca la palabra en cualquier sitio de la linea. Con las mutaciones de abajo, el mensaje lista la linea concreta.
- Mutacion 1 (cruzar ramas): en `instalar/install.sh`, lineas 101-102, se intercambian `echo linux-x64` y `echo linux-arm64`.
  - Resultado: `cada_sistema_detectado_da_su_alias` ROJO: `install.sh: Linux/x86_64 tiene que dar linux-x64`. 12 pasan, 1 falla. Restaurado con `git checkout`.
- Mutacion 2a: en `instalar/install.ps1`, linea 48, se mete `    [Environment]::Exit(1)`.
  - Resultado: `instaladores_ascii_y_con_la_llamada_en_la_ultima_linea` ROJO: `install.ps1 usa exit: ["    [Environment]::Exit(1)"]`. Restaurado.
- Mutacion 2b: en `instalar/install.ps1`, linea 48, se mete `    if ($false) { exit }`.
  - Resultado: el mismo test ROJO: `install.ps1 usa exit: ["    if ($false) { exit }"]`. Restaurado.
- Clasificacion de lo revisado: ninguna de las mutaciones dejo tests en verde; no hay test nuevo que siga verde con el comportamiento revertido dentro de lo que la correccion cubre.
- Nota (no es hallazgo, fuera de alcance): el test de exit no detecta `$host.SetShouldExit(...)` ni otras formas de terminar el proceso que no usen la palabra `exit`. Es un caso que el propio test no pretende cubrir.

## Lecturas fuera del contexto permitido

Ninguna. Lo que se leyo fue `xtask/src/instaladores.rs` (en la lista permitida) y un `grep` de las ramas de deteccion y de `exit` sobre `instalar/install.sh` e `instalar/install.ps1` (tambien permitidos). No se leyo `dist::TARGETS` ni `tarea.md`/`planificacion/`: los tests dependen de `dist::TARGETS` y no se verifico su contenido por separado.
