---
id: TASK-148e
titulo: "soporte comun de los tests unitarios (A8-18, A8-20 unitarios) (parte e de TASK-148)"
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
tokens_diseno: 730463
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148d]
---
## Objetivo

Parte e del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `estado::prueba` (con `EnMemoria` traído de `roadmap::tests`, `estado` y `motivo`) y `git::prueba` (`CONFIG`, `entorno`, `sh`), y migra ciclo/operacion, ciclo/liberar, toma/operacion, plan/operacion, plan/aprobar, plan/guardar, revision/operacion, roadmap, siguiente, ciclo/mod, estado/montaje y git/estado.

## Criterios de aceptacion
- [ ] Las funciones de prueba `estado` y `motivo` y el doble `EnMemoria` solo se definen en `estado::prueba`.
- [ ] `fn sh` y el `git init` de prueba de `estado/montaje.rs` y `git/estado.rs` aíslan con `git::prueba::entorno`, y un unitario comprueba que `git config user.email` devuelve el de `CONFIG`.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-18 y A8-20 con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T07:10:49Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T07:14:31Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T07:14:32Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T07:14:30Z | diseno | agente:abefd1a86ce63717f | 30 | 64746 | 652198 | 13489 | 24 | 730463 | - | - |
