# Peticion al unificador — TASK-095 (ronda 1)

- Tarea: TASK-095 — F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda
- Tipo: feature · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-16
- Rol: `brainstorm-unificador` — lanzalo con el agente de ese mismo nombre
- Vuelca el resultado en: `../plan-final.md`

## Salidas que tienes que consolidar

- `salida-brainstorm-arquitectura-1.md` — rol arquitectura
- `salida-brainstorm-riesgos-1.md` — rol riesgos

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **no declarada (decide la heuristica)**
- Heuristica (4 puntos): **media**
- Senales encontradas:
  - dependencias: 4 dependencias (TASK-002, TASK-010, TASK-012, TASK-014) (+4)

Los dos niveles coinciden. Se lanzan 2 roles.

## Como consolidas

1. Lee las salidas de arriba. **Si alguna falta o esta sin rellenar, sigue adelante con las que haya y escribe en el plan cual falto**: un plan con un punto de vista menos, dicho, vale mas que un plan que finge estar completo.
2. **No promedies.** Donde dos roles discrepen, el plan dice quien propone que, cual gana y por que. Un desacuerdo resuelto con una frase intermedia que no defiende nadie es la peor salida posible de este paso.
3. **Si no discrepan en nada, eso es la alarma, no la nota de calidad**: significa que los roles recibieron el mismo contexto o que alguno no hizo su trabajo. Dilo en el plan.
4. Cada afirmacion del plan que venga de un rol se atribuye a ese rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Los desacuerdos entre roles y como se resuelve cada uno.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

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
