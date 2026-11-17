# Informe de revision — TASK-028 (ronda 2)

- Commit revisado: 2da874858cbdd8149b00e8c3fa4fe864bd3a560a
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | abierto | crates/tc2/src/init/inventario.rs:342 | implementacion |
| MEN-5 | MENOR | abierto | crates/tc2/src/init/inventario.rs:49 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 (abierto, resuelto solo en parte) — `develop` con historia local ajena a `origin`

La correccion resuelve el caso del repo local sin commits: `ramas()` ya no lo considera vacio si
`origin` tiene `main`, y la linea de `develop` pasa a `git fetch origin main && git branch develop
origin/main && git push origin develop` (cubierto por `remoto_con_historia_no_es_repo_vacio`).

Queda abierto el caso con commits locales propios: `hay_commits` es verdadero, ninguna de las dos
ramas existe en local, y la rama de `origin` tiene historia distinta. La rama `else` de
`ramas()` sigue generando `git branch develop HEAD`, y `HEAD` no tiene relacion con `origin/main`.

Reproduccion (binario `target/debug/tc2.exe` del commit 4a25582, git real, remoto bare local):
- Remoto con `main` (un commit `inicial`). Repo local `git init -b master` con un commit
  `historia local` y `git remote add origin <bare>`.
- `tc2 init --simular` da `rama refs/heads/main ... decision=persona motivo=en-el-remoto` y
  `rama refs/heads/develop ... decision=persona motivo=con-historia orden=git branch develop HEAD
  && git push origin develop`.
- Se ejecuta esa orden tal cual en el repo de prueba: `git merge-base origin/main develop` devuelve
  vacio, es decir, `develop` publicada no tiene ancestro comun con `origin/main`.

`init` no publica por si mismo en esta rama (la linea es `persona`), asi que el dano llega cuando la
persona ejecuta la orden. Aun asi es el mismo defecto que IMP-1 (historia sin relacion en `origin`),
y el comentario de `ramas()` dice lo contrario de lo que imprime. Arreglo: tratar igual que el caso
sin commits — si `origin` tiene `main` o `develop`, la rama que falta sale de `origin/<rama>` con
`git fetch` (quitar la condicion `!hay_commits` del brazo `(None, Some(b))`), y solo se usa `HEAD`
si no hay ninguna rama de `origin` de la que partir. Anadir un test con repo local con commits y
remoto con `main` no relacionado.

### MEN-5 (nuevo, introducido por la correccion) — `comillas()` no protege en PowerShell con `'` en la ruta

`comillas()` escapa `'` como `'\''`, que es la forma POSIX. Las ordenes de `init` se imprimen para
que la persona las pegue, y en esta maquina la shell principal es PowerShell; el propio comentario
admite la excepcion (`salvo '`), pero no lo que pasa en ese caso. Con una ruta que contiene `'` y
`;`, PowerShell trocea la orden en sentencias separadas y ejecutaria la parte que cae fuera de
comillas. Comprobado con el parser de PowerShell (solo analisis, nada ejecutado):
- `git worktree move 'a'\''; calc; '\''b' tareas` → 4 sentencias (`calc` queda suelta), con errores de
  parseo que en una pegada interactiva impiden la ejecucion de la linea, pero no todas las rutas
  hostiles dan error (`x'; echo PWNED` → 2 sentencias, 1 error).
- Con POSIX `sh`/bash la misma orden es correcta: `set -- 'a'\''; echo PWNED; '\''b' tareas` da
  el argumento `a'; echo PWNED; 'b` y no ejecuta nada.
