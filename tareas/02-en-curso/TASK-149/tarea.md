---
id: TASK-149
titulo: "Auditoria 4D: capa comun de ordenes y operaciones con dueno (6 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-149-auditoria-4d-capa-comun-de-ordenes-y-ope
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2514362
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Extraer las piezas de la capa de órdenes que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiadas entre dueños: 6 casos IMPORTANTE de los grupos operacion-con-dueno, fallo-a-informe, envoltorio-de-orden y config-de-base, más las fichas fusionadas en ellos (A2-01, A3-04, A6-01). Toca `revision`, `plan`, `toma`, `ciclo`, `roadmap`, `estimacion`, `siguiente`, `tablero`, `seleccion` e `importar`. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A8-07 (absorbe A2-01 y A3-04): el preámbulo de `Operacion::aplicar` con dueño (limpiar `dueno`, `tarea_de`, `de_otra_persona` y guardar el login) es una función común, que usan `revision/guardar.rs:96-108`, `revision/operacion.rs:131-143`, `plan/guardar.rs:113-125`, `plan/operacion.rs:366-379`, `plan/aprobar.rs:520-533` y el `fn tarea` de `ciclo/operacion.rs:77-89`.
- [ ] A8-19: el formato de la anotación con dueño (`<orden>|<id>|<cuenta>|<login>[|<resto>]`) se escribe y se parsea en un solo sitio (`revision/operacion.rs:185-196`, `toma/operacion.rs:266-282`, `ciclo/operacion.rs:241-250`, `ciclo/liberar.rs:351-357` y los `anotacion()` de la ficha). Un test de ida y vuelta compara la operación entera.
- [ ] A8-06 (absorbe A6-01; resuelve de paso el MENOR A2-10): la conversión `Fallo -> Informe` es una función común (`roadmap/mod.rs:303-313`, `estimacion/mod.rs:245-254`, `siguiente.rs:76-85`, `tablero.rs:443-451`, `seleccion/orden.rs:592-602`, `ciclo/mod.rs:245-254`, `importar/mod.rs:415-428`, `revision/mod.rs:233-253`, y el resto de sitios de la ficha). La de `estimacion` conserva su `.con("id", id)`.
- [ ] A8-08 y A8-09: el adaptador «informe de la transacción -> informe de la orden» (`revision/mod.rs:201-210`, `toma/mod.rs:106-115`, `ciclo/mod.rs:423-432`, `plan/mod.rs:542-551`) es uno solo, con un orden único de argumentos, y `con_dueno` y `escribir` (`ciclo/mod.rs:367-386` ~ `plan/mod.rs:487-537`) están escritos una vez.
- [ ] A8-10: la política de lectura de la configuración de la base (`ciclo/mod.rs:166-177` ~ `plan/mod.rs:437-448`) es una función común, y las dos órdenes la usan con su nombre de orden en el informe.
- [ ] Las líneas de contrato de `review`, `start`, `plan`, `approve`, `liberar` y `roadmap` no cambian: los tests de integración existentes siguen en verde sin tocar sus aserciones. `cargo xtask verificar` sale en verde y el `## Resultado` lista los 6 IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T15:46:58Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T15:54:57Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T15:54:58Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T15:54:57Z | diseno | agente:a2556eb46cdca27b1 | 64 | 125609 | 2363006 | 25683 | 55 | 2514362 | - | - |
