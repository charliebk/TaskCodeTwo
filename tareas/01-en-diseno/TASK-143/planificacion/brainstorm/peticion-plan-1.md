# Peticion de redaccion del plan — TASK-143 (ronda 1)

- Tarea: TASK-143 — Auditoria 4D: A5 seleccion, inventario, recursos y doctor (6 IMPORTANTE)
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
Corregir los 6 casos IMPORTANTE del grupo A5 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones (dos `divergente`), 1 Desviación de F1-15 que cambia el comportamiento y 1 Disfraz de test con mutante confirmado. A5-07 va en la tarea de soporte de tests compartido. No se piden los MENOR (A5-04, A5-05, A5-10 a A5-14). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A5-01 (`divergente`, va primero): leer skills de una carpeta (`seleccion/catalogo.rs:232` ~ `inventario.rs:423`) es una sola función. El catálogo deja incidencia de lo que no puede leer, como exige C8 (`inventario.rs:28-30`), y lo prueba un test con una carpeta ilegible.
A5-02 (`divergente`): la carpeta instalada del plugin habilitado se resuelve con una sola regla, la del `scope` del proyecto (`seleccion/catalogo.rs:265` ~ `inventario.rs:815`). Lo prueba un test con dos instalaciones de `scope` distinto.
A5-03: con una revisora de dominio declarada y no instalada, la selección da el aviso `dominio-sin-revisora` (F1-15, S10 y S11, líneas 527 y 551-555) en vez de `no-instalado` (`seleccion/mod.rs:344`, `revisoras_de` en `:260`). Lo prueba un test nuevo.
A5-06: `las_reglas_se_leen_de_la_tabla` (`seleccion/mod.rs:849`) no convierte el `Err` en `default()`. Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib seleccion::tests::las_reglas_se_leen_de_la_tabla` sale en rojo.
A5-08 y A5-09: la comprobación «5 líneas, JSON con todo y `y N mas`» (`tests/doctor.rs:958` ~ `tests/recursos.rs:378`) queda en un ayudante, y `doctor_no_escribe_nada` (`tests/doctor.rs:1017`) llama a `doctor_contra` (`:519`).
`cargo xtask verificar` en verde, y el `## Resultado` lista los 6 IDs con su commit.
````
