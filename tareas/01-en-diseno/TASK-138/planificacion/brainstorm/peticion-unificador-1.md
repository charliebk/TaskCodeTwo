# Peticion al unificador — TASK-138 (ronda 1)

- Tarea: TASK-138 — Auditoria 4D de todo el codigo de TaskCodeTwo
- Tipo: feature · Complejidad declarada: media
- Ronda: 1
- Fecha: 2026-11-22
- Rol: `brainstorm-unificador` — lanzalo con el agente de ese mismo nombre
- Vuelca el resultado en: `../plan-final.md`

## Salidas que tienes que consolidar

- `salida-brainstorm-arquitectura-1.md` — rol arquitectura
- `salida-brainstorm-riesgos-1.md` — rol riesgos

## Complejidad: declarada frente a heuristica

- Declarada en la tarea: **media**
- Heuristica (0 puntos): **trivial**
- Senales encontradas:
  (ninguna senal encontrada — el texto de la tarea no aporta ninguna)

Los dos niveles NO coinciden. Se lanzan 2 roles, que es el mayor de los dos numeros: infraestimar sale caro y sobreestimar es barato. La discrepancia se te senala aqui a proposito — puede significar que el enunciado de la tarea no refleja lo que de verdad cuesta.

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
Auditar todo el codigo existente de TaskCodeTwo (crates/tc2/src, crates/tc2/tests, xtask, crates/distribucion y .github/workflows) contra la regla 4D de Carlos, recomendada por TaskCode 0.17 al actualizar: 1) Duplicacion: dos funciones o tipos que hacen lo mismo con pequenas diferencias. 2) Desviacion: codigo que se sale del objetivo de su tarea o de su ADR (o un criterio dado por cumplido sin estarlo). 3) Diluvio: lineas y tipos de mas donde bastan pocos. 4) Disfraz: codigo o tests que aparentan resolver algo y no lo comprueban. La auditoria solo diagnostica; las correcciones van en tareas de seguimiento.
````

### Criterios de aceptacion

````
docs/auditoria-4d.md lista cada caso con su regla (D1 a D4), fichero:linea, la evidencia que lo prueba (los dos fragmentos duplicados, el mutante que sobrevive, el criterio desviado) y su severidad CRITICO, IMPORTANTE o MENOR
Cubre todos los modulos de crates/tc2/src, los tests, xtask, crates/distribucion y los workflows; el documento lleva una tabla de cobertura por modulo con el numero de casos
Cada caso IMPORTANTE o CRITICO tiene su tarea de seguimiento en tareas/00-planificadas, agrupadas por modulo
Los casos de Disfraz en tests se prueban con un mutante que sobrevive, ejecutado de verdad
````
