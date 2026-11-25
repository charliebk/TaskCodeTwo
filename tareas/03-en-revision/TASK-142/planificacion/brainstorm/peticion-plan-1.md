# Peticion de redaccion del plan — TASK-142 (ronda 1)

- Tarea: TASK-142 — Auditoria 4D: A4 git, rama de estado y toma (9 IMPORTANTE)
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-24
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
- Heuristica (3 puntos): **simple**
- Senales encontradas:
  - palabra_alto_riesgo: concurrencia (+2)
  - criterios_aceptacion: 8 criterios (umbral: 5) (+1)

Los dos niveles coinciden. Se lanzan 1 rol.

## Que tiene que traer el plan final

- Enfoque propuesto, concreto: que se crea, que se extiende, en que orden.
- Lo que el rol no cubrio, contrastado contra los criterios de aceptacion.
- Reutilizacion (Regla 4D): lo que ya existe en el repo y se va a usar (helpers, funciones, fixtures, tests), buscado ANTES de proponer nada nuevo; y lo que se crea nuevo, con el motivo de no reutilizar lo que hay. Duplicar un helper que ya existe es un hallazgo IMPORTANTE.
- Riesgos aceptados y que los contiene.
- Plan de pruebas.
- Lo que necesita decision de una persona antes de implementar. **El checkpoint humano es obligatorio**: este plan no vale hasta que alguien lo apruebe con `taskctl approve`.

## Reglas

- **Propones enfoque; no implementas.** No escribas codigo de produccion ni modifiques ficheros del repo: tu salida es el plan.
- Evidencia, no suposicion: si afirmas que algo se comporta de cierta manera, es porque lo has mirado. Di de donde lo sacas.

## Enunciado de la tarea

### Objetivo

````
Corregir los 9 casos IMPORTANTE del grupo A4 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 8 Duplicaciones (dos `divergente`) y 1 Disfraz en código en la guarda de argumentos de git. A4-09 va en la tarea de soporte de tests compartido. No se piden los MENOR (A4-11, A4-12). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A4-01 (`divergente`, va primero): la consulta de ramas de tarea locales y remotas (`git/rama.rs:88-110` ~ `:160-196`) queda escrita una vez. El fallo del remoto se clasifica igual (sinred / error local / denegada) en `git/remoto.rs:175-181` y `git/tag.rs:100-114`, y lo prueba un test con un remoto que deniega y que hoy sale `sinred`.
A4-02 (`divergente`): el envoltorio «lanzar git y traducir el fallo» (`git/estado.rs:243-265`, `:269-298`, `:135-150` y `git/mod.rs:52-89`) es uno solo, y `leer_blobs` usa `primera_linea_error()`.
A4-05 y A4-07: `hay_remoto` es `url_origin(..)?.is_some()` (`git/estado.rs:451-460` ~ `:463-473`), y hay un solo analizador de `git status --porcelain -z` (`git/guardar.rs:158-176` ~ `git/mod.rs:216-228`).
A4-03 y A4-04: las 19 líneas de `intento` y `sin_red` (`estado/transaccion.rs:353-371` ~ `:533-551`) y el informe de un `publicar` fallido (`git/tag.rs:81-91` ~ `git/remoto.rs:516-526`) quedan escritos una vez cada uno.
A4-06: `crear` y `reanudar_paso` del worktree de tarea (`git/rama.rs:318-380` ~ `:388-456`) comparten el arranque y el bloque de carpeta ocupada.
A4-08: los ayudantes `rapida`, `escribir_estado` y `rama_en` y el repo sin remoto (`tests/estado_bordes.rs:19-53` ~ `tests/estado_casos.rs:19-44`, `:124-129` ~ `tests/estado_concurrencia.rs:216-220`) quedan en un módulo común de tests.
A4-10: `es_prohibido` (`git/puerto.rs:613-646`) rechaza las opciones cortas agrupadas que contienen una prohibida, o pasa a una lista permitida como pide F1-4 (decisión 3). `rechaza_cada_argumento_prohibido_sin_lanzar_git` (`puerto.rs:759-846`) incluye `(Push, ["-qf", "origin", "r"])`, `(Branch, ["-qD", "x"])` y `(Worktree, ["remove", "-ff", "w"])`, y sale en rojo con el código de d82ea9d.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 9 IDs con su commit.
````
