# Informe de revision — TASK-145 (ronda 1)

- Commit revisado: fce8bd62afb7e421e03fef98b93e6c8ff949cadf
- Revisor: general-purpose (sonnet) con rust-wasm-reviewer
- Veredicto: aprobada con correcciones

Veredicto propuesto: **aprobada** (sin CRITICO ni IMPORTANTE; 3 MENOR que se pueden anotar sin corregir o tratar en una tarea posterior). El unico diff de comportamiento observable es el aceptado en el `## Resultado`, y los mutantes relevantes se matan.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | crates/tc2/src/i18n/guarda.rs:82 | implementacion |
| MEN-2 | MENOR | abierto | crates/tc2/src/init/aplicar.rs:34 | implementacion |
| MEN-3 | MENOR | abierto | crates/tc2/src/desinstalar/mod.rs:528 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### MEN-1 — La guarda de i18n se salta con el ayudante nuevo `plan_dato::externo("prosa")`

`ningun_literal_fuera_del_catalogo` cuenta `Mensaje::externo(`, la entrada `EXTERNO` y `Err("... ...")`, pero la tarea introduce `plan_dato::externo(e: impl Display)` (plan_dato.rs:56), que mete prosa en un `Mensaje` externo, y ninguna comprobacion mira sus llamadas.

Reproduccion (clon propio, `cargo test --lib guarda`):
- Mutante: en `desinstalar/mod.rs::reescribir`, `return Err(externo("cambio desde el plan"));` en vez de la entrada del catalogo. Resultado: `6 passed` (la guarda sigue verde). Es exactamente el defecto de A7-07 (texto en espanol dentro de `{error}`), reintroducido por el camino nuevo.
- Con `Mensaje::nuevo(&EXTERNO)...` (la forma de la ficha A7-10) y con `Mensaje::externo(format!(..))`, la guarda se pone roja como se pide.
- Huecos accesorios de la misma guarda: `Err(` con el literal en la linea siguiente (rustfmt lo parte si es largo), `Err(format!("..."))` y `.texto(Idioma::Es)`/`diagnostico_en(Idioma::Es)` en codigo de produccion no se miran (hay usos fuera de A7, p. ej. `config/validar.rs:416`, `importar/mod.rs:475`). La ficha pedia la forma `Err("...".to_owned())`, que si cubre; por eso es MENOR y no IMPORTANTE.
- Sugerencia: contar tambien `externo("` (llamada con literal) y prohibir `externo(format!` fuera del catalogo.

### MEN-2 — El criterio «ningun texto se renderiza en espanol en el sitio» no tiene red en los sitios de git de `init`

`init/aplicar.rs::aplicar` pasa los fallos de `excluir`, `ramas_raiz` y `MontarEstado` con `i.mensaje()`. Solo `escribir_nuevo` (test `escribir_nuevo_sobre_uno_existente_en_ingles`) y `Informe::mensaje` (test `informe_como_mensaje`) tienen test en ingles; los sitios que llaman a `i.mensaje()` no.

Reproduccion: mutante en `init/aplicar.rs:34`, `.map_err(|i| externo(i.diagnostico_en(Idioma::Es).unwrap_or_else(|| i.linea())))` (la forma original `texto(&i)` de A7-07). `cargo test --lib init` -> 7 ok y `cargo test --test init` -> 25 ok: **sobrevive**. La guarda tampoco lo caza (ver MEN-1). Es el mismo riesgo: una regresion de A7-07 en un sitio concreto pasa la suite. No hay un test de extremo a extremo con `TC2_IDIOMA=en` que fuerce un fallo de git en `init` ni en `desinstalar` (los de `tests/init.rs:808` y `tests/desinstalar.rs:951` miran otros textos).

### MEN-3 — Desviacion declarada: la ruta sale dos veces en el fallo de `desinstalar` al reescribir un fichero cambiado

Confirmada: `CAMBIO_DESDE_EL_PLAN` lleva `{ruta}` y `FALLO_AL_APLICAR` ya la lleva, asi que sale «no se pudo quitar X: X cambio despues de hacer el plan. ...» (el test `cambio_desde_el_plan_en_ingles` fija el texto en ingles con la ruta repetida). No pierde informacion, no cambia el stdout, los motivos ni los codigos y `init` ya tenia esa forma antes de la tarea (`escribir_nuevo`); es redundancia de redaccion y la unificacion del catalogo la compensa (una entrada en vez de dos). Clasificacion: MENOR, aceptable tal cual; si se quiere evitar, una variante de la entrada sin `{ruta}` para el hueco `{error}`.

