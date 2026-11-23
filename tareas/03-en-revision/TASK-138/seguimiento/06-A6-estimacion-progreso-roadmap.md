# Seguimiento de TASK-138, grupo A6 (copiar fuera del repo antes de `taskctl import --tipo fix`)

### Auditoria 4D: A6 estimacion, progreso, roadmap y tablero (6 IMPORTANTE)
> Corregir los 6 casos IMPORTANTE del grupo A6 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 5 Duplicaciones (una `divergente`) y 1 Disfraz de test con mutante confirmado. A6-01 va en la tarea de la capa de órdenes (A8-06). No se pide el MENOR (A6-08). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
- A6-04 (`divergente`, va primero): el juego de tareas de `tests/siguiente.rs:53-94` y su copia de `tests/roadmap.rs:437-486` salen de una sola fuente. Si roadmap necesita `TASK-009` y otros títulos, lo dice su propio juego y no un comentario que remite al otro.
- A6-02: `linea_esperada` (`grafo.rs:642-656`) usa `ids_de_la_linea` (`roadmap/generar.rs:401-416`) en vez de reimplementarla.
- A6-03: la operación de prueba `Poner`, `fila` y `terminada` (`tests/estimacion.rs:278-345` ~ `tests/progreso.rs:92-155`) quedan en un módulo común de tests.
- A6-05 y A6-06: `siguiente::tests` usa el `EnMemoria` de `roadmap::tests` (`siguiente.rs:139-149` ~ `roadmap/mod.rs:578-588`), y `c5_sin_red_y_sin_origin_sin_claves` llama a `start_de_luis` (`tests/solape.rs:514-529` ~ `:602-617`).
- A6-07: `c7_fuera_de_un_repo` (`progreso/mod.rs:479-497`) tiene una entrada cuyo resultado depende de usar la carpeta del proceso como respaldo (adenda TASK-084 de F1-17, `orden_estado` en `:205-210`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib progreso::tests::c7_fuera_de_un_repo` sale en rojo.
- `cargo xtask verificar` en verde, y el `## Resultado` lista los 6 IDs con su commit.
