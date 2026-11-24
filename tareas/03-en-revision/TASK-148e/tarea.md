---
id: TASK-148e
titulo: "soporte comun de los tests unitarios (A8-18, A8-20 unitarios) (parte e de TASK-148)"
tipo: fix
sprint: 0
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: fix/task-148-auditoria-4d-soporte-de-tests-compartido
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 730463
tokens_implementacion: 6194957
tokens_revision: null
creado: 2026-11-23
actualizado: 2026-11-24
dependencias: [TASK-148d]
---
## Objetivo

Parte e del troceo de TASK-148 (Auditoria 4D: soporte de tests compartido en tests/comun y modulos prueba (9 IMPORTANTE)), partes TASK-148a, TASK-148b, TASK-148c, TASK-148d, TASK-148e, TASK-148f en la rama fix/task-148-auditoria-4d-soporte-de-tests-compartido.

Crea `estado::prueba` (con `EnMemoria` traído de `roadmap::tests`, `estado` y `motivo`) y `git::prueba` (`CONFIG`, `entorno`, `sh`), y migra ciclo/operacion, ciclo/liberar, toma/operacion, plan/operacion, plan/aprobar, plan/guardar, revision/operacion, roadmap, siguiente, ciclo/mod, estado/montaje y git/estado.

## Criterios de aceptacion
- [x] Las funciones de prueba `estado` y `motivo` y el doble `EnMemoria` solo se definen en `estado::prueba`.
- [x] `fn sh` y el `git init` de prueba de `estado/montaje.rs` y `git/estado.rs` aíslan con `git::prueba::entorno`, y un unitario comprueba que `git config user.email` devuelve el de `CONFIG`.
- [x] `cargo test -p tc2 --no-fail-fast` y `cargo xtask verificar` en verde; el `## Resultado` lista A8-18 y A8-20 con su commit.

## Resultado

Soporte comun de los tests unitarios de `crates/tc2/src` en dos modulos `#[cfg(test)] pub(crate) mod prueba`, sin cambios de produccion ni de API publica.

- **A8-20** (unitarios): e7abcab. `git::prueba` con `CONFIG`, `entorno`, `aislar`, `sh` y `R`; lo usan `ciclo`, `estado/montaje` (sin sus cuatro `-c`) y el `git init` de `git/estado.rs`.
- **A8-18**: 771e187. `estado::prueba` con `EnMemoria` (y `de_rutas`), `estado`, `motivo` y `texto`; migrados ciclo/operacion, ciclo/liberar, toma/operacion, plan/operacion, plan/aprobar, plan/guardar, revision/operacion, roadmap y siguiente.
- Criterio 1: `git grep -nE "struct EnMemoria|fn estado\(|fn motivo\(r|fn texto\(r" -- crates/tc2/src`: los ayudantes de prueba solo en `estado/prueba.rs` (salen ademas cuatro funciones de produccion con otra firma: `confianza::estado`, `grafo::estado`, `roadmap/generar::estado`, `config/claves::texto`).
- Test nuevo `git::prueba::tests::sh_lee_la_configuracion_aislada`, en rojo con dos mutantes (`aislar` sin `CONFIG`; `sh` sin `entorno`).
- Recuento `cargo test -p tc2 --no-fail-fast` (target propio): antes 779 (cierre de TASK-148d); despues 48 binarios, 780 ok, 0 fallos, 13 ignorados (779 + 1). `--lib`: 397 → 398.
- Pendiente (decision 3, fuera del criterio): `rama_local` y `mktree` de `estado/montaje.rs` siguen con `Command::new("git")` directo, y los `Git::nuevo()` sin entorno de `montaje.rs` (`previsto`, `montar_con`) y `git/estado.rs` (`hashear`, `con_entrada`) leen la configuracion del usuario, como antes.
- `cargo xtask verificar` lo ejecuta `finish`.

## Divergencias

- `R` (el `type R<T>`) tambien pasa a `git::prueba` porque `sh` lo devuelve; `montaje` conserva su `R` local.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-24T07:10:49Z | plan | automatico | persona | 0.17.1 |
| 2026-11-24T07:14:31Z | approve | automatico | persona | 0.17.1 |
| 2026-11-24T07:14:32Z | start | automatico | persona | 0.17.1 |
| 2026-11-24T07:43:57Z | review | automatico | persona | 0.17.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-24T07:14:30Z | diseno | agente:abefd1a86ce63717f | 30 | 64746 | 652198 | 13489 | 24 | 730463 | - | - |
| 2026-11-24T07:43:57Z | implementacion | sesion:f5474097-7166-4a1c-ae42-79e139302dce | 12 | 9850 | 2627405 | 3582 | 0 | 2640849 | - | - |
| 2026-11-24T07:43:57Z | implementacion | agente:a7201cf6b87eed8e4 | 64 | 130725 | 3406703 | 16616 | 7 | 3554108 | - | - |

## Autorrevision

- rust-wasm-reviewer — 771e187
- 4D — 771e187: helper buscado, criterios citados, 1 test mutado
