# Peticion al unificador — TASK-001 (ronda 1)

- Tarea: TASK-001 — F1-0 Crear TaskCodeTwo
- Tipo: feature · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-14
- Rol: `brainstorm-unificador` — lanzalo con el agente de ese mismo nombre
- Vuelca el resultado en: `../plan-final.md`

## Salidas que tienes que consolidar

(ninguna: esta tarea no lanza brainstorm, ver el bloque de complejidad)

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **no declarada (decide la heuristica)**
- Heuristica (0 puntos): **trivial**
- Senales encontradas:
  (ninguna senal encontrada — el texto de la tarea no aporta ninguna)

Los dos niveles coinciden. Se lanzan ningun rol.

## Como consolidas

No hay salidas de brainstorm que consolidar: redacta el plan directamente a partir del enunciado. Deja dicho en el plan que se redacto sin brainstorm y por que (el numero de roles sale del lookup de complejidad, no de un descuido).

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Enunciado de la tarea

### Objetivo

````
Crear TaskCodeTwo: repo en GitHub, `develop` y `main`, TaskCode 0.16 instalado, `tareas/`, config, CI mínima; importar este plan con `taskctl import` (RR-14)

Tarea F1-0 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: —.
````

### Criterios de aceptacion

````
`taskctl doctor` en verde
las tareas F1..F9 planificadas con dependencias
Cumple lo que pide la tarea F1-0 del plan y los requisitos que cita
````
