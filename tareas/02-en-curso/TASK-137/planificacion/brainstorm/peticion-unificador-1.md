# Peticion al unificador — TASK-137 (ronda 1)

- Tarea: TASK-137 — F6-3 seguimiento: revision-ia cita el informe del run de origen cuando la verificacion sale de cache
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-20
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
Cuando verificacion reutiliza un perfil de la cache por arbol (F5-2), su run no sube informe-verificacion-<perfil> y revision-ia.yml (job preparar) lo busca solo en ese run: el check sale con informe=sin-informe y las revisoras van sin la seccion del informe (F5-4). Se cierra el hueco citando el informe del run de origen. Descubierto en el E2E de TASK-049 (run de verificacion 44740343425 reutilizado, run de revision-ia 44740402515 con informe=sin-informe).
````

### Criterios de aceptacion

````
Un PR cuya verificacion sale de cache produce prompts con la seccion del informe del run de origen, probado en charliebk/taskcodetwo-e2e-github (un PR que ejecuta la verificacion y un commit vacio que la reutiliza)
Si el informe del origen ha caducado o no se encuentra, el check lo dice y las revisoras van sin seccion, como hoy
docs/ci/revision-ia.md (Limites conocidos) actualizado
El lint de xtask/src/workflows.rs sigue en verde
````
