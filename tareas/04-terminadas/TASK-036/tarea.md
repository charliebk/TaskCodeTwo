---
id: TASK-036
titulo: "F4-1 plan con brainstorm de roles"
tipo: feature
sprint: 4
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-036-f4-1-plan-con-brainstorm-de-roles
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 2363277
tokens_implementacion: 34633956
tokens_revision: 8060109
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-029]
---
## Objetivo

`plan` con brainstorm de roles (arquitectura, riesgos, testing, dominio, rendimiento) y unificador, proporcional al carril (RR-02)

Tarea F4-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F3-2.

## Criterios de aceptacion
- [x] Mismos roles que la 0.16
- [x] uno en fix, todos en riesgo alto
- [x] Cumple lo que pide la tarea F4-1 del plan y los requisitos que cita

## Resultado

Hecho: `tc2 plan TASK-NNN [--carril c]` y `tc2 plan guardar TASK-NNN <destino> <fichero>` (modulo `plan/`), con el brainstorm de la 0.16.2 portado: los 5 roles y el unificador con sus textos literales (`plan/roles.rs`), la heuristica de complejidad como constantes (`plan/heuristica.rs`) y las plantillas de las peticiones y del esqueleto de `plan-final.md` (`plan/plantillas.rs`). `carril.rs` gana `Carril::declarado` y `Carril::roles_diseno`: 1 rol en fix (D2a), los 4 de base en riesgo alto con rendimiento por etiqueta (D1a), y `politica_carriles` solo sube. `--carril` solo sube por encima del declarado y queda congelado (D4a); complejidad = la mayor de declarada y heuristica (D5a). Publica por `estado::transaccion` en `TASK-NNN/planificacion/` de la rama de estado: deja la tarea `en-diseno` con carril y modo congelados y la fila de transicion; repetir da `sin-cambios`; no pisa un `plan-final.md` existente. `plan guardar` (D3a) publica `plan-final.md` o `brainstorm/salida-<rol>-<n>.md` desde un fichero, solo `en-diseno` y sin aprobar (con `tareas/` sucio toda transaccion se rechaza, asi que el agente no puede escribir directamente ahi como en la 0.16). Adenda en F1-10 con las divergencias (tope de hotfix antes de `politica_carriles`; con `roles_diseno: 5` el quinto es rendimiento; las rondas solo se anaden y una nueva solo se abre si cambia el juego de roles; sin catalogo para `Planificar`/`Guardar`, que exigen red). `cargo xtask verificar --perfil rapida` en verde.

Implementado por un subagente con el plan aprobado (D1a-D5a, consigna de Carlos para el H4); revisado por dos agentes independientes (uno ejercito el CLI real).

Pendiente para TASK-037 (`approve`): re-planificar con los mismos roles tras «pedir cambios» da `sin-cambios` con las peticiones antiguas; `approve` tendra que cerrar la ronda. No verificado: que los agentes `brainstorm-*` esten instalados (inventario de TASK-080).

Revision ronda 1: aprobada con correcciones por los dos, sin CRITICOS ni IMPORTANTES. MENORES corregidos en d8d4190 y verificados con `verificar-menores`: `plan` rechaza una tarea con `plan_aprobado: true` como ya hacia `guardar` (MEN-1 de los dos); el diagnostico de `plan guardar` sanea la ruta y el error (MEN-2 rust); tests que fallan al quitar `pausada`, `de-otra-persona` y `plan-aprobado` de `guardar`, el orden del tope de hotfix y el fichero vacio, no UTF-8 o grande (MEN-2 calidad); el uso ya no anuncia `--carril errata` (MEN-3); textos de las peticiones precisados (MEN-4).

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T02:39:23Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T02:48:23Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T02:48:25Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T04:03:11Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T04:59:02Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T02:48:22Z | diseno | agente:a50215938700c6e28 | 56 | 125320 | 2216643 | 21258 | 51 | 2363277 | - | - |
| 2026-11-19T04:03:11Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 24 | 19454 | 7033324 | 5613 | 0 | 7058415 | - | - |
| 2026-11-19T04:03:11Z | implementacion | agente:a9749980ff28e205b | 194 | 373874 | 27097544 | 103929 | 22 | 27575541 | - | - |
| 2026-11-19T04:15:33Z | revision | agente:a69e520d3b6b35c0b | 32 | 145336 | 1894127 | 13157 | 3 | 2052652 | 1 | rust-wasm-reviewer |
| 2026-11-19T04:15:33Z | revision | agente:ac4f87651e9a786a1 | 72 | 208577 | 5770728 | 28080 | 15 | 6007457 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — dbe7b5f
- code-quality-reviewer — dbe7b5f

## Verificacion ligera

- 2026-11-19T04:29:04Z: ronda 1; commit revisado 6ecf89825e4a41e13abdd7f70a2fa08bcbf65ff6; commit verificado d8d4190d8019ffa47f6bc2d512830221e71460fb; ficheros: crates/tc2/src/i18n/catalogo/plan.rs, crates/tc2/src/plan/mod.rs, crates/tc2/src/plan/operacion.rs, crates/tc2/src/plan/plantillas.rs, crates/tc2/tests/plan.rs; corregidos: MEN-1, MEN-2, MEN-3, MEN-4, MEN-1, MEN-2; pendientes: ninguno

## Duracion

- Estimada: 210 min
- Real (de la rama abierta al cierre con merge, sin pausas): 131 min
- Paso de la hora: 131 min reales frente a 210 estimados
- Cierre (finish): 1607 s, de 2026-11-19T04:32:15Z a 2026-11-19T04:59:02Z
