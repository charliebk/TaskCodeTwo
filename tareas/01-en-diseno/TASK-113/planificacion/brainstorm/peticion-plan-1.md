# Peticion de redaccion del plan — TASK-113 (ronda 1)

- Tarea: TASK-113 — Puerta de valor F10: informe, datos y descarte del H10
- Tipo: feature · Complejidad declarada: simple
- Ronda: 1
- Fecha: 2026-11-16
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

- Declarada en la tarea: **simple**
- Heuristica (3 puntos): **simple**
- Senales encontradas:
  - etiquetas_adicionales: 3 etiquetas (2 mas alla de la primera) (+2)
  - criterios_aceptacion: 7 criterios (umbral: 5) (+1)

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
Dejar en el repo la evidencia de la puerta de valor de la Fase 10 (experimento A/B de 2026-11-16: 64 corridas, juez a ciegas, veredicto NO-GO por el criterio fijado) y aplicar la decision de Carlos (2026-11-16): la Fase 10 se descarta. Se borran sus tareas y el hito H10 del roadmap, y los documentos que la definian quedan marcados como descartados con enlace a la evidencia, para conservar el porque.
````

### Criterios de aceptacion

````
docs/mediciones/fase10-puerta-valor/informe.md y corridas.tsv existen y son byte a byte los del experimento (sha256 coincide con el de la carpeta de pruebas)
El informe recoge criterio fijado antes de medir, cifras por repo y modo, veredicto NO-GO, uso real del indice y factores que contaminan la medida
corridas.tsv lleva un README que explica cada columna y como se midio
docs/INDEX.md enlaza el informe
Las tareas TASK-096..TASK-112 (F10-1..F10-17) ya no existen en tareas/ y ninguna tarea viva depende de ellas
tareas/ROADMAP.md y docs/BOARD.md, regenerados con taskctl, no contienen el hito H10 ni esas tareas
RR-20 y RF-77..RF-84 (requisitos y plan), la seccion Fase 10 del plan, el ADR F1-19 (y su fila en docs/adr/README.md) y docs/plan/AMPLIACION-2026-11-16.md quedan marcados como descartados el 2026-11-16 con enlace al informe
````
