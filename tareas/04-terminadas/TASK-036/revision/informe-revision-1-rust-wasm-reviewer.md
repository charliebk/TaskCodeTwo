# Informe de revision — TASK-036 (ronda 1)

- Commit revisado: 6ecf89825e4a41e13abdd7f70a2fa08bcbf65ff6
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/plan/operacion.rs:290 | plan |
| MEN-2 | MENOR | abierto | crates/tc2/src/plan/mod.rs:115 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — `plan` no rechaza una tarea con `plan_aprobado: true`, pero `plan guardar` si

- Donde: crates/tc2/src/plan/operacion.rs (`Planificar::aplicar`, tras la comprobacion de pausa) frente a crates/tc2/src/plan/guardar.rs (`Guardar::aplicar`, rechazo `plan-aprobado`).
- Que pasa: `Liberar` deja `plan_aprobado` tal cual (liberar.rs: "`rama` y `plan_aprobado`, igual"), asi que una tarea liberada de `en-curso` queda `planificada` con `plan_aprobado: true`. `tc2 plan` no mira ese campo: la pasa a `en-diseno`, abre una ronda nueva de peticiones y anade la fila `plan`. Luego `plan guardar` la rechaza con `plan-aprobado` para toda salida y para `plan-final.md`. La tarea queda en `en-diseno` con un brainstorm que nadie puede guardar.
- Reproduccion: por lectura del codigo y de liberar.rs (no hay en este commit una orden de aprobacion que lo ejecute de punta a punta; no construido como test). Caso: tarea `en-curso` con `plan_aprobado: true` -> `tc2 liberar` -> `tc2 plan TASK-NNN` (ok) -> `tc2 plan guardar TASK-NNN plan-final.md f` (rechazada `plan-aprobado`).
- Impacto: inconsistencia entre las dos ordenes en un camino alcanzable pero que exige liberar una tarea ya aprobada; sin perdida de datos.
- Sugerencia: rechazar tambien en `plan` con `plan-aprobado` (mismo texto `PLAN_APROBADO`), o documentar que re-planificar una tarea aprobada es intencionado y relajar `guardar`.

### MEN-2 — El mensaje de `plan guardar` imprime la ruta del fichero sin sanear

- Donde: crates/tc2/src/plan/mod.rs:115 (`leer_fichero`, cierre `mal`: `ruta.display().to_string()`), junto con `e.to_string()` del error de E/S en `NO_SE_LEE`.
- Que pasa: `destino` pasa por `plataforma::errores::sanear` (quita controles y corta a 200), pero la ruta del fichero y el texto del error de E/S entran tal cual en el diagnostico. Una ruta con saltos de linea (valida en Linux) rompe la garantia de una sola linea de diagnostico que `plan_mal_llamado_es_uso` comprueba solo para casos sin controles.
- Reproduccion: por lectura; ningun test cubre rutas con `\n`.
- Impacto: cosmetico, solo stderr propio; la ruta la escribe quien lanza la orden.
- Sugerencia: pasar `ruta` por `sanear` como `destino`.

## Revision de lo pedido

- Lectura del fichero externo (`leer_fichero`): limite con `take(MAX_BYTES + 1)` sobre el handle, asi que un fichero enorme no se carga entero; 1 MiB exacto pasa y 1 MiB + 1 da `grande`; UTF-8 invalido da `no-utf8`; vacio o solo blancos da `vacio`; directorio o inexistente da `no-se-lee` (uso, 2). Sin `unwrap`/`expect`/indexado sobre entrada fuera de tests (`u64::try_from(..).unwrap_or`, `get(..10).unwrap_or_default`). La ruta relativa se resuelve contra `e.dir`; una absoluta sustituye a la base (comportamiento de `Path::join`). Observado y no reportado: un BOM UTF-8 se conserva y un `\r` suelto no se normaliza (solo `\r\n`); no es comportamiento incorrecto en un caso real. No comprobar que el fichero este fuera del repo (divergencia declarada) es aceptable: solo se lee.
- Destinos: `destino_valido` es una lista cerrada (`plan-final.md` o `brainstorm/salida-<rol>-<n>.md`); `..`, `./`, `\`, subcarpetas, `.bak`, ceros a la izquierda, `n` que desborda u32, rol `unificador` o desconocido y `peticion-*` dan `None` (14 casos de `destinos_de_la_lista`). La ruta final se construye solo con el destino ya validado: no hay escape de `planificacion/`. Un rol conocido sin peticion da `sin-peticion`.
- Divergencias declaradas: tope de hotfix antes de `politica_carriles` (coherente con que la configuracion solo sube; probado, hotfix 4 pasa a 1); con `roles_diseno` 5 el quinto es rendimiento sin etiqueta (documentado y probado); rondas solo se anaden (`ronda_mayor` compara peticiones esperadas con existentes; `-1.md` no confunde `-11.md`); sin catalogo para Planificar/Guardar porque `exige_red` es true y nunca queda pendiente. Las cuatro son razonables y estan documentadas en el codigo.
- `unsafe`: ninguno. Limite JS-WASM: sin cambios (no hay `#[wasm_bindgen]` en el diff).

## Lecturas fuera del contexto permitido

- crates/tc2/src/ciclo/liberar.rs (cabecera) y crates/tc2/src/plataforma/errores.rs (`sanear`): para MEN-1 y MEN-2, confirmar que `liberar` conserva `plan_aprobado` y como funciona `sanear`.

## Alcance
- Ficheros revisados: carril.rs, config/mod.rs, i18n/catalogo/{cli,mod,plan}.rs, lib.rs, plan/{mod,guardar,heuristica,operacion,plantillas,roles}.rs, tests/plan.rs (plantillas y tests de integracion solo en lo necesario).
- Contexto adicional pedido: ninguno salvo lo anotado arriba.
- Limite JS-WASM: sin cambios de firma exportada.

## Reproduccion
- Clon: `%LOCALAPPDATA%/pruebas-claude/TaskCodeTwo/task-036-r1-rust` en 6ecf898 (borrado al terminar).
- Suite: `cargo xtask verificar --perfil rapida` en verde (exit=0) sobre dbe7b5f (codigo identico: `git diff --stat dbe7b5f 6ecf898 -- crates docs` vacio), log `verif-036.log`; no repetida. En el clon con `--test-threads=4`: `cargo test -p tc2 --lib plan` 19 ok; `--lib carril` 3 ok; `--test plan` 5 ok; `cargo clippy -p tc2 --all-targets` sin avisos; `cargo fmt --check` ok.
- Mutacion (cada una revertida): Fix 1 a 2 (rojo: `roles_de_diseno_por_carril`, `rondas_y_carril`, `uno_en_fix_y_todos_en_riesgo_alto`); RiesgoAlto suelo a 3 (rojo, los mismos tres); quitar el filtro de ceros a la izquierda en destinos (rojo: `destinos_de_la_lista`); `MAX_CRITERIOS + 1` en la validacion (rojo: `enunciado_y_validacion`); quitar la comprobacion de rol conocido (rojo: `destinos_de_la_lista`). Todas las protecciones pedidas tienen red.
- Medidas: no aplican.

## Revisado sin hallazgos
Heuristica de complejidad y cortes, seleccion de roles, `Carril::roles_diseno` (tabla de 16 casos), transicion de estado y fila `plan`, idempotencia (`sin-cambios`), CRLF a LF en `Guardar::nueva`, catalogo es/en.
