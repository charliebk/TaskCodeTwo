# Informe de revision — TASK-148d (ronda 1)

- Commit revisado: 46a793f57e41fda34bb846be3286691a227d2b40
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada

Veredicto propuesto: **aprobada**. Sin hallazgos; todos los mutantes aplicados ponen algun test en rojo.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

sin hallazgos.

## Regla 4D

- Duplicacion: cumple. `git grep -e read_dir -e "fn instantanea" -e "fn recorrer" -e pila` sobre los seis ficheros y `soporte.rs` solo da los dos tests nuevos de `soporte.rs`; ningun recorrido con otro nombre (`pila`, `recorrer`, `read_dir`).
- Desviacion: cumple. Solo cambian tests y el soporte de tests; ninguna firma publica de produccion.
- Diluvio: cumple. Una funcion (`instantanea`) y un auxiliar privado; sin traits ni structs.
- Disfraz: cumple. Cada `foto` migrada conserva su parte de git (status con ignorados, for-each-ref, rev-parse del arbol, refs y commits del bare, worktrees, index/config/exclude en doctor) y la parte de carpeta pasa a ser mas fuerte (recursiva, con carpetas, hash). Los mutantes de produccion la ponen en rojo (ver Mutacion).

## Lecturas fuera del contexto permitido

- `crates/tc2/src/tablero.rs`, `doctor/mod.rs`, `importar/mod.rs`, `init/mod.rs`, `desinstalar/mod.rs`: leidos solo para colocar los mutantes de produccion (el paso de mutacion de la skill).
- `tareas/03-en-revision/TASK-148d/tarea.md` y `task-workflow/revision.md`: contexto de la tarea y del flujo.

## Alcance
- Ficheros revisados: `crates/tc2/tests/comun/instantanea.rs`, `comun/mod.rs`, `desinstalar.rs`, `doctor.rs`, `frescura.rs`, `importar.rs`, `init.rs`, `soporte.rs`, `tablero.rs` (solo tests).
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios; no hay firmas `#[wasm_bindgen]` en el diff.

## Reproduccion
- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-148d-r1`, en 46a793f, con `CARGO_TARGET_DIR` propio dentro (borrados al terminar).
- Build: los tests compilan con `--locked` (nativo; no aplica wasm32).
- Formato y lints: `cargo fmt --check` limpio; `cargo clippy --all-targets --locked -p tc2 -- -D warnings` limpio.
- Suite: la completa no se repite (linea base de la peticion: antes 777 ok / 13 ignorados; despues 48 binarios, 779 ok, 0 fallos, 13 ignorados). Ejecutados uno a uno: soporte 12 ok, init 25 ok, desinstalar 19 ok, importar 9 ok, tablero 6 ok, frescura 12 ok, doctor 12 ok.
- Casos construidos: mutantes de produccion que crean una carpeta vacia `zz/y` (invisible para las fotos antiguas).
- Mutacion (todos revertidos; el arbol del clon quedo limpio):
  - `instantanea.rs`: sin carpetas, sin recursion, sin exclusion, hash sustituido por longitud, exclusion solo en la raiz: los cinco ponen en rojo `soporte` (instantanea_*).
  - `tablero_en` crea `.git/taskcodetwo/zz/y`: rojo `tablero::board_y_status_no_escriben_nada` y `frescura::c4_las_cuatro_solo_escriben_la_ref_remota_y_la_marca`.
  - `doctor::comprobar` hace lo mismo: rojo `doctor::doctor_no_escribe_nada`.
  - `importar_en` crea `.git/zz/y`: rojo `importar::c6_simular_no_cambia_ni_un_byte`.
  - `desinstalar::plan` igual: rojo `simular_no_toca_nada`, `worktree_de_tarea_sin_publicar_rechaza_sin_tocar_nada`, `worktree_bloqueado_o_carpeta_actual_dentro_rechaza`.
  - `init::init` igual: rojo `simular_no_toca_nada_y_coincide`, `init_en_repo_016_rechaza`, `init_con_config_invalida_no_la_pisa`, `sin_red_no_escribe`. (Un primer mutante en `init::plan` sobrevivio porque los tests de init no pasan por esa funcion, no por falta de red: `init()` llama a `inventario::plan` directamente.)
- Medidas: no aplican.

## Revisado sin hallazgos
- `instantanea`: claves relativas con `/`, carpetas `dir`, ficheros `<len> <sha256>`, exclusion por nombre a cualquier profundidad, carpeta inexistente da mapa vacio, fichero ilegible hace panic con la ruta.
- Equivalencia con las fotos antiguas: tablero y doctor (ya recursivas) ganan carpetas vacias; frescura (antes plana, con `.ok()`) gana recursion y falla ante un fichero ilegible; init (antes contenido completo) y desinstalar pasan a hash, equivalente para comparar igualdad; importar gana carpetas.
- `desinstalar.rs`: el `read_dir` de `wt_reg` pasa a `instantanea(..).is_empty()`, con la misma semantica (vacia o inexistente).
- Observacion no clasificable como hallazgo: `p.is_dir()` sigue enlaces simbolicos, de modo que un enlace a un ancestro recursaria sin fin; las fixtures no crean enlaces. Cosmetico: en tablero, frescura y doctor la cadena `"{:?}\n"` se escribio con salto de linea literal en vez de `\n`, con el mismo resultado.
