# Peticion de redaccion del plan — TASK-141 (ronda 1)

- Tarea: TASK-141 — Auditoria 4D: A3 plan, importar y alta (5 IMPORTANTE)
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-24
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
Corregir los 5 casos IMPORTANTE del grupo A3 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 2 Duplicaciones (una `divergente`), 1 Desviación que cambia el comportamiento y 2 Disfraces de test con mutante confirmado. A3-04 y A3-06 van en las tareas de abstracción común (A8-07 y soporte de tests). No se piden los MENOR (A3-07, A3-09). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A3-03 (`divergente`, va primero): `leer_fichero` de `plan/mod.rs:139-170` e `importar/mod.rs:246-271` es una sola función con un solo tope (`plan/guardar.rs:33-34` ~ `importar/fichero.rs:32-33`). `import` sanea la ruta y el error antes del diagnóstico (F1-8) y rechaza el fichero vacío, como ya hace `plan`, y lo prueba un test de `import` con una ruta que lleva un salto de línea.
A3-05: la secuencia de `approve` y la de `approve --hito` (`plan/mod.rs:283-295` ~ `:349-362`) comparten una sola función. `cargo test -p tc2 --test aprobar` sigue en verde.
A3-08: `new` e `import` rechazan al dar de alta un enunciado que no está listo según la misma regla que `plan` (`alta.rs:75-108` frente a `plan/heuristica.rs:130-144`, RF-01). Lo prueba un test de `new` con un enunciado que hoy acepta y `plan` rechaza.
A3-01: `anotacion_ida_y_vuelta` (`alta.rs:1051-1058`) compara la operación reconstruida campo a campo, y `desde_anotacion` (`alta.rs:567-571`) deja de perder lo que no es el título, o documenta y prueba por qué no lo necesita. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib alta::tests::anotacion_ida_y_vuelta` sale en rojo.
A3-02: un test llama a `es_esqueleto` con rutas que eligen roles por dominio, la rama con unificador de `plan/aprobar.rs:117`. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib plan::aprobar` sale en rojo.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 5 IDs con su commit.
````
