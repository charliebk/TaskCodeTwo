# Peticion de redaccion del plan — TASK-139 (ronda 1)

- Tarea: TASK-139 — Auditoria 4D: A1 plataforma e identidad (11 IMPORTANTE)
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
- Heuristica (1 puntos): **simple**
- Senales encontradas:
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
Corregir los 11 casos IMPORTANTE del grupo A1 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 6 Duplicaciones (una `divergente`) y 5 Disfraces en `crates/tc2/src/plataforma/` y sus tests. Cada caso tiene su ficha con evidencia literal y, para los D4 de test, el diff del mutante que hoy sobrevive. Los MENOR del grupo (A1-12) no se piden. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A1-02 (`divergente`, va primero): la lógica neutral de `abrir-pr` de `plataforma/github.rs:434` ~ `plataforma/gitlab.rs:561` queda en una sola función, y la carrera sin PR que releer da la misma línea byte a byte en los dos adaptadores (RR-15). Lo prueba un test nuevo con el mock para las dos plataformas.
A1-01 y A1-03: la lógica neutral de `leer_puertas_de` (`github.rs:742` ~ `gitlab.rs:864`) y las reglas de `prs-abiertos` (`github.rs:412` ~ `gitlab.rs:538`) viven una sola vez fuera de los adaptadores. Cada adaptador conserva solo la lectura de su API (F1-13).
A1-04 y A1-05: el bucle de la tabla de errores HTTP y el servido de fixtures (`tests/plataforma.rs:468` ~ `tests/plataforma_gitlab.rs:476`) y `adaptador` ~ `con_personas` (`tests/plataforma.rs:32` ~ `:568`) quedan en un ayudante común de los tests de plataforma.
A1-06: el cliente HTTP del arnés E2E (`tests/plataforma_e2e.rs:74` ~ `tests/plataforma_gitlab_e2e.rs:83`) queda en un solo módulo compartido por los dos E2E. Los dos E2E siguen compilando con `cargo test -p tc2 --test plataforma_e2e --no-run` y `--test plataforma_gitlab_e2e --no-run`.
A1-07: `rulesets` (`github.rs:960`) pagina con `leer_paginas`, o se funde con `rulesets_de`, y conserva el `None` del 403 de plan. Lo prueba un test nuevo con 100 rulesets y `Link rel=next` cuya página 2 trae `taskcodetwo-develop`: `leer_puertas` lo devuelve.
A1-08 y A1-11: con el diff del mutante de cada ficha aplicado, el test nombrado sale en rojo. A1-08 es la regla de host que empieza por `.` o `-` en `http.rs:100`, con `cargo test -p tc2 --lib plataforma::http::tests::bases_validas_e_invalidas`. A1-11 es el 403 con `retry-after` y sin `x-ratelimit-remaining` de `errores.rs:57`, con `cargo test -p tc2 --test plataforma errores_dirigidos_a_la_persona`. Hoy los dos salen en verde.
A1-09 y A1-10: con los mutantes de las fichas aplicados, los tests nombrados salen en rojo. A1-09 son las reglas de GitLab de `gitlab.rs:546`, con `cargo test -p tc2 --test prs_abiertos`. A1-10 es la comprobación de rol de `gitlab.rs:1251` con un token de revisor configurado, con `cargo test -p tc2 --test plataforma_gitlab protecciones_de_gitlab`.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 11 IDs con su commit.
````
