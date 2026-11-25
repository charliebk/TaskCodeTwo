# Peticion de redaccion del plan — TASK-144 (ronda 1)

- Tarea: TASK-144 — Auditoria 4D: A6 estimacion, progreso, roadmap y tablero (6 IMPORTANTE)
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-25
- Rol: `brainstorm-arquitectura` — lanzalo con el agente de ese mismo nombre
- Quien orquesta vuelca tu respuesta en: `../plan-final.md`

**Esta tarea se planifica con un solo rol y no hay unificador**: tu respuesta ES el plan final. Redactala con las secciones de abajo; quien orquesta la vuelca tal cual en `plan-final.md` (tu no escribes ficheros), sin salida intermedia que consolidar.

## Tu pregunta

> Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?

## Que miras

- Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.
- Que se crea nuevo, que se extiende y en que orden se construye.
- Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.
- El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.

## Que NO miras

Que lo mires solo tu no te autoriza a cubrirlo todo: lo que queda fuera de tu pregunta no se mira, pero se DICE. Anota en el plan lo que no cubres, para que quien lo apruebe sepa que ese lado no se ha revisado:

- Como se prueba (es del rol de testing).
- Por donde se rompe (es del rol de riesgos).
- Las reglas de negocio (son del rol de dominio).

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **no declarada (decide la heuristica)**
- Heuristica (1 puntos): **simple**
- Senales encontradas:
  - criterios_aceptacion: 6 criterios (umbral: 5) (+1)

Los dos niveles coinciden. Se lanzan 1 rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.
- Reutilizacion (Regla 4D): lo que ya existe en el repo y se va a usar (helpers, funciones, fixtures, tests), buscado ANTES de proponer nada nuevo; y lo que se crea nuevo, con el motivo de no reutilizar lo que hay. Duplicar un helper que ya existe es un hallazgo IMPORTANTE.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.

## Enunciado de la tarea

### Objetivo

````
Corregir los 6 casos IMPORTANTE del grupo A6 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 5 Duplicaciones (una `divergente`) y 1 Disfraz de test con mutante confirmado. A6-01 va en la tarea de la capa de órdenes (A8-06). No se pide el MENOR (A6-08). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A6-04 (`divergente`, va primero): el juego de tareas de `tests/siguiente.rs:53-94` y su copia de `tests/roadmap.rs:437-486` salen de una sola fuente. Si roadmap necesita `TASK-009` y otros títulos, lo dice su propio juego y no un comentario que remite al otro.
A6-02: `linea_esperada` (`grafo.rs:642-656`) usa `ids_de_la_linea` (`roadmap/generar.rs:401-416`) en vez de reimplementarla.
A6-03: la operación de prueba `Poner`, `fila` y `terminada` (`tests/estimacion.rs:278-345` ~ `tests/progreso.rs:92-155`) quedan en un módulo común de tests.
A6-05 y A6-06: `siguiente::tests` usa el `EnMemoria` de `roadmap::tests` (`siguiente.rs:139-149` ~ `roadmap/mod.rs:578-588`), y `c5_sin_red_y_sin_origin_sin_claves` llama a `start_de_luis` (`tests/solape.rs:514-529` ~ `:602-617`).
A6-07: `c7_fuera_de_un_repo` (`progreso/mod.rs:479-497`) tiene una entrada cuyo resultado depende de usar la carpeta del proceso como respaldo (adenda TASK-084 de F1-17, `orden_estado` en `:205-210`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib progreso::tests::c7_fuera_de_un_repo` sale en rojo.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 6 IDs con su commit.
````
