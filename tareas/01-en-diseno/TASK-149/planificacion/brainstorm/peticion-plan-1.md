# Peticion de redaccion del plan — TASK-149 (ronda 1)

- Tarea: TASK-149 — Auditoria 4D: capa comun de ordenes y operaciones con dueno (6 IMPORTANTE)
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
Extraer las piezas de la capa de órdenes que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiadas entre dueños: 6 casos IMPORTANTE de los grupos operacion-con-dueno, fallo-a-informe, envoltorio-de-orden y config-de-base, más las fichas fusionadas en ellos (A2-01, A3-04, A6-01). Toca `revision`, `plan`, `toma`, `ciclo`, `roadmap`, `estimacion`, `siguiente`, `tablero`, `seleccion` e `importar`. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A8-07 (absorbe A2-01 y A3-04): el preámbulo de `Operacion::aplicar` con dueño (limpiar `dueno`, `tarea_de`, `de_otra_persona` y guardar el login) es una función común, que usan `revision/guardar.rs:96-108`, `revision/operacion.rs:131-143`, `plan/guardar.rs:113-125`, `plan/operacion.rs:366-379`, `plan/aprobar.rs:520-533` y el `fn tarea` de `ciclo/operacion.rs:77-89`.
A8-19: el formato de la anotación con dueño (`<orden>|<id>|<cuenta>|<login>[|<resto>]`) se escribe y se parsea en un solo sitio (`revision/operacion.rs:185-196`, `toma/operacion.rs:266-282`, `ciclo/operacion.rs:241-250`, `ciclo/liberar.rs:351-357` y los `anotacion()` de la ficha). Un test de ida y vuelta compara la operación entera.
A8-06 (absorbe A6-01; resuelve de paso el MENOR A2-10): la conversión `Fallo -> Informe` es una función común (`roadmap/mod.rs:303-313`, `estimacion/mod.rs:245-254`, `siguiente.rs:76-85`, `tablero.rs:443-451`, `seleccion/orden.rs:592-602`, `ciclo/mod.rs:245-254`, `importar/mod.rs:415-428`, `revision/mod.rs:233-253`, y el resto de sitios de la ficha). La de `estimacion` conserva su `.con("id", id)`.
A8-08 y A8-09: el adaptador «informe de la transacción -> informe de la orden» (`revision/mod.rs:201-210`, `toma/mod.rs:106-115`, `ciclo/mod.rs:423-432`, `plan/mod.rs:542-551`) es uno solo, con un orden único de argumentos, y `con_dueno` y `escribir` (`ciclo/mod.rs:367-386` ~ `plan/mod.rs:487-537`) están escritos una vez.
A8-10: la política de lectura de la configuración de la base (`ciclo/mod.rs:166-177` ~ `plan/mod.rs:437-448`) es una función común, y las dos órdenes la usan con su nombre de orden en el informe.
Las líneas de contrato de `review`, `start`, `plan`, `approve`, `liberar` y `roadmap` no cambian: los tests de integración existentes siguen en verde sin tocar sus aserciones. `cargo xtask verificar` sale en verde y el `## Resultado` lista los 6 IDs con su commit.
````
