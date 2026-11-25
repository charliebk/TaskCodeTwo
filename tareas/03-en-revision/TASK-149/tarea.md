---
id: TASK-149
titulo: "Auditoria 4D: capa comun de ordenes y operaciones con dueno (6 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-149-auditoria-4d-capa-comun-de-ordenes-y-ope
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2514362
tokens_implementacion: 21769944
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Extraer las piezas de la capa de órdenes que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiadas entre dueños: 6 casos IMPORTANTE de los grupos operacion-con-dueno, fallo-a-informe, envoltorio-de-orden y config-de-base, más las fichas fusionadas en ellos (A2-01, A3-04, A6-01). Toca `revision`, `plan`, `toma`, `ciclo`, `roadmap`, `estimacion`, `siguiente`, `tablero`, `seleccion` e `importar`. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A8-07 (absorbe A2-01 y A3-04): el preámbulo de `Operacion::aplicar` con dueño (limpiar `dueno`, `tarea_de`, `de_otra_persona` y guardar el login) es una función común, que usan `revision/guardar.rs:96-108`, `revision/operacion.rs:131-143`, `plan/guardar.rs:113-125`, `plan/operacion.rs:366-379`, `plan/aprobar.rs:520-533` y el `fn tarea` de `ciclo/operacion.rs:77-89`.
- [x] A8-19: el formato de la anotación con dueño (`<orden>|<id>|<cuenta>|<login>[|<resto>]`) se escribe y se parsea en un solo sitio (`revision/operacion.rs:185-196`, `toma/operacion.rs:266-282`, `ciclo/operacion.rs:241-250`, `ciclo/liberar.rs:351-357` y los `anotacion()` de la ficha). Un test de ida y vuelta compara la operación entera.
- [x] A8-06 (absorbe A6-01; resuelve de paso el MENOR A2-10): la conversión `Fallo -> Informe` es una función común (`roadmap/mod.rs:303-313`, `estimacion/mod.rs:245-254`, `siguiente.rs:76-85`, `tablero.rs:443-451`, `seleccion/orden.rs:592-602`, `ciclo/mod.rs:245-254`, `importar/mod.rs:415-428`, `revision/mod.rs:233-253`, y el resto de sitios de la ficha). La de `estimacion` conserva su `.con("id", id)`.
- [x] A8-08 y A8-09: el adaptador «informe de la transacción -> informe de la orden» (`revision/mod.rs:201-210`, `toma/mod.rs:106-115`, `ciclo/mod.rs:423-432`, `plan/mod.rs:542-551`) es uno solo, con un orden único de argumentos, y `con_dueno` y `escribir` (`ciclo/mod.rs:367-386` ~ `plan/mod.rs:487-537`) están escritos una vez.
- [x] A8-10: la política de lectura de la configuración de la base (`ciclo/mod.rs:166-177` ~ `plan/mod.rs:437-448`) es una función común, y las dos órdenes la usan con su nombre de orden en el informe.
- [x] Las líneas de contrato de `review`, `start`, `plan`, `approve`, `liberar` y `roadmap` no cambian: los tests de integración existentes siguen en verde sin tocar sus aserciones. `cargo xtask verificar` sale en verde y el `## Resultado` lista los 6 IDs con su commit.

## Resultado

Capa comun de ordenes y operaciones de la rama de estado: los 6 casos del grupo (A8-06, A8-07, A8-08, A8-09, A8-10, A8-19, con A6-01, A2-01, A3-04 y A2-10 absorbidos) y las copias de `valor` que dejo TASK-140. Ninguna linea de contrato cambia: los tests de integracion de `review`, `start`, `plan`, `approve`, `liberar` y `roadmap` no se han tocado y pasan.

| Ficha | Commit | Que queda |
|---|---|---|
| A8-06 (A6-01, A2-10) | bb0a417 | `Fallo::informe(comando)` en `estado/transaccion.rs`; se borra `roadmap::de_fallo` |
| A8-08 | c360bba | `estado::como_orden`; se borran `de_review`, `de_start`, los dos `de_la_orden` y los dos `rechazada()` |
| A8-09 | b67b539 | `con_dueno` en `estado/dueno.rs`; `escribir` como metodo de `toma::Entorno` |
| A8-10 | b917683 | `config::de_base_o_suelo` |
| A8-07 (A2-01, A3-04) | 8f11a44 | `estado::tarea_propia`; 5 preambulos sustituidos |
| A8-19 | 932567c | `estado::AnotacionConDueno` con `escribir()`/`leer()`; 9 escritores y 4 lectores |
| copias de `valor` | 959fdad | `contrato::valor_de` y `FALTA_VALOR` (seleccion, alta, hallazgos) |

Tests nuevos, cada uno con mutantes comprobados en rojo: `fallo_como_informe_de_la_orden`, `rechazo_de_lectura_con_id_el_primero`, `informe_de_la_transaccion_como_de_la_orden` (reforzado con un caso `hito` tras sobrevivir el mutante «no quita el id»), `rechazo_con_lo_que_sabe_la_operacion`, `config_de_la_base_o_suelo_a_nombre_de_la_orden` (Git real), `tarea_propia_ajena_heredada_e_inexistente`, `anotacion_con_dueno_ida_y_vuelta` y `valor_de_la_opcion_o_falta_valor`.

Suite completa (`cargo test --workspace --no-fail-fast`, target propio): 860 ok, 0 fallidos, 13 ignorados. `cargo xtask verificar` lo ejecuta `finish`.

### Divergencias

- Toma con `resto` (riesgo aceptado en el plan): `start|TASK-001|7|ana|x` antes daba login `ana|x`; ahora `leer` da login `ana` y resto `x`, y toma la rechaza.
- Test extra `valor_de_la_opcion_o_falta_valor`: el `falta-valor` de `seleccion` no lo cubria ningun test de la orden.

### Hallazgo nuevo (copias fuera del plan, no corregidas)

Para la segunda auditoria: `estado/transaccion.rs` `aplicar_en` (~467, conversion `Fallo -> Informe` con `operacion=` delante); `importar/mod.rs:322-326` (renombrar y `pendiente=no`, hermana de `lib.rs`); `importar/mod.rs:337-339` (otra copia de `escribir`; importar no usa `toma::Entorno`); `plan/aprobar.rs:178` y `:228` (variante del preambulo con dueno que devuelve tupla); `ciclo/liberar.rs:128` (`permiso`); `arreglo=tc2 actualizar` para `formato-1` repetido en `ciclo::con_rechazo` y `toma::rechazo_de_toma`.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T15:46:58Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T15:54:57Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T15:54:58Z | start | automatico | persona | 0.17.1 |
| 2026-11-25T17:08:44Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T15:54:57Z | diseno | agente:a2556eb46cdca27b1 | 64 | 125609 | 2363006 | 25683 | 55 | 2514362 | - | - |
| 2026-11-25T17:08:44Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 24 | 16666 | 2732574 | 5373 | 0 | 2754637 | - | - |
| 2026-11-25T17:08:44Z | implementacion | agente:a6264d9616ecb04fa | 226 | 489160 | 18456423 | 69498 | 5 | 19015307 | - | - |

## Autorrevision

- rust-wasm-reviewer — 959fdad
- 4D — 959fdad: helper buscado, criterios citados, 8 tests mutados
