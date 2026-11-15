# Peticion de redaccion del plan — TASK-074 (ronda 1)

- Tarea: TASK-074 — F1-16 ADR lotes desatendidos con revision humana al final
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
  - dependencias: 2 dependencias (TASK-006, TASK-011) (+2)

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
ADR: lotes desatendidos (RR-18): rama de lote, política de carriles, qué para el lote y qué aparta una tarea, presupuesto, reanudación, informe y su equivalente en GitLab

Tarea F1-16 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-5, F1-10.

Contexto (Carlos, 2026-11-14, RR-18, imprescindible): hoy se hace por instrucción al agente («arrancar consecutivamente TASK-xxx a TASK-yyy, sin trocear y aceptando las recomendaciones en todas las preguntas») en ToolsaWeb, OpenDockTrading y TaskCode, para listas de documentación o programación que no necesitan revisión humana hasta el final, o para dejar trabajo por la noche. Diseño propuesto (P13 del plan): rama `lote/<nombre>` desde develop; cada tarea pasa sus puertas automáticas y se integra en la rama del lote (la siguiente ve el código de la anterior); al final un único PR del lote contra develop que aprueba la persona. Solo carriles permitidos; una tarea en rojo o con un CRÍTICO/IMPORTANTE sin resolver se aparta con sus dependientes y el lote sigue; nunca se integra nada en rojo. Presupuesto de tiempo y tokens; `--reanudar` sin repetir; informe de la mañana (terminadas, apartadas y por qué, hallazgos sin corregir, tokens, tiempo). Lo ejecuta el CLI de forma determinista, no el agente. Detalle completo: docs/plan/AMPLIACION-2026-11-14.md §5 bis.
````

### Criterios de aceptacion

````
Diseño probado a mano con un lote real de ToolsaWeb
Cumple lo que pide la tarea F1-16 del plan y los requisitos que cita
````