## Regla 4D

- Duplicacion: cumple. No hay copias del bucle de aplicar (un solo `plan_dato::aplicar_en_orden`/`rechazar` para `init` y `desinstalar`), de `leer`/`no_se_lee` (un solo `lectura::opcional`/`no_se_lee`; la `leer`/`no_se_lee` de `init` y `desinstalar` desaparecen), de `NO_SE_LEE` (una entrada en `catalogo/comun.rs`, quitada de `init` y `desinstalar`), de `t()` (una en `plan_dato`; `init` y `desinstalar` la importan) ni de los ayudantes de `ciclo` (`ciclo::prueba` unico; `liberar.rs` y `operacion.rs` ya no tienen `AHORA`, `cuenta`, `ana`, `fila` propios, y `repo_con_remoto` sustituye a los dos montajes git). `estado` y `confiar` comparten `Calculo`. Los candidatos 4D de la peticion son nombres coincidentes (`aplicable`/`aplazar`/`ruta` son los metodos del `impl Paso` de cada orden, no copias) y los `Repo`/`Calculo` homonimos son tipos locales distintos.
- Desviacion: cumple en lo funcional. La API publica no cambia: ningun `pub` anadido ni quitado en `crates/tc2/src` (`git diff origin/develop..HEAD | grep` de `pub fn/struct/enum/const/mod/trait` vacio; `plan_dato` y `lectura` son `mod` privados, `ciclo::prueba` es `cfg(test)`, `Informe::mensaje` es `pub(crate)`). Los unicos cambios de comportamiento son los aceptados (fixture `descartada`, stderr en ingles sin espanol, `config.yml` ilegible -> `config error` codigo 1 sin romper el hook); la redaccion duplicada es MEN-3.
- Diluvio: cumple. Un trait de tres metodos y dos funciones para dos ordenes; `Calculo` con un `bool` de cache en vez de dos structs.
- Disfraz: cumple en el codigo. En tests, la guarda ampliada y los tests de `Informe::mensaje` son reales (mutantes rojos); queda la red incompleta de MEN-1 y MEN-2.

## Alcance

- Ficheros revisados: los 23 de la peticion (ciclo/{liberar,mod,operacion,prueba}.rs, confianza/mod.rs, config/mod.rs, contrato.rs, desinstalar/{maquina,mod,proyecto}.rs, estado/lectura.rs, i18n/catalogo/{comun,desinstalar,init}.rs, i18n/guarda.rs, init/{aplicar,inventario,mod}.rs, lectura.rs, lib.rs, plan_dato.rs, tests/init.rs, tests/verificar.rs).
- Contexto adicional pedido: `confianza/mod.rs` (`leer_almacen`, `confiar_en`), `verificacion/ejecutar.rs` (`decidir_y_lanzar`), `tablero.rs::carpeta_actual`, `i18n/guarda.rs` completo, y `tareas/03-en-revision/TASK-145/tarea.md` (`## Resultado`). Todo para confirmar los llamadores de `del_arbol` y que los cambios de `confianza` no alteran el almacen.
- Limite JS-WASM: no aplica (sin `#[wasm_bindgen]` en el diff).

## Reproduccion

