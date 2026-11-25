# Plan — TASK-144: Auditoria 4D: A6 estimacion, progreso, roadmap y tablero (6 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 150 min

Señales: 11 ficheros (nuevo `tests/comun/tareas.rs`; `tests/comun/mod.rs`, `ops.rs`, `remoto.rs`; `tests/siguiente.rs`, `roadmap.rs`, `estimacion.rs`, `progreso.rs`, `solape.rs`; `src/grafo.rs`, `src/progreso/mod.rs`); una capa (solo tests; sin cambios de API ni de producción); 6 criterios (A6-05 ya cumplido); E2E pesados y `cargo xtask verificar`.

**Estado (develop):** A6-04 viva (`tests/siguiente.rs:40-92` ~ `tests/roadmap.rs:417-466`; `TASK-009` y `"Titulo de {id}"` solo en roadmap); A6-02 viva (`src/grafo.rs:641-656`, copia de `roadmap/generar.rs:401`); A6-03 viva (`tests/estimacion.rs:205-260` ~ `tests/progreso.rs:25-80`); **A6-05 resuelta por TASK-148** (`src/siguiente.rs:137` usa `crate::estado::prueba::EnMemoria`); A6-06 viva (`tests/solape.rs:383-404` y `:475-490`); A6-07 viva (`src/progreso/mod.rs:479-497`: el `if buscar(&d).is_none()` salta las aserciones y todas las entradas dan `SIN_TAREA`). A6-01 va en A8-06 (TASK-149).

**Orden:**
1. A6-04 (divergente): `tests/comun/tareas.rs` con `tarea(id, sprint, estado, deps)` (título `"Titulo de {id}"`) y `juego_de_next()` (las 9 tareas y el `ROADMAP.md` con H2 antes que H1); `Fixture::publicar_estado(bare, ficheros) -> String` en `comun/remoto.rs` (init, escribir, add, commit, push, `rev-parse HEAD`). `siguiente.rs` usa `juego_de_next()`; `roadmap.rs` usa `juego_de_next()` y añade `TASK-009` en revisión explícitamente (su comentario describe ese juego). **Cambia solo el lado de siguiente:** títulos de `"t"` a `"Titulo de {id}"`; ninguna aserción mira títulos (`siguiente.rs:105`, `:157-161`).
2. A6-02: `linea_esperada` = `crate::roadmap::generar::ids_de_la_linea(TW_ESPERADO, titulo).unwrap_or_default()`.
3. A6-03: `Poner` a `comun/ops.rs`; `poner` como `Fixture::poner(dir, ficheros)` (precedente `cambiar_estado`); `BASE`, `fila` y `terminada` a `comun/tareas.rs`, con `terminada` parametrizada (`ajustar: impl FnOnce(&mut Tarea)`, `criterios`) porque estimación y progreso necesitan datos distintos (unificarlos cambiaría el histórico de las estimaciones).
4. A6-06: `c5` usa `start_de_luis(&e, "TASK-002")`.
5. A6-07 (D4, test primero): reescribir `c7_fuera_de_un_repo` con `dir()`, `fichero::carpeta`, `fichero::escribir` y `fichero::tests::ejemplo("TASK-001")`: `principal/.git` con un progreso y `orden_estado("no es json", Some(&principal), 0) == linea_en(&principal, 0)` y `!= linea::SIN_TAREA`; el `if buscar(&d).is_none()` pasa a `assert!` con mensaje (como el E2E `r1_01`). Verde hoy, rojo con el mutante de la ficha (`cwd.filter(|_| false)` en `mod.rs:206`) y con el control (`None => String::new()`).
6. A6-05: no se toca; `## Resultado` cita el commit de TASK-148.

## Reutilizacion (Regla 4D)

Lo que ya existe y se usa (helpers, funciones, fixtures, tests), buscado en el repo antes de disenar; y lo que se crea nuevo, con el motivo de no reutilizar:

- `Fixture::transaccion` (base de `poner`), el precedente `Fixture::cambiar_estado` con `ops::Escribir`, `comun/ops.rs`, `comun::escribir`, `Fixture::crudo`, `ids_de_la_linea` (pública), `start_de_luis`, `estado::prueba::EnMemoria`, y en `progreso::tests` `dir()`, `fichero::tests::ejemplo`, `fichero::{carpeta, escribir}`.
- Nuevo: `comun/tareas.rs` (ningún módulo de `comun` fabrica `tarea.md`); `Fixture::publicar_estado` (nada publica la rama de estado desde un repo aparte; `transaccion` pasa por `tc2`). La misma publicación aparece en `tests/doctor.rs:40`, `recursos.rs:121`, `tablero.rs:70` y `:584` (decisión 2).

## Lo que el rol no cubrio

- Riesgos: modos de fallo de los E2E al mover los ayudantes (orden de inicialización, nombre de la carpeta `estado`).
- Testing: solo el mínimo por criterio.
- Dominio: si el test reforzado de A6-07 refleja la adenda de TASK-084 (F1-17); si el título importa en alguna salida de `next`/`roadmap` no leída; si `tablero.rs:584` es la misma mecánica.

## Riesgos aceptados y que los contiene

- `grafo::tests` depende de `roadmap::generar` (solo `cfg(test)`, función ya pública, sin ciclo en producción).
- `comun` puede crecer con ayudantes sin uso (el `dead_code` los taparía): lo contienen la revisión y el `## Resultado`.
- El `assert!` de c7 falla si el temporal está dentro de un repo: buscado (hoy pasa sin probar nada).
- Cambia el título de las tareas del juego de siguiente: los E2E comparan IDs y claves.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A6-04: `--test siguiente` y `--test roadmap` (pesados).
- A6-02: `--lib grafo::tests::toolsaweb_listas_y_en_marcha_como_su_roadmap`.
- A6-03: `--test estimacion` y `--test progreso` (pesados).
- A6-05 y A6-06: `--lib siguiente` y `--test solape c5_sin_red_y_sin_origin_sin_claves`.
- A6-07: `--lib progreso::tests::c7_fuera_de_un_repo` verde, rojo con el mutante y rojo con el control.
- Final: `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar`, sin solapar.

## Lo que necesita decision de una persona

1. Dar A6-05 por cumplida por TASK-148 (la fuente única es `estado::prueba`), citando su commit. Recomendado.
2. Migrar a `Fixture::publicar_estado` también las copias de `doctor`, `recursos` y `tablero` que sean la misma mecánica. Recomendado.
3. c7 falla si el temporal está dentro de un repo. Recomendado.
4. Unificar el título a `"Titulo de {id}"`. Recomendado.

**Respuestas de la persona (Carlos, consigna del 2026-11-23: modo automatico con la opcion recomendada):** se aplican las recomendaciones 1 a 4.

**Regla de una hora:** 150 min, sin trocear: Carlos descartó trocear las tareas de corrección 4D (consigna del 2026-11-23).
