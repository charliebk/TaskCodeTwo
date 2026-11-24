# Peticion de redaccion del plan — TASK-140 (ronda 1)

- Tarea: TASK-140 — Auditoria 4D: A2 hallazgos y revision (7 IMPORTANTE)
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
  - criterios_aceptacion: 7 criterios (umbral: 5) (+1)

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
Corregir los 7 casos IMPORTANTE del grupo A2 de `docs/auditoria-4d.md` (commit auditado d82ea9d): 4 Duplicaciones en `crates/tc2/src/hallazgos/` y 3 Disfraces de test con mutante confirmado. A2-01 y A2-06 se fusionaron en las tareas de abstracción común (A8-07 y A8-05) y aquí no se piden. Tampoco los MENOR (A2-07 a A2-10). No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A2-02 y A2-03: la extracción del id válido y la closure del `Fallo` (`hallazgos/mod.rs:695-704` ~ `:828-837`) y el corte por `TOPE_HALLAZGOS` con su conversión `Fallo -> Rechazo` (`mod.rs:956-977` ~ `:980-1013`) quedan escritos una vez cada uno.
A2-04: la fila Markdown de un hallazgo y sus topes de `plano` (64, 200, 400) están en una sola función, que usan `mod.rs:1193-1204`, `mod.rs:1866-1881` y `ronda.rs:325-337`.
A2-05: los siete analizadores de opciones de `hallazgos/orden.rs` (`:117-139`, `:287-302`, `:435-446`, `:488-511`, `:664-678`, `:717-731`, `:759-798`) usan un analizador común. `uso` y `argumento-desconocido` siguen saliendo igual, y lo comprueba `cargo test -p tc2 --test hallazgos`.
A2-11: hay un test que comprueba el dictamen de `veredicto_ronda` en modo `tope` y en modo `ligera` (`mod.rs:1561-1564`). Con los dos mutantes de la ficha aplicados, `cargo test -p tc2 --lib hallazgos` sale en rojo; hoy sale en verde.
A2-12: hay un test que ejecuta la rama «mezcla con conflictos -> `None`» de `desde_incremental` (`orden.rs:653-657`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --test hallazgos el_diff_incremental_no_arrastra_lo_que_trae_la_base` (o el test nuevo) sale en rojo.
A2-13: `cada_regla_del_cierre_rechaza_con_su_motivo` (o un test nuevo) prueba el tope de cierres `fuera-de-tope` (`mod.rs:982-993`). Con el mutante de la ficha aplicado, `cargo test -p tc2 --lib hallazgos::tests::cada_regla_del_cierre_rechaza_con_su_motivo` sale en rojo.
`cargo xtask verificar` en verde, y el `## Resultado` lista los 7 IDs con su commit.
````
