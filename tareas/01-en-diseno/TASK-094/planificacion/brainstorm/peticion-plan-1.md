# Peticion de redaccion del plan — TASK-094 (ronda 1)

- Tarea: TASK-094 — F1-18 ADR hito de decisiones (ADR) en init y en actualizar
- Tipo: feature · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-14
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
- Heuristica (2 puntos): **simple**
- Senales encontradas:
  - dependencias: 2 dependencias (TASK-012, TASK-013) (+2)

Los dos niveles coinciden. Se lanzan 1 rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.

## Enunciado de la tarea

### Objetivo

````
ADR: el «Hito de decisiones (ADR)» como paso del producto en todo proyecto (no «Hito 0»): en un proyecto nuevo (`init`), investigar el contexto y escribir los ADR antes de la primera tarea de codigo; en un proyecto existente o con la 0.16 (`actualizar`), un estudio de adopcion que inventaria lo que se hace a mano, mide el uso real (tareas, rondas, tokens, verificacion), contrasta con la 2.0 y deja ADR y tareas para cada hueco. Obligatorio pero proporcional (su tamano sale del propio estudio), y `start` de una tarea de codigo lo exige cerrado o declarado no necesario por una persona, con un comando determinista. Decidido con Carlos el 2026-11-14 durante el H1; precedentes medidos: el H1 de TaskCodeTwo (ADR F1-1..F1-11) y el inventario de F1-11 sobre ToolsaWeb y OpenDockTrading.
````

### Criterios de aceptacion

````
[ ] Contrato del hito en `init` y en `actualizar` (entradas, salidas, cuando se considera cerrado) con un ejemplo medido sobre OpenDockTrading
[ ] Regla de proporcionalidad con cifras del H1 (tokens y tiempo por ADR) y comando que lo exige antes de `start` en tareas de codigo
[ ] Cumple RR-13 y lo que deciden F1-11 (inventario parche/propio) y F1-12 (actualizacion)
````
