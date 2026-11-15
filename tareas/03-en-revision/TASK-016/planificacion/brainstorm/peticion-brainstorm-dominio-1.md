# Peticion de brainstorm — TASK-016, rol dominio (ronda 1)

- Tarea: TASK-016 — F2-1 Esqueleto: TypeScript, build reproducible
- Tipo: feature · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-15
- Rol: `brainstorm-dominio` — lanzalo con el agente de ese mismo nombre
- Vuelca tu respuesta en: `salida-brainstorm-dominio-1.md`

## Tu pregunta

> ¿Que sabe quien va a usar esto que no se deduce leyendo el codigo?

## Que miras

- El vocabulario real: que nombra cada cosa quien trabaja en este dominio.
- Los invariantes del negocio que ningun tipo del lenguaje impone y que aun asi no se pueden violar.
- Los casos que en el codigo son simetricos y en el negocio no lo son.
- Cuando una solucion correcta en codigo es incorrecta para quien la va a usar.

## Que NO miras

No es una lista de cortesia. Si la ignoras y lo cubres todo "por si acaso", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:

- Donde encaja el cambio (es del rol de arquitectura).
- Por donde se rompe (es del rol de riesgos).
- Como se prueba (es del rol de testing).

## Enunciado de la tarea

### Objetivo

````
Esqueleto: TypeScript, build reproducible (sin `dist` versionado), lint, CI en Linux/macOS/Windows

Tarea F2-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-*.
````

### Criterios de aceptacion

````
CI verde en los tres desde el primer commit
Cumple lo que pide la tarea F2-1 del plan y los requisitos que cita
````

## Como entregas

Escribe en `salida-brainstorm-dominio-1.md`, con estas secciones y en este orden:

- Vocabulario e invariantes que el enfoque tiene que respetar
- Donde una solucion tecnicamente correcta seria incorrecta aqui
- Lo que hace falta preguntarle a una persona antes de implementar

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.
- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.
- Trabajan en paralelo contigo, sin verte: **arquitectura**, **riesgos**, **testing**. No cubras lo suyo — si lo haces, el unificador recibira el mismo punto de vista repetido y lo leera como confirmacion.
