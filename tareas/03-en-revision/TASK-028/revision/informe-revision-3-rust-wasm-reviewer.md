# Informe de revision — TASK-028 (ronda 3)

- Commit revisado: baef020e65d468e9fae8fcb9d6b62530091616dd (el HEAD de la rama, 5c1d4a5, solo añade ficheros de tareas/; el codigo es el de baef020)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/src/init/inventario.rs:284 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/init/inventario.rs:51 | implementacion |
| MEN-6 | MENOR | abierto | crates/tc2/src/init/inventario.rs:371 | implementacion |
| MEN-7 | MENOR | abierto | crates/tc2/src/init/inventario.rs:384 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 (corregido) — historia local sin relacion con origin

Verificado con el binario `target/debug/tc2.exe` del commit baef020 y un remoto bare con `main`
(commit `inicial`). En el clon, un commit propio sin relacion con ese `main`:

- Escenario A, sin fetch: `develop` sale `decision=persona motivo=historia-sin-relacion
  orden=git fetch origin main && git branch develop origin/main`. Ninguna orden lleva `git push`
  (0 coincidencias). Sale el aviso.
- Escenario B, con `git fetch`: igual que A. 0 `git push`.
- Mutacion: `let sin_relacion = false && hay_commits && ...` en `inventario.rs`. El test
  `historia_local_sin_relacion_no_propone_publicar` queda en rojo
  (`crates/tc2/tests/init.rs:829`: left `Some("con-historia")`, right `Some("historia-sin-relacion")`).
  Revertido con `git checkout`; el arbol del clon quedo limpio.

### MEN-5 (abierto, resuelto solo para comilla ASCII) — `comillas()` en PowerShell

- Resuelto para `'` ASCII: `comillas(s, true)` duplica la comilla. Los tests `comillas_para_powershell`
  y `comillas_para_sh` pasan. El parser de PowerShell (solo analisis, nada ejecutado) da una sola
  sentencia y 0 errores para `git worktree move 'C:/r/a''; calc; ''b $x y' tareas`.
- Residuo 1: PowerShell tambien trata como comillas simples U+2018, U+2019, U+201A y U+201B. Con una
  ruta que contiene U+2019 (`C:/r/a’; calc; ‘b`), `comillas()` no la escapa y el parser da 4
  sentencias y 1 error (`git worktree move 'C:/r/a’ | calc | tareas`). Riesgo bajo: la ruta sale de
  `git worktree list` del propio repo.
- Residuo 2: el modo lo decide `cfg!(windows)` en el binario (`inventario.rs:202`), no la shell de
  la persona. Con Git Bash o cmd en Windows, una ruta con `'` sale con la forma de PowerShell y en
  `sh` se concatena mal (`'C:/a''b'` da `C:/ab`) sin error.
- Lo que falta: el test no cubre U+2019 ni la eleccion de shell. Arreglo: escapar tambien las
  comillas tipograficas (verificar con el parser) o no imprimir la orden si la ruta trae comillas.

### MEN-6 (nuevo, introducido por la correccion) — aviso y motivo afirman "sin relacion" sin comprobarlo

Escenario D: el clon tiene `main` en C1; `origin/main` avanza a C2 (hijo de C1), y se borra
`refs/remotes/origin/main` con `git update-ref -d`. Resultado de `init --simular`: `develop`
`motivo=historia-sin-relacion`, sin push, y el aviso "la historia de este clon no tiene nada en
comun con la de origin". Pero tras `git fetch`, `git merge-base main origin/main` da `067a588`,
que es el `main` local: si hay historia comun.

