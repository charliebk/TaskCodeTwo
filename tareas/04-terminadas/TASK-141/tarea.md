---
id: TASK-141
titulo: "Auditoria 4D: A3 plan, importar y alta (5 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: fix/task-141-auditoria-4d-a3-plan-importar-y-alta-5-i
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2314328
tokens_implementacion: 21749855
tokens_revision: 10090939
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: []
---
## Objetivo

Corregir los 5 casos IMPORTANTE del grupo A3 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 2 Duplicaciones (una `divergente`), 1 Desviación que cambia el comportamiento y 2 Disfraces de test con mutante confirmado. A3-04 y A3-06 van en las tareas de abstracción común (A8-07 y soporte de tests). No se piden los MENOR (A3-07, A3-09). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [x] A3-03 (`divergente`, va primero): `leer_fichero` de `plan/mod.rs:139-170` e `importar/mod.rs:246-271` es una sola función con un solo tope (`plan/guardar.rs:33-34` ~ `importar/fichero.rs:32-33`). `import` sanea la ruta y el error antes del diagnóstico (F1-8) y rechaza el fichero vacío, como ya hace `plan`, y lo prueba un test de `import` con una ruta que lleva un salto de línea.
- [x] A3-05: la secuencia de `approve` y la de `approve --hito` (`plan/mod.rs:283-295` ~ `:349-362`) comparten una sola función. `cargo test -p tc2 --test aprobar` sigue en verde.
- [x] A3-08: `new` e `import` rechazan al dar de alta un enunciado que no está listo según la misma regla que `plan` (`alta.rs:75-108` frente a `plan/heuristica.rs:130-144`, RF-01). Lo prueba un test de `new` con un enunciado que hoy acepta y `plan` rechaza.
- [x] A3-01: `anotacion_ida_y_vuelta` (`alta.rs:1051-1058`) compara la operación reconstruida campo a campo, y `desde_anotacion` (`alta.rs:567-571`) deja de perder lo que no es el título, o documenta y prueba por qué no lo necesita. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta` sale en rojo.
- [x] A3-02: un test llama a `es_esqueleto` con rutas que eligen roles por dominio, la rama con unificador de `plan/aprobar.rs:117`. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib plan::aprobar` sale en rojo.
- [x] `cargo xtask verificar` en verde, y el `## Resultado` lista los 5 IDs con su commit.

## Resultado

Corregidos los 5 IMPORTANTE del grupo A3.

| Ficha | Commit |
|---|---|
| A3-03 (`divergente`) | 1fcc11b |
| A3-05 | cacc96b |
| A3-08 (D2) | 345ccd6 |
| A3-01 (D4) | 324b2f8 |
| A3-02 (D4) | ab11d28 |

- A3-03: modulo privado `crate::lectura` (`MAX_BYTES`, `fichero`, `NoSeLeyo` con ruta y error saneados) como unica lectura acotada; `plan` e `import` solo traducen a su informe. Se borra `plan::guardar::MAX_BYTES`; `importar::fichero::MAX_BYTES` queda como reexportacion.
- Cambios de comportamiento aceptados: `import` de un fichero vacio o solo con espacios da `import uso motivo=vacio` (antes `sin-entradas`) y la ruta y el error salen saneados en stderr; `new`/`import` con criterios y sin objetivo dan `causa=objetivo-vacio`, y con 13 criterios `causa=demasiados-criterios`. El borrador solo con titulo se sigue admitiendo.
- Mutantes D4 en rojo: A3-01 (`Alta::nueva("otro")`, antes y despues del refactor) y A3-02 (borrar la rama de `juegos_de_rondas`, `aprobar.rs:117`). Extra: quitar `sanear` de `lectura.rs` pone en rojo `ruta_que_no_se_lee_saneada`.
- Tests nuevos: `importar::tests::ruta_que_no_se_lee_saneada`, `plan::aprobar::tests::esqueleto_de_los_roles_de_la_ronda`; ampliados `c7_limites_y_sin_red` (import), `c9_` (alta), `entradas_malas_rechazan_el_lote_entero` y `anotacion_ida_y_vuelta`.
- Fixtures ajustadas de forma mecanica (`--objetivo o` o `> o`) en `tests/estimacion.rs`, `tests/progreso.rs`, `tests/toma.rs`, `tests/importar.rs` y unitarios de `importar/fichero.rs` (con los numeros de linea y longitud recalculados); ninguno se debilita.
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 787 (cierre de TASK-140); despues 48 binarios, 789 ok, 0 fallos, 13 ignorados (787 + 2).
- Para TASK-150 (decision 4): su «leer con tope» unico va a `crate::lectura`.
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- `CAUSA_DEMASIADOS_CRITERIOS` lleva `{max}` con `MAX_CRITERIOS` para no duplicar el 12.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T19:39:12Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T19:50:50Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T19:50:54Z | start | automatico | persona | 0.17.1 |
| 2026-11-24T21:11:24Z | review | automatico | persona | 0.17.1 |
| 2026-11-24T22:27:32Z | finish | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T19:50:48Z | diseno | agente:a39668713434380c8 | 66 | 111657 | 2177396 | 25209 | 49 | 2314328 | - | - |
| 2026-11-24T21:11:24Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 24 | 15380 | 7355211 | 5406 | 0 | 7376021 | - | - |
| 2026-11-24T21:11:24Z | implementacion | agente:a6f4f072a8c0e9c53 | 188 | 211328 | 14109190 | 53128 | 15 | 14373834 | - | - |
| 2026-11-24T22:02:08Z | revision | agente:a5047424bd7e80da7 | 144 | 173429 | 9889892 | 27474 | 18 | 10090939 | 1 | rust-wasm-reviewer |

## Autorrevision

- rust-wasm-reviewer — ab11d28
- 4D — ab11d28: helper buscado, criterios citados, 3 mutantes vistos en rojo

## Verificacion ligera

- 2026-11-24T22:02:10Z: ronda 1; commit revisado 65a9b47af269e1b7ec3bd6ee34daace322bf0d0d; commit verificado 7dc870a79156e7964c31f86977d1c57078d998c2; ficheros: crates/tc2/src/alta.rs, crates/tc2/src/lectura.rs; corregidos: MEN-1, MEN-2, MEN-4; pendientes: MEN-3

## Duracion

- Estimada: 210 min
- Real (de la rama abierta al cierre con merge, sin pausas): 157 min
- Paso de la hora: 157 min reales frente a 210 estimados
- Cierre (finish): 1510 s, de 2026-11-24T22:02:22Z a 2026-11-24T22:27:32Z
