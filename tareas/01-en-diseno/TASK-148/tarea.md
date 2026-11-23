---
id: TASK-148
titulo: "Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-diseno
plan_aprobado: false
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2072119
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-23
dependencias: []
---
## Objetivo

Extraer a `crates/tc2/tests/comun/` (integración) y a módulos `prueba` de `src` (tests unitarios) el soporte de tests que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiado entre dueños: 9 casos IMPORTANTE (siete `divergente`, contando A3-06) de los grupos lanzador-tests, servidor-http-test, instantanea-de-carpeta, soporte-tests-unitarios y dobles-plataforma. La tarea también cierra las fichas fusionadas en ellos (A3-06 en parte, A4-09, A5-07, A7-06, A2-06). Toca los tests de A1 a A8. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion
- [ ] A8-02 (`divergente`, va primero): un solo lanzador de `CARGO_BIN_EXE_tc2` en `tests/comun/mod.rs` quita del entorno los mismos tokens (`TC2_TOKEN_PERSONA`, `TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR`) en todos los ficheros de test. `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo lo da en `tests/comun/`.
- [ ] A8-11 (`divergente`): `tests/identidad.rs:33-98` y `tests/init.rs:31-105` usan `comun::http_mock::Mock`, y no queda ningún `Servidor::nuevo` propio en `crates/tc2/tests`.
- [ ] A8-12 (`divergente`, absorbe A7-06): una sola instantánea de carpeta en `tests/comun/` que anota carpetas y recorre subcarpetas reemplaza las seis de `tests/init.rs:211-232`, `tests/desinstalar.rs:37-62`, `tests/importar.rs:382-396`, `tests/tablero.rs:303-330`, `tests/frescura.rs:317-347` y `tests/doctor.rs:971-1000`. Un test de la propia instantánea comprueba que una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto.
- [ ] A8-16 (`divergente`, absorbe A4-09) y A3-06: la operación de prueba `Escribir`, `salida`, `con_fila` y `preparar` (`tests/ciclo.rs:198-222`, `tests/plan.rs:479-503`, `tests/solape.rs:131-155`, `tests/toma.rs:187-211`, `tests/liberar.rs`) viven en `tests/comun/`. `valor` es una sola, con `trim_end` (`tests/importar.rs:36-41` frente a `tests/alta.rs:179-183`), y `como_persona` ~ `hito_como_persona` (`tests/aprobar.rs:191-202` ~ `:385-396`) es una función.
- [ ] A8-20 (`divergente`): el git de sistema para preparar escenarios (`tests/agente_git/mod.rs:40-55`, `tests/comun/mod.rs:60-76`, `ciclo/mod.rs:473-485`, `estado/montaje.rs:336-356`) aísla la configuración del usuario de la misma forma en todos los sitios, con un ayudante para integración y otro `pub(crate)` para los unitarios.
- [ ] A8-05 (`divergente`, absorbe A2-06): un doble nulo de `Plataforma`, con un comportamiento único si se llama a un método que no se usa, sustituye a los cinco de `identidad.rs:649-679`, `revision/mod.rs:537-568`, `tablero.rs:998-1025`, `tests/revision.rs:68-130` y `tests/tablero.rs`.
- [ ] A8-01 (absorbe A4-09 y parte de A3-06) y A8-18: `entorno_ssh`, `entorno_sin_red`, `identificar`, `git`, `otro_clon`, `editar`, `texto` y `sin_catalogo` (rangos de la ficha A8-01) están en `tests/comun/`. `estado` + `motivo` de los tests unitarios (`ciclo/operacion.rs:297-311`, `ciclo/liberar.rs:439-453`, `toma/operacion.rs:320-334` y los `motivo` de la ficha A8-18) están en un módulo `pub(crate)` de prueba.
- [ ] `cargo test -p tc2 --no-fail-fast` da el mismo número de tests en verde que antes (se pega el recuento antes y después en el `## Resultado`), `cargo xtask verificar` sale en verde y el `## Resultado` lista los 9 IDs con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-23T14:17:53Z | plan | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23T14:26:39Z | diseno | agente:a3d66ffa5f9c04410 | 50 | 121972 | 1919336 | 30761 | 36 | 2072119 | - | - |
