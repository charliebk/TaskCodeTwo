# Peticion de redaccion del plan — TASK-150 (ronda 1)

- Tarea: TASK-150 — Auditoria 4D: utilidades comunes de E-S y tiempo (4 IMPORTANTE)
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
  - criterios_aceptacion: 5 criterios (umbral: 5) (+1)

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
Extraer las utilidades de E/S y de tiempo que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiadas entre dueños: 4 casos IMPORTANTE (tres `divergente`) de los grupos escritura-atomica, lectura-con-tope, espera-con-plazo y fechas-utc, más la ficha fusionada A7-02. Toca `huella`, `confianza`, `desinstalar`, `identidad`, `estado`, `hallazgos`, `seleccion`, `progreso`, `actualizar_cli`, `verificacion`, `alta`, `toma` y `xtask`. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A8-04 (`divergente`, absorbe A7-02; va primero): una sola escritura atómica, con el reintento del `rename` en Windows de `huella.rs:89-116`, la usan `confianza/mod.rs:160-178` y `desinstalar/mod.rs:571-588`. Fija el mismo nombre de temporal, conserva los permisos y borra el temporal si falla `write` o `rename`, y lo prueba un test de la propia función.
A8-17 (`divergente`): un solo «leer con tope» (`take(tope + 1)` y rechazar si se pasa) sustituye a los de `identidad.rs:229-240`, `estado/frescura.rs:171-181`, `hallazgos/orden.rs:153-160`, `seleccion/orden.rs:280-287`, `progreso/fichero.rs:125-133`, `progreso/mod.rs:51-59` y el resto de la ficha. Se decide y se prueba qué hace con los bytes que no son UTF-8.
A9-01 (`divergente`): «esperar a un hijo con plazo y matarlo» es una sola función (`actualizar_cli.rs:574-592` ~ `verificacion/ejecutar.rs:380-393`) que mata el árbol entero (`matar_arbol`), y la documentación de `actualizar_cli.rs:516-518` dice lo que hace.
A8-03: el formateo `AAAA-MM-DDTHH:MM:SSZ` y el algoritmo civil están una sola vez en `tc2` (`alta.rs:673-698` ~ `toma/operacion.rs:250-261`), y el test `iso` de `tests/identidades.rs:366-384` usa la función del crate. `xtask/src/main.rs:430-448` (`fecha_utc`) se queda como está si xtask sigue sin depender de tc2, y se anota el motivo.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 4 IDs con su commit.
````
