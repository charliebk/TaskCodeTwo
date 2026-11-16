# Peticion de redaccion del plan — TASK-116 (ronda 1)

- Tarea: TASK-116 — Nombres de la 2.0 alineados con F1-12 D1a
- Tipo: fix · Complejidad declarada: simple
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
- Heuristica (2 puntos): **simple**
- Senales encontradas:
  - etiquetas_adicionales: 2 etiquetas (1 mas alla de la primera) (+1)
  - criterios_aceptacion: 5 criterios (umbral: 5) (+1)

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
El codigo de tc2 usa nombres que chocan con la 0.16 y contradicen F1-12 D1a (taskcode/tareas, .git/taskcode-estado.lock, .git/tc2/, .taskcode/worktrees). Alinearlos con D1a para que la 2.0 y la 0.16 no compartan nada y tc2 desinstalar (F2-14) pueda reconocer lo suyo por nombre.
````

### Criterios de aceptacion

````
La rama de estado es taskcodetwo/tareas (estado::RAMA) y los tests de estado y alta pasan con ese nombre
El cerrojo, la cache de identidad y lo demas que tc2 guarda en el directorio comun de git viven bajo .git/taskcodetwo/ (grep sin taskcode-estado.lock ni .git/tc2 en crates/tc2/src)
La carpeta de worktrees de tarea no coincide con ninguna ruta de la 0.16 (constante en git/mod.rs y test)
Un repo con huella de la 0.16 (.git/taskcode/, tareas/ en develop) queda intacto tras new/start de la 2.0 (test)
cargo xtask verificar --perfil rapida en verde
````