`historia_relacionada()` devuelve `false` cuando no puede comprobar (su comentario lo dice: "Si no se
puede saber, `false`"), y el aviso lo presenta como hecho. El lado prudente (no publicar) se
mantiene, pero el texto afirma algo no verificado. Arreglo: distinguir "sin relacion" de "no
comprobado" (el aviso debe decir que falta fetch) y que el texto no afirme lo que no se midio.

### MEN-7 (nuevo, introducido por la correccion) — accion `persona` sin orden

Escenario C: repo con `main` y `develop` locales en el mismo commit, `origin` con un `main` sin
relacion, sin fetch. Salida: `accion ruta=refs/heads/develop ... decision=persona
motivo=historia-sin-relacion` sin campo `orden`, y el resumen dice `2 acciones las haces tu
(decision=persona): cada linea trae la orden`.

Causa: `inventario.rs:382-385`. La rama `remoto && !en_remoto && sin_relacion` fija
`decision=Persona` y `motivo`, pero no llama a `.orden()` (a diferencia de `persona()`). El texto
promete una orden que no sale. Arreglo: dar la orden que corresponda o cambiar el texto del resumen.

## Alcance

- Ficheros revisados: `crates/tc2/src/init/inventario.rs`, `crates/tc2/src/init/mod.rs` (aviso,
  lineas 291-300, y `Accion`), `crates/tc2/src/i18n/catalogo/init.rs` (`AVISO_SIN_RELACION`) y
  `crates/tc2/tests/init.rs` (diff de la ronda).
- Contexto adicional pedido: ninguno. No hay cambio de firma exportada, asi que no hizo falta
  llamador ni `.d.ts`.
- Limite JS-WASM: sin cambios. El crate no tiene objetivo `wasm32` en esta rama y el diff no toca
  `#[wasm_bindgen]` ni tipos que crucen el limite.
- Produccion nueva sin `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` ni `unsafe`: grep sobre
  las lineas anadidas de `2da8748..baef020` en `crates/`. Unicas coincidencias en
  `crates/tc2/tests/init.rs` (`unwrap` y un `clone` de test).

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r3-rust`, rama
  `feature/task-028-f3-1-taskctl-init-setup`, HEAD `5c1d4a5` (codigo de `baef020`). Repos de prueba:
  `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r3-repos`. Ambos se borran
  al terminar; el clon no tiene commits propios.
- Build: `cargo build -p tc2 --locked` (nativo), sin errores. El objetivo `wasm32` no aplica.
- Formato y lints: `cargo fmt --check -p tc2` sin diferencias. `cargo clippy --all-targets --locked
  -- -D warnings` sin avisos.
- Suite de integracion: `cargo test -p tc2 --test init --locked`, 18 passed, 0 failed (incluye
  `historia_local_sin_relacion_no_propone_publicar`, `montada_en_otra_ruta_da_la_orden_entrecomillada`
  y `remoto_con_historia_no_es_repo_vacio`).
- Unitarios: `cargo test -p tc2 --lib --locked init`, 5 passed, 0 failed (`comillas_para_sh`,
  `comillas_para_powershell`, `misma_regla_con_o_sin_barra_final`, y los dos de `init::tests`).
- Casos construidos: A, B, C y D descritos arriba, con remoto bare local (`git init --bare` por clon).
- Mutacion: `sin_relacion = false && ...` en `inventario.rs`, test de IMP-1 en rojo. Revertida.
- Parser de PowerShell (solo analisis): comillas ASCII, 1 sentencia; U+2019, 4 sentencias y 1 error.
- Medidas: no aplican.

## Revisado sin hallazgos

- Ramas con repo vacio y con historia relacionada (`ramas()` y los tests existentes, suite en verde).
- Ausencia de `unwrap`, `expect`, `panic!` y `unsafe` en produccion (ver Alcance).

## Lecturas fuera del contexto permitido

- `tareas/03-en-revision/TASK-028/revision/informe-revision-2-rust-wasm-reviewer.md` y
  `peticion-revision-3-rust-wasm-reviewer.md`: indicadas en el encargo de la ronda.
- `tareas/03-en-revision/TASK-028/revision/informe-revision-3-rust-wasm-reviewer.md` (plantilla):
  indicada en el encargo.
- `task-workflow/revision.md` del plugin taskcode 0.16.1: criterios de la linea de veredicto (solo
  para orientarse; el veredicto se deja en PENDIENTE).
