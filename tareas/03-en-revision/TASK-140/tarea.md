---
id: TASK-140
titulo: "Auditoria 4D: A2 hallazgos y revision (7 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-140-auditoria-4d-a2-hallazgos-y-revision-7-i
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1263206
tokens_implementacion: 12963980
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: []
---
## Objetivo

Corregir los 7 casos IMPORTANTE del grupo A2 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones en `crates/tc2/src/hallazgos/` y 3 Disfraces de test con mutante confirmado. A2-01 y A2-06 se fusionaron en las tareas de abstracción común (A8-07 y A8-05) y aquí no se piden. Tampoco los MENOR (A2-07 a A2-10). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A2-02 y A2-03: la extracción del id válido y la closure del `Fallo` (`hallazgos/mod.rs:695-704` ~ `:828-837`) y el corte por `TOPE_HALLAZGOS` con su conversión `Fallo -> Rechazo` (`mod.rs:956-977` ~ `:980-1013`) quedan escritos una vez cada uno.
- [x] A2-04: la fila Markdown de un hallazgo y sus topes de `plano` (64, 200, 400) están en una sola función, que usan `mod.rs:1193-1204`, `mod.rs:1866-1881` y `ronda.rs:325-337`.
- [x] A2-05: los siete analizadores de opciones de `hallazgos/orden.rs` (`:117-139`, `:287-302`, `:435-446`, `:488-511`, `:664-678`, `:717-731`, `:759-798`) usan un analizador común. `uso` y `argumento-desconocido` siguen saliendo igual, y lo comprueba `cargo test -p tc2 --test hallazgos`.
- [x] A2-11: hay un test que comprueba el dictamen de `veredicto_ronda` en modo `tope` y en modo `ligera` (`mod.rs:1561-1564`). Con los dos mutantes de la ficha aplicados, `cargo test -p tc2 --lib hallazgos` sale en rojo; hoy sale en verde.
- [x] A2-12: hay un test que ejecuta la rama «mezcla con conflictos -> `None`» de `desde_incremental` (`orden.rs:653-657`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --test hallazgos el_diff_incremental_no_arrastra_lo_que_trae_la_base` (o el test nuevo) sale en rojo.
- [x] A2-13: `cada_regla_del_cierre_rechaza_con_su_motivo` (o un test nuevo) prueba el tope de cierres `fuera-de-tope` (`mod.rs:982-993`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo` sale en rojo.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 7 IDs con su commit.

## Resultado

Corregidos los 7 IMPORTANTE del grupo A2 en `crates/tc2/src/hallazgos/`, todo con piezas privadas del modulo, sin cambios de firmas `pub` ni del Markdown publicado.

| Ficha | Commit |
|---|---|
| A2-13 (D4) | df6c0dc |
| A2-11 (D4) | 17a2aa4 |
| A2-12 (D4) | e485bd3 |
| A2-02 | b0bedef |
| A2-03 | 117aa2f |
| A2-04 | 8cdddda |
| A2-05 | ef2855a |

- Mutantes de las fichas, aplicados sobre su linea literal antes del refactor, todos en rojo: A2-13 (`j >= TOPE_HALLAZGOS * 10`: `I1:id-repetido` en vez de `#100:fuera-de-tope`), A2-11 (`Tope -> Verde` y `Verde -> Incompleta`), A2-12 (`1 => (Some(arbol), "merge")`).
- Tests nuevos: `hallazgos::tests::la_fila_de_un_hallazgo_sale_entera_en_cada_tabla` (verde antes del refactor de A2-04) y `tests/hallazgos.rs::las_subordenes_analizan_sus_opciones_igual` (36 casos, verde antes y despues de A2-05). Ampliados los tests que nombran A2-11, A2-12 y A2-13.
- Decision 2: con dos errores en la misma entrada cambia el motivo (p. ej. `--revisora "no vale" --raro` pasa de `revisora-invalida` a `argumento-desconocido`); con uno la salida es identica.
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 785 (cierre de TASK-139); despues 48 binarios, 787 ok, 0 fallos, 13 ignorados (785 + 2).
- Fuera de alcance (para XO): copias de `valor` en `seleccion/orden.rs` y `alta.rs`.
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- La validacion del valor se llama `comprobar` (no `exigir`, para no confundirla con `crate::git::exigir`).
- `Args` pierde `#[derive(Default)]`, ya sin uso.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T16:45:38Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T16:52:08Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T16:52:09Z | start | automatico | persona | 0.17.1 |
| 2026-11-24T17:48:36Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T16:52:07Z | diseno | agente:ac89536960f8a860a | 46 | 77424 | 1168108 | 17628 | 31 | 1263206 | - | - |
| 2026-11-24T17:48:36Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 18 | 11625 | 5156830 | 4343 | 0 | 5172816 | - | - |
| 2026-11-24T17:48:36Z | implementacion | agente:a77a4c4c94868bde8 | 116 | 186596 | 7565581 | 38871 | 24 | 7791164 | - | - |

## Autorrevision

- rust-wasm-reviewer — ef2855a
- 4D — ef2855a: helper buscado, criterios citados, 4 mutantes vistos en rojo
