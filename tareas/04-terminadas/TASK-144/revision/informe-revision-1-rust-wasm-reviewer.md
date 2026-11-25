# Informe de revision — TASK-144 (ronda 1)

- Commit revisado: 67f6db4fc47f3b152df82d2002f75c14b5f9669f
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada

Veredicto propuesto: **aprobada**. Sin hallazgos CRITICO, IMPORTANTE ni MENOR.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

sin hallazgos

## Regla 4D

- Duplicacion: cumple. No quedan copias del juego de tareas de `next` (una sola fuente en `comun::tareas::juego_de_next`), de `publicar_estado` (`Fixture::publicar_estado`, unico `init -b estado` de los tests), de `Poner`/`fila`/`terminada`/`CABECERA`/`BASE` (solo en `comun`) ni de `linea_esperada` (ahora llama a `ids_de_la_linea`). Quedan envoltorios locales de una o cinco lineas (`terminada` de estimacion, `seis()` de progreso) que delegan en el comun y fijan solo lo propio.
- Desviacion: cumple. El diff de `crates/tc2/src` es solo `grafo.rs` y `progreso/mod.rs`, ambos dentro de `#[cfg(test)] mod tests`; no cambia codigo de produccion ni la API publica (no hay `wasm_bindgen` en el diff).
- Diluvio: cumple. Solo se anaden una funcion por concepto (`Poner`, `poner`, `publicar_estado`, `tarea`, `terminada`, `fila`, `juego_de_next`); ningun trait ni capa nuevos.
- Disfraz: cumple. `c7_fuera_de_un_repo` ejerce el respaldo (rojo con el mutante de la ficha y con el control, ver Mutacion); `linea_esperada` conserva las aserciones de longitud 54 y de `En marcha` no vacia.

## Alcance

- Ficheros revisados: los 14 de la peticion (`grafo.rs`, `progreso/mod.rs`, `tests/comun/{mod,ops,remoto,tareas}.rs`, `tests/{doctor,estimacion,progreso,recursos,roadmap,siguiente,solape,tablero}.rs`).
- Contexto adicional pedido: ninguno (se leyo ademas `roadmap/generar.rs:401` y `estimacion/estimar.rs:139`, ver abajo).
- Limite JS-WASM: sin cambios de firma exportada.

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-144-r1\repo`, rama `fix/task-144-auditoria-4d-a6-estimacion-progreso-road` (HEAD 1242ba5), con `CARGO_TARGET_DIR` propio.
- Build: `cargo clippy -p tc2 --all-targets --locked -- -D warnings` limpio (nativo; no hay objetivo wasm32 en este crate).
- Formato y lints: `cargo fmt --check` exit 0; clippy `-D warnings` exit 0.
- Suite: linea base de la peticion (800 antes, 800 ok / 0 fallos / 13 ignorados despues; no se repite entera). Ejecutado aqui: `--lib` 411 ok / 2 ignorados; `siguiente` 6 ok; `roadmap` 13 ok; `estimacion` 11 ok; `progreso` 12 ok; `solape` 15 ok (147 s); `tablero` 6 ok; `recursos` 12 ok. `doctor`: la suite completa tarda mucho en esta maquina cargada (varios tests por encima de 60 s en paralelo) y no la di por terminada; en la primera pasada pasaron 5 tests (entre ellos `capacidades_por_plan...`, `salida_acotada_y_json_con_todo`, `rama_de_estado_ausente_o_corrupta_falla`, `sin_git_falla_y_fuera_de_un_repo_rechaza`) y `develop_sin_proteger_falla` (que usa `con_estado` migrado) pasa solo en 33,6 s. No se vio ningun fallo; las demas quedaron sin concluir por tiempo, no por rojo.
- Casos construidos: texto generado de estimacion y progreso. Las aserciones de texto de `estimacion.rs` y `progreso.rs` no se tocaron en el diff (solo cambian los ayudantes); `tareas::terminada` produce el mismo `tarea.md` que las dos copias anteriores (criterios `- [x] a\n- [x] b` en estimacion y `- [x] a` en progreso, carril y complejidad aplicados antes de `plan_aprobado`, mismas filas), y las suites de ambos siguen en verde. En `siguiente`, los titulos pasan de `"t"` a `"Titulo de {id}"` y ninguna asercion mira titulos (cierto: la suite verde).
- Mutacion (todos con rojo salvo el equivalente):
  - A6-07 mutante de la ficha (`cwd.map(..)` -> `cwd.filter(|_| false).map(..)` en `orden_estado`): `c7_fuera_de_un_repo` en rojo en `progreso/mod.rs:513` («tc2 · sin tarea en curso» frente a la linea de TASK-001).
  - A6-07 control (`None => String::new()`): rojo en `progreso/mod.rs:501` (`""` frente a «tc2 · sin tarea en curso»).
  - `publicar_estado` sin el `push`: `siguiente` 3 de 6 en rojo (`next_json_valido`, `next_en_es_y_en`, `next_sin_red_usa_la_copia_del_clon`).
  - `juego_de_next` sin TASK-006: `siguiente` 3 de 6 en rojo (los mismos).
  - `terminada` pierde un criterio (de 2 a 1) o todos: `estimacion` y `progreso` siguen verdes. Es un mutante equivalente: `estimar::tamano` agrupa 0..=3 criterios en el mismo tamano y ninguna aserciondepende del numero de casillas; ya era asi antes del cambio (las copias llevaban el mismo contenido).
- Medidas: no aplican (solo tests).

## Revisado sin hallazgos

- `c7_fuera_de_un_repo`: comprueba con `assert!` que la carpeta temporal esta fuera de un repo (falla en voz alta, ya no se salta en silencio), cubre JSON valido, JSON invalido con `cwd` fuera de repo, entrada vacia, y el respaldo con un principal con progreso de TASK-001; pasa de `if` silencioso a `-> io::Result<()>` con `?`. Limpia la carpeta al final (solo si todo va bien, patron previo del fichero).
- `linea_esperada` -> `ids_de_la_linea(..).unwrap_or_default()`: la diferencia con la version anterior es que una linea ausente da `[]` y no `[""]`; no debilita nada porque el test fija `len == 54` en las listas y `En marcha` es no vacia en `esperado.txt` (`TASK-257`).
- `Fixture::publicar_estado`: mismas operaciones que las copias (init, escribir, `add -A`, commit, push, `rev-parse HEAD`); en `roadmap.rs` TASK-009 se anade al final del juego y el orden de escritura no importa (`add -A`); `doctor`, `recursos` y `tablero` conservan el orden de escritura y los ficheros extra; el `ROADMAP.md` del juego usa `estado::ROADMAP` (`"ROADMAP.md"`).
- `c5_sin_red_y_sin_origin_sin_claves` llama a `start_de_luis` (mismo entorno: `dir = e.b`, quien luis 1002, `sin_catalogo`) y recupera el registro del puerto de git del mismo `g`.
- A6-05 (771e187, `estado::prueba::EnMemoria`) ya viene de TASK-148e y no esta en el diff de la rama.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/roadmap/generar.rs` (`ids_de_la_linea`): para comparar su comportamiento con la version anterior de `linea_esperada`.
- `crates/tc2/src/estimacion/estimar.rs` (`tamano`): para decidir si el mutante de criterios es equivalente.
- `crates/tc2/src/estado/mod.rs` (`ROADMAP`): para confirmar el nombre del fichero del juego.
