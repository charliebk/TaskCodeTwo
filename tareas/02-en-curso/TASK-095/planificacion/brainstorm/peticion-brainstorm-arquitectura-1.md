# Peticion de brainstorm — TASK-095, rol arquitectura (ronda 1)

- Tarea: TASK-095 — F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda
- Tipo: feature · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-16
- Rol: `brainstorm-arquitectura` — lanzalo con el agente de ese mismo nombre
- Vuelca tu respuesta en: `salida-brainstorm-arquitectura-1.md`

## Tu pregunta

> Dado lo que ya existe, ¿donde encaja este cambio y que forma tiene?

## Que miras

- Los modulos y ficheros que ya resuelven algo parecido, para extenderlos en vez de duplicarlos.
- Que se crea nuevo, que se extiende y en que orden se construye.
- Los limites que el cambio cruza: contratos publicos, formatos de fichero, esquemas.
- El precedente interno mas cercano: como se resolvio la ultima vez un problema de esta forma.

## Que NO miras

No es una lista de cortesia. Si la ignoras y lo cubres todo "por si acaso", el acotado de contexto se deshace sin que se note y esta tarea deja de tener varios puntos de vista:

- Como se prueba (es del rol de testing).
- Por donde se rompe (es del rol de riesgos).
- Las reglas de negocio (son del rol de dominio).

## Enunciado de la tarea

### Objetivo

````
ADR: memoria del proyecto (RR-20): git como verdad e índice SQLite reconstruible, ubicación del índice, regla de frescura por SHA, esquema, vocabulario y tres capas de etiquetado, conversor de engram (decisión A/B) y retirada de engram del manifiesto

Tarea F1-19 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-1, F1-9, F1-11, F1-14.

Contexto (Carlos, 2026-11-16, RR-20, hito H10): memoria del proyecto propia que sustituye a engram. Git es la fuente de verdad (tareas, etiquetas, ficheros tocados, memoria importada) y una base SQLite local es un índice reconstruible que nunca se versiona y nunca responde con datos viejos: comprueba en cada lectura hasta qué commit está indexado, porque el merge-pr lo hace la persona en la plataforma y ningún hook local se entera. Etiquetado en tres capas con vocabulario cerrado, `tc2 buscar` como única búsqueda del plugin, conversores de tareas y de engram, grafo y visor 3D opcional con dockview. Detalle completo: docs/plan/AMPLIACION-2026-11-16.md.
````

### Criterios de aceptacion

````
Índice de las tareas terminadas de TaskCodeTwo reconstruido dos veces con el mismo hash
Un merge hecho en la plataforma se refleja en la siguiente búsqueda sin hooks
Cumple lo que pide la tarea F1-19 del plan y los requisitos que cita
````

## Como entregas

Escribe en `salida-brainstorm-arquitectura-1.md`, con estas secciones y en este orden:

- Enfoque propuesto, con rutas y nombres concretos
- Que se extiende y que se crea
- Limites que cruza
- La decision de diseño que mas te preocupa (UNA sola)

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es un documento.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.
- Prefiere lo concreto: rutas, nombres y casos reales por encima de recomendaciones genericas.
- Trabajan en paralelo contigo, sin verte: **riesgos**. No cubras lo suyo — si lo haces, el unificador recibira el mismo punto de vista repetido y lo leera como confirmacion.