- Clon: `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-145-r1\repo`, rama `fix/task-145-auditoria-4d-a7-config-init-desinstalar` en 795a01b (el codigo es el de fce8bd6; el commit posterior solo anade la peticion). `CARGO_TARGET_DIR` dentro de la carpeta. Sin tocar el repo original.
- Build: `cargo test`/`clippy` con `--locked`, nativo (el diff no toca nada de `wasm32`).
- Formato y lints: `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -p tc2 -- -D warnings` exit 0.
- Suite: linea base de la peticion (808 ok, 0 fallos, 13 ignorados, 48 binarios; antes 800), no repetida. Ejecutado por mi parte: `--lib` (418 ok, 0 fallos, 2 ignorados), `--test verificar` bajo mutante, `--test desinstalar` (19 ok, con el mutante equivalente), `--test init` (25 ok, con el mutante MEN-2), `--lib` de `config`, `ciclo`, `confianza`, `init` y `guarda`.
- Casos construidos: `config.yml` en UTF-16LE con BOM (el del test de la tarea) para `del_arbol`, `verificar`, `confiar` y `--hook`; `develop` sin publicar en `origin` (test A7-11 de `tests/init.rs`); fichero cambiado entre el plan y su aplicacion en `escribir_nuevo` y `reescribir`.
- Mutacion (rojo = mata):
  - `aplicar_en_orden` sin cortar en el primer fallo -> rojo (`plan_dato::corta_en_el_primer_fallo`).
  - `aplicar_en_orden` con motivo `error` -> `no-aplicada` -> rojo (mismo test).
  - `rechazar` que no aplaza (cuerpo vacio) -> rojo (`rechazar_aplaza_las_aplicables`).
  - En `desinstalar`, quitar la llamada a `rechazar` -> rojo (`tests/desinstalar.rs::worktree_bloqueado_o_carpeta_actual_dentro_rechaza`).
  - `del_arbol` que vuelve a tragarse todo como `SinConfig` -> rojo en `config::del_arbol_distingue_no_existe_de_no_se_lee` y en `tests/verificar.rs::config_ilegible_da_error_y_el_hook_no_bloquea` (codigo 3 en vez de 1).
  - `Informe::mensaje` que pierde el diagnostico -> rojo (`init::aplicar::informe_como_mensaje`).
  - `confianza::estado` sin guardar la cache (`let _ = c.guardar(..)` fuera) -> rojo (`tests/verificar.rs::en_caliente_no_se_rehashea_y_un_script_tocado_si`).
  - Guarda: forma de la ficha A7-10 (`EXTERNO` y `Mensaje::externo(format!` en `desinstalar`) -> rojo.
  - Sobreviven: `externo("prosa")` con la guarda (MEN-1) y `externo(diagnostico_en(Es))` en `init::aplicar::Excluir` con la suite de `init` (MEN-2). Sobrevive tambien `aplicable()` de `desinstalar` con `!= Pendiente`: equivalente (las lineas no `Borra` llevan `Efecto::Ninguno`), no es hallazgo.
- Equivalencia de `confianza`: `leer_almacen` solo devuelve entradas con `version == VERSION_ALMACEN` o vacio, por lo que `guardar` fijando la version tambien en `estado` (donde antes no se fijaba) no cambia nada: `Confiado` exige entrada previa, luego version 2. `confiar` pasa `cache = false` (rehashea todo, como antes).
- Intermitencias: no observadas (el unico plazo largo fue el binario `init`, 344 s en frio con mutante, sin fallos).
- Limpieza: el clon y su target se borran al terminar.

## Revisado sin hallazgos

- `config::del_arbol`: `NotFound` -> `sin-config`; cualquier otro error (no UTF-8, sin permiso, directorio) -> `config error ruta=...` con causa y sin la pista de `tc2 init`; los dos llamadores (`confiar_en`, `decidir_y_lanzar`) reescriben `comando` y propagan con `?`, y `--hook` sale 0 sin ejecutar nada (test de la tarea, rojo con el mutante).
- `lectura::opcional`/`no_se_lee` y la migracion de `init`/`desinstalar` (mismo informe, mismo `ruta`, mismo diagnostico; solo cambia que el nombre de la orden se pasa).
- `plan_dato`: el filtro perezoso `filter(aplicable)` con `aplazar` dentro del bucle equivale al bucle anterior (la linea que falla y las siguientes quedan `pendiente`; las no aplicables no se tocan); `init` conserva el informe de `puertas` y `desinstalar` el `ruta` del fallo.
- `Informe::mensaje` y su uso en `estado/lectura.rs`, `init` y `desinstalar` (conserva el diagnostico anidado y traducible; sin diagnostico, la linea de contrato como `EXTERNO`, ADR F1-4).
- `carpeta_actual(NOMBRE)`: mismo `Informe` que el codigo que sustituye.
- `ciclo::prueba` y `repo_con_remoto`: el test `md_de_partida` fija el literal `el de entonces` y las dos variantes de `Extra`; los tests de `ciclo` salen igual que antes.
- Sin `unwrap`/`expect`/`panic!` nuevos en codigo de produccion; sin `unsafe`; sin dependencias nuevas.

## Lecturas fuera del contexto permitido

- `crates/tc2/src/confianza/mod.rs` (`leer_almacen`, `confiar_en`) y `crates/tc2/src/verificacion/ejecutar.rs` (`decidir_y_lanzar`): llamadores de `del_arbol` y del almacen; necesarios para comprobar el codigo de salida y la equivalencia de `estado`/`confiar`.
- `crates/tc2/src/tablero.rs` (`carpeta_actual`): para comprobar que el cambio de `init::orden` y `desinstalar::orden` no altera el informe.
- Usos de `Idioma::Es` y `Result<_, String>` en el resto de `crates/tc2/src` (busqueda): para valorar los huecos de la guarda (MEN-1).
