---
id: TASK-150
titulo: "Auditoria 4D: utilidades comunes de E-S y tiempo (4 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-150-auditoria-4d-utilidades-comunes-de-e-s-y
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1470926
tokens_implementacion: 14586326
tokens_revision: 8784640
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Extraer las utilidades de E/S y de tiempo que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiadas entre dueños: 4 casos IMPORTANTE (tres `divergente`) de los grupos escritura-atomica, lectura-con-tope, espera-con-plazo y fechas-utc, más la ficha fusionada A7-02. Toca `huella`, `confianza`, `desinstalar`, `identidad`, `estado`, `hallazgos`, `seleccion`, `progreso`, `actualizar_cli`, `verificacion`, `alta`, `toma` y `xtask`. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A8-04 (`divergente`, absorbe A7-02; va primero): una sola escritura atómica, con el reintento del `rename` en Windows de `huella.rs:89-116`, la usan `confianza/mod.rs:160-178` y `desinstalar/mod.rs:571-588`. Fija el mismo nombre de temporal, conserva los permisos y borra el temporal si falla `write` o `rename`, y lo prueba un test de la propia función.
- [x] A8-17 (`divergente`): un solo «leer con tope» (`take(tope + 1)` y rechazar si se pasa) sustituye a los de `identidad.rs:229-240`, `estado/frescura.rs:171-181`, `hallazgos/orden.rs:153-160`, `seleccion/orden.rs:280-287`, `progreso/fichero.rs:125-133`, `progreso/mod.rs:51-59` y el resto de la ficha. Se decide y se prueba qué hace con los bytes que no son UTF-8.
- [x] A9-01 (`divergente`): «esperar a un hijo con plazo y matarlo» es una sola función (`actualizar_cli.rs:574-592` ~ `verificacion/ejecutar.rs:380-393`) que mata el árbol entero (`matar_arbol`), y la documentación de `actualizar_cli.rs:516-518` dice lo que hace.
- [x] A8-03: el formateo `AAAA-MM-DDTHH:MM:SSZ` y el algoritmo civil están una sola vez en `tc2` (`alta.rs:673-698` ~ `toma/operacion.rs:250-261`), y el test `iso` de `tests/identidades.rs:366-384` usa la función del crate. `xtask/src/main.rs:430-448` (`fecha_utc`) se queda como está si xtask sigue sin depender de tc2, y se anota el motivo.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 4 IDs con su commit.

## Resultado

Utilidades comunes de E/S y tiempo: los 4 IMPORTANTE (A8-04 con A7-02, A8-17, A9-01, A8-03). Unico cambio de contrato, el decidido: un informe de revisor que no es UTF-8 da `informe=invalido` en `hallazgos` y `seleccion`.

| Ficha | Commit | Que queda |
|---|---|---|
| A8-04 (A7-02) | 73996b0 | `crate::escritura::atomica` (enlaces, permisos, carpeta, temporal `<nombre>.tc2-tmp-<pid>`, reintento y borrado del temporal); se borran `huella::escribir_local`, `desinstalar::escribir_atomico` y la escritura en linea de `confianza` |
| A8-17 | 6671650 | `lectura::{acotado, con_tope, texto_con_tope}`; se borran las dos `leer_con_tope` y 4 lecturas en linea; stdin de `seleccion` incluido |
| A9-01 | 27b9d9e | `crate::proceso::{grupo_propio, esperar_con_plazo}` con `matar_arbol`; `ejecutar_con_limite` mata el arbol al plazo |
| A8-03 | 689cb93 | `tc2::fecha::{instante, fecha_de_dias}`; se borran `alta::instante_utc`, `alta::fecha_de_dias`, `toma::operacion::instante` y el `iso` de `tests/identidades.rs` |

