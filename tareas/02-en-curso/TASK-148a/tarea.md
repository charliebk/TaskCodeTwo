---
id: TASK-148a
titulo: "lanzador del binario en tests/comun (A8-02, absorbe A5-07) (parte a de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
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

Parte a del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/lanzador.rs` (`BINARIO`, `tc2`, `texto`) con `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `TC2_IDIOMA=es` y sin tokens ni `TC2_APP_*`, y migra los 26 ficheros que lanzan `CARGO_BIN_EXE_tc2`.

Objetivo de la tarea entera:

> Extraer a `crates/tc2/tests/comun/` (integración) y a módulos `prueba` de `src` (tests unitarios) el soporte de tests que la auditoría 4D (`docs/auditoria-4d.md`, commit auditado d82ea9d) encontró copiado entre dueños: 9 casos IMPORTANTE (siete `divergente`, contando A3-06) de los grupos lanzador-tests, servidor-http-test, instantanea-de-carpeta, soporte-tests-unitarios y dobles-plataforma. La tarea también cierra las fichas fusionadas en ellos (A3-06 en parte, A4-09, A5-07, A7-06, A2-06). Toca los tests de A1 a A8. No depende de ninguna tarea ni reordena la cadena H4 ni TASK-040.

## Criterios de aceptacion

- [ ] `git grep -n 'CARGO_BIN_EXE_tc2' -- crates/tc2/tests` solo devuelve ficheros de `tests/comun/`.
- [ ] Un test de `tests/soporte.rs` comprueba que `comun::tc2(dir)` quita `TC2_TOKEN_PERSONA`, `TC2_TOKEN_AGENTE`, `TC2_TOKEN_REVISOR` y las variables `TC2_APP_*`, y fija `TC2_IDIOMA=es`.
- [ ] `cargo test -p tc2 --no-fail-fast` da el recuento de antes más los tests nuevos y las copias del test de `http_mock` justificadas, y `cargo xtask verificar` sale en verde; el `## Resultado` lista A8-02 y A5-07 con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-23T14:17:53Z | plan | automatico | persona | 0.17.1 |
| 2026-11-23T14:29:33Z | approve | automatico | persona | 0.17.1 |
| 2026-11-23T14:29:41Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-23T14:26:39Z | diseno | agente:a3d66ffa5f9c04410 | 50 | 121972 | 1919336 | 30761 | 36 | 2072119 | - | - |
