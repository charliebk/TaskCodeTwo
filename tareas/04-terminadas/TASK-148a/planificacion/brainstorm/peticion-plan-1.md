# Peticion de redaccion del plan — TASK-148 (ronda 1)

- Tarea: TASK-148 — Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)
- Tipo: fix · Complejidad declarada: no declarada (decide la heuristica)
- Ronda: 1
- Fecha: 2026-11-23
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
Extraer a `crates/tc2/tests/comun/` (integración) y a módulos `prueba` de `src` (tests unitarios) el soporte de tests que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiado entre dueños: 9 casos IMPORTANTE (siete `divergente`, contando A3-06) de los grupos lanzador-tests, servidor-http-test, instantanea-de-carpeta, soporte-tests-unitarios y dobles-plataforma. La tarea también cierra las fichas fusionadas en ellos (A3-06 en parte, A4-09, A5-07, A7-06, A2-06). Toca los tests de A1 a A8. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.
````

### Criterios de aceptacion

````
A8-02 (`divergente`, va primero): un solo lanzador de `CARGO_BIN_EXE_tc2` en `tests/comun/mod.rs` quita del entorno los mismos tokens (`TC2_TOKEN_PERSONA`, `TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR`) en todos los ficheros de test. `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo lo da en `tests/comun/`.
A8-11 (`divergente`): `tests/identidad.rs:33-98` y `tests/init.rs:31-105` usan `comun::http_mock::Mock`, y no queda ningún `Servidor::nuevo` propio en `crates/tc2/tests`.
A8-12 (`divergente`, absorbe A7-06): una sola instantánea de carpeta en `tests/comun/` que anota carpetas y recorre subcarpetas reemplaza las seis de `tests/init.rs:211-232`, `tests/desinstalar.rs:37-62`, `tests/importar.rs:382-396`, `tests/tablero.rs:303-330`, `tests/frescura.rs:317-347` y `tests/doctor.rs:971-1000`. Un test de la propia instantánea comprueba que una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto.
A8-16 (`divergente`, absorbe A4-09) y A3-06: la operación de prueba `Escribir`, `salida`, `con_fila` y `preparar` (`tests/ciclo.rs:198-222`, `tests/plan.rs:479-503`, `tests/solape.rs:131-155`, `tests/toma.rs:187-211`, `tests/liberar.rs`) viven en `tests/comun/`. `valor` es una sola, con `trim_end` (`tests/importar.rs:36-41` frente a `tests/alta.rs:179-183`), y `como_persona` ~ `hito_como_persona` (`tests/aprobar.rs:191-202` ~ `:385-396`) es una función.
A8-20 (`divergente`): el git de sistema para preparar escenarios (`tests/agente_git/mod.rs:40-55`, `tests/comun/mod.rs:60-76`, `ciclo/mod.rs:473-485`, `estado/montaje.rs:336-356`) aísla la configuración del usuario de la misma forma en todos los sitios, con un ayudante para integración y otro `pub(crate)` para los unitarios.
A8-05 (`divergente`, absorbe A2-06): un doble nulo de `Plataforma`, con un comportamiento único si se llama a un método que no se usa, sustituye a los cinco de `identidad.rs:649-679`, `revision/mod.rs:537-568`, `tablero.rs:998-1025`, `tests/revision.rs:68-130` y `tests/tablero.rs`.
A8-01 (absorbe A4-09 y parte de A3-06) y A8-18: `entorno_ssh`, `entorno_sin_red`, `identificar`, `git`, `otro_clon`, `editar`, `texto` y `sin_catalogo` (rangos de la ficha A8-01) están en `tests/comun/`. `estado` + `motivo` de los tests unitarios (`ciclo/operacion.rs:297-311`, `ciclo/liberar.rs:439-453`, `toma/operacion.rs:320-334` y los `motivo` de la ficha A8-18) están en un módulo `pub(crate)` de prueba.
`cargo test -p tc2 --no-fail-fast` da el mismo número de tests en verde que antes (se pega el recuento antes y después en el `## Resultado`), `cargo xtask verificar` sale en verde y el `## Resultado` lista los 9 IDs con su commit.
````