Riesgo bajo: la ruta sale de `git worktree list` del propio repo de la persona. Arreglo: entrecomillar
para el shell de la persona (doble comilla con `` ` `` o `$` escapados para PowerShell, o no imprimir
la orden si la ruta contiene `'`), y corregir el comentario de la funcion.

### Verificacion de los hallazgos de la ronda 1 (no incluidos en la tabla)

- **MEN-1** (`comillas()` en `git worktree move`): corregido para POSIX; el residuo de PowerShell es
  MEN-5. Test `montada_en_otra_ruta_da_la_orden_entrecomillada` en verde.
- **MEN-3** (`_nota` en la propuesta): corregido. `ruleset_github` anade `_nota` solo si hay checks
  (`develop`), y el `tareas` no lo lleva. Tests `puertas` y `init` en verde.
- **MEN-4** (`motivo_de_deteccion`): corregido. Toma la ultima parte de la clave del catalogo
  (`PLATAFORMA_CONTRADICE` → `plataforma-contradice`). La clave es una constante del catalogo, no lleva
  datos de la persona ni el token. Test `plataforma_rechazada_conserva_el_motivo` en verde.

### Notas (no abren hallazgo)

- `misma_linea` trata `/tareas` y `/tareas/` como la misma regla. En git, `/tareas/` solo casa
  directorios; para `tareas` (un worktree) no cambia nada, pero en otro patron podria no anadir una
  regla que falta.
- Con historia solo en una rama distinta de `main`/`develop` en `origin` (p. ej. `feature/x`), la
  orden sugerida crea `main` desde esa rama. Es coherente con lo que dice el comentario, pero conviene
  saberlo.

## Alcance

- Ficheros revisados (diff 2c3706b..4a25582, solo los que casan con los patrones): `crates/tc2/src/init/aplicar.rs`,
  `crates/tc2/src/init/inventario.rs`, `crates/tc2/src/init/mod.rs`, `crates/tc2/src/plataforma/puertas.rs`,
  `crates/tc2/tests/init.rs`.
- Contexto adicional pedido: ninguno. Los ADR y ficheros de `docs/` del diff no son Rust y no los revise.
- Limite JS-WASM: sin cambios. El crate no tiene objetivo `wasm32` en esta rama y el diff no toca
  `#[wasm_bindgen]` ni tipos que crucen el limite. No aplica pedir llamador ni `.d.ts`.
- Produccion nueva sin `unwrap`, `expect`, `panic!`, `unreachable!` ni `unsafe`: comprobado con grep sobre
  las lineas anadidas del diff `2c3706b..4a25582` en `crates/`; sin coincidencias. Los `unwrap` de
  `tests/init.rs` son de test.

## Reproduccion

- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-028-r2-rust`, rama
  `feature/task-028-f3-1-taskctl-init-setup` (HEAD `ed7869f`). Entre `4a25582` y `ed7869f` solo cambian
  ficheros de `tareas/`, asi que el codigo revisado es el de `4a25582`. Clon borrado al terminar.
- Suite `cargo test -p tc2` (sin `--locked`, como pide la peticion): codigo de salida 0, 24 binarios
  con `ok` y 0 fallos. `init.rs`: 17 de 17 (12 de la ronda 1 mas 5 nuevos). `plataforma.rs`: 4 de 4 en 43 s,
  sin cuelgue. Sin `LNK1104`, no fue necesario repetir ningun test.
- Formato y lints: `cargo clippy --all-targets --locked -- -D warnings`: codigo de salida 0, sin avisos.
  `cargo fmt --check` no se ejecuto en esta ronda.
- Build para `wasm32`: no aplica (el crate no tiene ese objetivo en esta rama).
- Casos construidos: IMP-1 original (repo sin commits, remoto con `main`) cubierto por el test nuevo
  y en verde; IMP-1 residual (repo con commits propios, remoto con `main` no relacionado) reproducido con
  el binario y la orden ejecutada, `merge-base` vacio. MEN-5: parseo con `System.Management.Automation.Language.Parser`
  de PowerShell y comprobacion POSIX con bash (solo analisis, sin ejecutar las ordenes hostiles).
- Mutacion: no se hizo en esta ronda (la verificacion se apoya en los tests nuevos y en el caso reproducido).
- Medidas: no aplican.

## Incidencias del revisor

- Un `cd`/variable mal formados (rtk interpreta `VAR=... && ...` como comando) hicieron que varios
  comandos de solo lectura (`git diff --stat`, `git diff ... | grep`, `git merge-base`/`git log`) se
  ejecutaran con el cwd del repo original y no del clon. Un comando con `git fetch`/`git branch`/`git push`
  de la misma linea no llego a ejecutarse porque la cadena `&&` se corto en el `cd`. Comprobado despues:
  el repo original sigue en `feature/task-028-f3-1-taskctl-init-setup`, HEAD `ed7869f`, sin cambios de
  git hechos por el revisor. El `informe-revision-2-code-quality-reviewer.md` modificado en el repo
  original no lo toque yo.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/init/inventario.rs` completo (ramas, plan, personas) en el clon: para verificar IMP-1 y MEN-1.
- `crates/tc2/tests/init.rs` (diff de la ronda): para ver que tests cubren cada correccion.
- `crates/tc2/src/plataforma/deteccion.rs` (`detectar`) e `i18n/mod.rs` (`Mensaje::clave`), y
  `i18n/catalogo/plataforma.rs`: para confirmar que `motivo_de_deteccion` solo toma una clave constante.
- `tareas/03-en-revision/TASK-028/revision/informe-revision-1-rust-wasm-reviewer.md` y la peticion: el enunciado de los hallazgos.
