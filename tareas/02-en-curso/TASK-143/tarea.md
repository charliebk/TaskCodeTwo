---
id: TASK-143
titulo: "Auditoria 4D: A5 seleccion, inventario, recursos y doctor (6 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: fix/task-143-auditoria-4d-a5-seleccion-inventario-rec
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2679413
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-25
dependencias: []
---
## Objetivo

Corregir los 6 casos IMPORTANTE del grupo A5 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones (dos `divergente`), 1 Desviación de F1-15 que cambia el comportamiento y 1 Disfraz de test con mutante confirmado. A5-07 va en la tarea de soporte de tests compartido. No se piden los MENOR (A5-04, A5-05, A5-10 a A5-14). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A5-01 (`divergente`, va primero): leer skills de una carpeta (`seleccion/catalogo.rs:232` ~ `inventario.rs:423`) es una sola función. El catálogo deja incidencia de lo que no puede leer, como exige C8 (`inventario.rs:28-30`), y lo prueba un test con una carpeta ilegible.
- [x] A5-02 (`divergente`): la carpeta instalada del plugin habilitado se resuelve con una sola regla, la del `scope` del proyecto (`seleccion/catalogo.rs:265` ~ `inventario.rs:815`). Lo prueba un test con dos instalaciones de `scope` distinto.
- [x] A5-03: con una revisora de dominio declarada y no instalada, la selección da el aviso `dominio-sin-revisora` (F1-15, S10 y S11, líneas 527 y 551-555) en vez de `no-instalado` (`seleccion/mod.rs:344`, `revisoras_de` en `:260`). Lo prueba un test nuevo.
- [x] A5-06: `las_reglas_se_leen_de_la_tabla` (`seleccion/mod.rs:849`) no convierte el `Err` en `default()`. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib seleccion::tests::las_reglas_se_leen_de_la_tabla` sale en rojo.
- [x] A5-08 y A5-09: la comprobación «5 líneas, JSON con todo y `y N mas`» (`tests/doctor.rs:958` ~ `tests/recursos.rs:378`) queda en un ayudante, y `doctor_no_escribe_nada` (`tests/doctor.rs:1017`) llama a `doctor_contra` (`:519`).
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 6 IDs con su commit.

## Resultado

Corregidos los IMPORTANTE del grupo A5 (A5-07 lo cerro TASK-148).

| Ficha | Commit |
|---|---|
| A5-01 (`divergente`) | 9388109 |
| A5-02 (`divergente`) | 91a6e16 |
| A5-03 (D2 de F1-15) | 4b93705 |
| A5-06 (D4) | 495aee4 |
| A5-08 | 7862b68 |
| A5-09 | 26dbb31 |

- Cambios de comportamiento aceptados: en `tc2 revisoras --plugin`, lo ilegible sale como aviso `ilegible:<motivo>:<id>` (antes desaparecia sin rastro; la rama sin `--plugin` queda como pendiente anotado); las revisoras de un plugin salen de la instalacion que el inventario da por habilitada para el `scope` del proyecto; una revisora de dominio declarada y no instalada da S11 con `dominio-sin-revisora:<d>` en vez de S10/S14 con `no-instalado` (F1-15:551-555).
- Rojos antes del arreglo: E2E de A5-01 con avisos `["config-de-base-no-leida"]` sin el de ilegible; A5-02 leia la instalacion de otro proyecto (`["p:de-a"]` frente a `["p:de-b"]`); A5-03 `["no-instalado:security-review"]` frente a `["dominio-sin-revisora:puertas-seguridad"]`. A5-06: verde con el codigo actual, rojo con el mutante de la ficha (`Err("S12: condicion fuera del vocabulario: carril=fix")`) y con el control (`agentes: ["general-purpose"]`).
- Tests nuevos: `inventario::tests::skills_de_una_carpeta_deja_incidencias`, `tests/revisoras.rs::plugin_con_skills_ilegible_da_aviso`, `inventario::tests::la_carpeta_del_plugin_es_la_del_scope_del_proyecto`, `seleccion::tests::revisora_de_dominio_no_instalada_es_dominio_sin_revisora`; reforzado `las_reglas_se_leen_de_la_tabla`. Ayudantes de test `comun::salida_acotada`, `doctor_contra_en` y `conectar` (tambien en el caso sin red).
- API publica: campo nuevo `inventario::Plugin::carpeta`, nueva firma de `catalogo::de_instalados`, campos nuevos en `catalogo::Lectura`.
- Fixtures de seleccion: sin cambios (`diez_tareas_de_taskcodetwo` en verde).
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 796 (cierre de TASK-142); despues 48 binarios, 800 ok, 0 fallos, 13 ignorados (796 + 4).
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- `salida_acotada` absorbe tambien las comprobaciones de JSON en una linea y sin ANSI, duplicadas en los dos tests.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-25T01:23:09Z | plan | automatico | persona | 0.17.1 |
| 2026-11-25T01:35:23Z | approve | automatico | persona | 0.17.1 |
| 2026-11-25T01:35:26Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-25T01:35:21Z | diseno | agente:a5aabafa72024696c | 72 | 104702 | 2549422 | 25217 | 45 | 2679413 | - | - |

## Autorrevision

- rust-wasm-reviewer — 26dbb31
- 4D — 26dbb31: helper buscado, criterios citados, 2 mutantes vistos en rojo
