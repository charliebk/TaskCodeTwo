# Informe de revision — TASK-116 (ronda 2)

- Commit revisado: a206230 (HEAD de la rama; el codigo es el de 727d2c5, el ultimo commit solo anade la peticion)
- Revisor: rust-wasm-reviewer
- Veredicto: PENDIENTE

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | corregido | docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md:7 | implementacion |
| MEN-2 | MENOR | corregido | docs/adr/F1-16-lotes-desatendidos.md:9 | implementacion |
| MEN-3 | MENOR | corregido | docs/adr/F1-11-extensiones-declaradas.md:8 | implementacion |

Verificacion de cada hallazgo:

- **MEN-1 (F1-1)**: corregido en 973642c. La linea `- Adenda (2026-11-16, TASK-116): ...` esta en la cabecera (linea 7 de F1-1), con el enlace `F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella`. El ancla coincide con el titulo `#### Adenda de TASK-116 (2026-11-16): todos los nombres de la huella`. F1-1 conserva 16 apariciones del nombre viejo, todas cubiertas por esa adenda. El indice `docs/adr/README.md` tambien lleva el nombre nuevo y el enlace.
- **MEN-2 (F1-16)**: corregido. F1-16 tiene la linea de adenda en su cabecera y conserva 3 apariciones de nombres viejos, cubiertas por ella.
- **MEN-3 (F1-5, F1-6, F1-8, F1-9, F1-7, F1-10, F1-11)**: corregido. Todos llevan linea de adenda. Cada ADR con nombre viejo queda cubierto:
  - F1-1 (16), F1-2 (14), F1-3 (28), F1-4 (15), F1-5 (13), F1-6 (9), F1-7 (1), F1-8 (4), F1-9 (1), F1-10 (1), F1-11 (9), F1-16 (3): con linea de adenda.
  - F1-12 (22): es la propia adenda.
  - F1-13 (1, linea 451): explica el cambio (`taskcode` y `taskcode/tareas` pasan a `taskcodetwo/tareas`); no es un uso vigente del nombre viejo.
  - F1-19 (3): sin adenda, descartado por la instruccion de la ronda.
  - F1-14, F1-15, F1-17 y F1-18: sin apariciones del nombre viejo.
  Busqueda: `taskcode/tareas|taskcode-estado|\.git/tc2|refs/taskcode/|\.taskcode/worktrees|taskcode-estado-tmp` en `docs/`.
- **Codigo de 76be757** (`crates/tc2/tests/convivencia_016.rs`): el test de convivencia por el camino sin red es correcto. Tras `tc2 new` y `crear_rama_y_worktree`, la URL del remoto se cambia a `http://127.0.0.1:1/x.git`, `estado::transaccion` con `Anotar` cae en `sin_red` (`crates/tc2/src/estado/transaccion.rs:284-285` y `:477-536`), que no exige red y deja la pendiente en `refs/taskcodetwo/pendientes/000001`. El test comprueba la linea de salida exacta, la huella de la 0.16 intacta, que toda ref nueva tiene nombre de la 2.0 y que la raiz del proyecto solo gana la carpeta de worktrees. Restaurar la URL antes de comprobar las refs es correcto. Mutacion: no ejecutada en esta ronda (la correccion es de test, no de codigo de produccion; la ronda 1 ya midio el poder del test con la mutacion de `huella.rs`).
- **Codigo de 75bb6b2** (`crates/tc2/src/git/puerto.rs:514`): solo anade el comentario `// intencionado: caso de rechazo del nombre viejo` dentro de la tupla del caso de prueba. No cambia la logica. Compila, pasa `cargo fmt --check` y clippy.

Nota sin severidad (no es hallazgo): `puerto.rs` ya tenia el comentario `// El prefijo viejo ya no es propio (TASK-116).` en la linea 511, y el nuevo de la linea 514 repite la idea. No altera nada.

## Alcance
- Ficheros revisados: `crates/tc2/tests/convivencia_016.rs` (76be757), `crates/tc2/src/git/puerto.rs` (75bb6b2, lineas 490-532), y `crates/tc2/src/estado/transaccion.rs` (lectura para seguir el camino sin red del test). Los ADR de `docs/adr/` se revisan solo en lo que cambia la correccion (cabeceras y nombres viejos).
- Contexto adicional pedido: ninguno (el diff de Rust es pequeno y el test lleva sus propias dependencias).
- Limite JS-WASM: sin cambios de firma exportada con `#[wasm_bindgen]` ni de tipos que crucen el limite (crate nativo). No hay llamador TypeScript que revisar.

## Lecturas fuera del contexto permitido
- `crates/tc2/src/estado/transaccion.rs`: para comprobar que el camino sin red del test (`Anotar` sin red) produce la linea `estado sinred operacion=anotar pendiente=si pendientes=1` que el test espera. Motivo: el test depende de ese comportamiento y su correccion no se puede juzgar sin el.
- `docs/adr/F1-12-actualizacion-entre-versiones.md` (seccion de la adenda) y `docs/adr/F1-13-adaptador-de-plataforma.md` (busqueda del nombre viejo): la peticion ya los lista (F1-12 esta en el alcance); F1-13 se miro para confirmar que su unica aparicion explica el cambio.
- `git show` de los commits: el hook de la sesion reescribio la llamada y no se pudo ejecutar; el codigo de cada commit se leyo en el clon en su estado de HEAD (mismo codigo que 727d2c5).

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-116-r2-rust`, rama `fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1`, HEAD a206230. Se borra al terminar.
- Build: compilado en nativo como parte de `cargo test` y `cargo clippy`. No se compilo a `wasm32`: el diff no toca nada del limite JS-WASM.
- Formato y lints: `cargo fmt --all -- --check` limpio (exit 0). `cargo clippy --all-targets --locked -- -D warnings`: sin avisos (exit 0).
- Suite: `cargo test -p tc2 --no-fail-fast`, una sola ejecucion, resultado `166 passed, 5 ignored` en 22 suites, exit 0. Ninguna caida por red local en esta ronda. La linea base de la ronda 1 (4 tests de `tests/plataforma.rs` inestables) no se repitio en esta ejecucion.
- Casos construidos: busqueda de nombres viejos en `docs/` con el patron de la seccion de hallazgos (140 apariciones en 15 ficheros, todas cubiertas por la adenda o por F1-13/F1-19 segun el caso). En el test de convivencia, el caso sin red se construye apuntando el remoto a `127.0.0.1:1` (conexion rechazada, no colgada, en esta maquina).
- Mutacion: no ejecutada en esta ronda (ver la nota de 76be757 arriba).
- Medidas: no aplican.

## Revisado sin hallazgos
- Correccion de las cabeceras de F1-1, F1-2, F1-3, F1-4, F1-5, F1-6, F1-7, F1-8, F1-9, F1-10, F1-11 y F1-16: cada una tiene su linea de adenda, con el ancla correcta.
- `crates/tc2/tests/convivencia_016.rs` (76be757): el test de camino sin red, su orden de operaciones y sus aserciones.
- `crates/tc2/src/git/puerto.rs` (75bb6b2): solo comentario; sin cambio de comportamiento.
- Formato y clippy de todo el workspace: sin avisos.
