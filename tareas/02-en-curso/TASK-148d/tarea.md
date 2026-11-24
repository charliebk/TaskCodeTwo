---
id: TASK-148d
titulo: "instantanea de carpeta comun (A8-12, absorbe A7-06) (parte d de TASK-148)"
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
tokens_diseno: 316853
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148c]
---
## Objetivo

Parte d del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `comun/instantanea.rs` (recursiva, anota carpetas, `"<len> <sha256>"`, con lista de exclusión) y migra init, desinstalar, importar, tablero, frescura y doctor.

## Criterios de aceptacion
- [ ] Ninguno de `tests/init.rs`, `tests/desinstalar.rs`, `tests/importar.rs`, `tests/tablero.rs`, `tests/frescura.rs` y `tests/doctor.rs` recorre carpetas por su cuenta: usan `comun::instantanea`.
- [ ] Un test de `tests/soporte.rs` comprueba que una carpeta vacía nueva y un fichero en una subcarpeta cambian la foto, y que un nombre excluido no la cambia.
- [ ] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-12 y A7-06 con su commit.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T05:55:38Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T05:58:24Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T05:58:25Z | start | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T05:58:24Z | diseno | agente:aa5704f48c80de0c1 | 16 | 56118 | 250824 | 9895 | 20 | 316853 | - | - |