Tests nuevos con mutante en rojo: `escritura::tests::{reintenta_con_un_lector_abierto, rename_que_falla_borra_el_temporal}`, `lectura::tests::{acotado_en_la_frontera, texto_con_tope_es_utf8_estricto}`, `seleccion::orden::tests::informe_no_utf8_es_invalido`, `proceso::tests::al_plazo_mata_el_arbol_entero`, `fecha::tests::{fechas_civiles, instante_en_utc}` e `iso_formatea_como_github`. `escritura::tests::sigue_el_enlace_y_conserva_los_permisos` es `cfg(unix)`: no corre en esta maquina Windows y sus mutantes no se han comprobado.

Suite completa (`cargo test --workspace --no-fail-fast`, target propio): 865 ok, 0 fallidos, 13 ignorados. `cargo xtask verificar` lo ejecuta `finish`.

### Divergencias

- `seleccion::orden::Entorno.stdin` devuelve `io::Result<Option<Vec<u8>>>` para comprobar el tope en un solo sitio.
- Si falla `try_wait` en `verificacion`, ahora tambien mata el arbol antes de devolver el error.
- El mutante de frontera se expresa como frontera estricta (el codigo usa `<=`).
- Revision 1, IMP-1 y MEN-1 (3c1d524): las entradas que no son informes (respuesta, diff, contexto, JSON y rutas de `hallazgos`; `--rutas` y `--tarea` de `seleccion`) vuelven a leerse lossy con `lectura::texto_con_tope_lossy`, como en develop; el estricto queda solo para los informes del revisor. Test `diff_y_respuesta_no_utf8_se_leen`. MEN-2 (9ce49a2): doc de `escritura::atomica`. Suite: 866 ok, 0 fallidos, 13 ignorados.

### Fuera del plan (no tocado)

`actualizar_cli::sustituir_con` (sustitucion del binario, no escritura atomica), `descarga.rs:377` (cuerpo HTTP), las lecturas que truncan sin rechazar, el bucle `try_wait` de `tests/progreso.rs:443` (otro crate), `inventario::dias_desde_epoca` (inverso) y `xtask::fecha_utc` (motivo anotado).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T18:33:25Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T18:41:47Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T18:41:49Z | start | automatico | persona | 0.17.1 |
| 2026-11-25T19:42:14Z | review | automatico | persona | 0.17.1 |
| 2026-11-25T20:53:26Z | review | automatico | persona | 0.17.1 |
| 2026-11-25T22:07:32Z | finish | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T18:41:46Z | diseno | agente:afaf9edd246673d34 | 52 | 89610 | 1355310 | 25954 | 51 | 1470926 | - | - |
| 2026-11-25T19:42:14Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 18 | 13274 | 2413773 | 4291 | 0 | 2431356 | - | - |
| 2026-11-25T19:42:14Z | implementacion | agente:a75fa5f062ea72442 | 158 | 407532 | 11694112 | 53168 | 7 | 12154970 | - | - |
| 2026-11-25T20:00:30Z | revision | agente:af7b3472141f92569 | 42 | 141134 | 2529203 | 13358 | 2 | 2683737 | 1 | rust-wasm-reviewer |
| 2026-11-25T21:18:13Z | revision | agente:ac816c5d430368ffb | 50 | 114480 | 2506870 | 7513 | 3 | 2628913 | 2 | rust-wasm-reviewer |
| 2026-11-25T21:18:13Z | revision | agente:a27ecd91f81f9f3b1 | 70 | 245323 | 3210367 | 16230 | 12 | 3471990 | - | - |

## Autorrevision

- rust-wasm-reviewer — 689cb93
- 4D — 689cb93: helper buscado, criterios citados, 9 tests mutados
- rust-wasm-reviewer — 9ce49a2

## Duracion

- Estimada: 150 min
- Real (de la rama abierta al cierre con merge, sin pausas): 206 min
- Paso de la hora: 206 min reales frente a 150 estimados
- Cierre (finish): 2939 s, de 2026-11-25T21:18:33Z a 2026-11-25T22:07:32Z
