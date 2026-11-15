---
id: TASK-019
titulo: "F2-4 Reserva de IDs por push atomico"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-019-f2-4-reserva-de-ids-por-push-atomico
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 457193
tokens_implementacion: 4012088
tokens_revision: 3859867
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-018]
---
## Objetivo

Reserva de IDs por push atómico

Tarea F2-4 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F2-3.

## Criterios de aceptacion
- [x] Dos clones crean tarea a la vez → IDs distintos
- [x] test E2E
- [x] Cumple lo que pide la tarea F2-4 del plan y los requisitos que cita

## Resultado

**Implementado** (en Rust, sin dependencias nuevas):

- `crates/tc2/src/alta.rs`: la operación `Alta` sobre la transacción de F2-3.
  - Calcula el ID sobre el estado fresco de cada intento: el máximo de las carpetas `TASK-<cifras>` más uno. Cuentan también las terminadas y las descartadas, y los huecos no se rellenan (F1-2 D4a, F3-7).
  - Escribe `TASK-NNN/tarea.md` con la plantilla del ADR F1-9, `rama: null` hasta F2-4b.
  - Tiene `exige_red=true`: sin red da `sinred pendiente=no`.
- `crates/tc2/src/lib.rs`: orden `tc2 new [--json] "<titulo>"`, que sale `new ok id=TASK-NNN ...`. Un título vacío, con caracteres de control o con U+2028/U+2029 da `uso` (2).
- **Fallo de F2-3 destapado por el E2E**: dos clones que crean a la vez la rama de estado inexistente recibían `[remote rejected] (reference already exists)`, que salía como un falso `denegada` (8). Ahora es contención y se re-ejecuta (`git/remoto.rs`, `es_contencion`). Anotado como adenda F2-4 en el ADR F1-4.

**Divergencias con el plan**: los E2E están en `crates/tc2/tests/alta.rs` y no en `alta_concurrencia.rs`.

**Pruebas**:

- E2E con el binario real:
  - dos clones a la vez, 3 altas cada uno: salen `TASK-001`..`TASK-006` consecutivas, el ID de cada salida coincide con lo publicado, 0 merges y 0 refs propias;
  - un segundo clon sin `fetch` recibe `TASK-002`;
  - sin red sale 4 sin tocar refs.
- Unitarios de `siguiente_id`, plantilla y escape, fechas y validación del título.
- 5 de 5 ejecuciones del E2E en verde. `cargo xtask verificar --perfil completa` en verde y build reproducible (Windows).

**Revisión** (ronda 1, 2 revisores):

- Resultado: 0 CRÍTICO, 0 IMPORTANTE, 4 MENOR, todos corregidos (b82ba1b).
  - MEN-1: el prefijo `TASK-` se compara sin mayúsculas, como `choque`.
  - MEN-2 y MENOR-2: mensajes de uso, y `new a b` da `argumentos-de-mas`.
  - MENOR-1: test de U+2029.
- Lo que reprodujeron los revisores:
  - 4 clones × 3 altas y 3 clones × 2 × 3 rondas, sin repetidos;
  - la rama de estado inexistente se crea sola;
  - `tareas/` con cambios no se pisa (`tareas=con-cambios`);
  - mutantes muertos, incluido quitar `reference already exists`.

**Sin verificar**: altas simultáneas contra GitHub o GitLab y desde máquinas distintas (F9-4); fecha `creado` en UTC hasta F2-5.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T15:00:18Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T15:04:10Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T15:04:12Z | start | automatico | persona | 0.16.1 |
| 2026-11-15T15:13:06Z | review | automatico | persona | 0.16.1 |
| 2026-11-15T15:26:52Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T15:04:10Z | diseno | agente:a3b86e10be5377cdf | 24 | 57311 | 387358 | 12500 | 22 | 457193 | - | - |
| 2026-11-15T15:13:06Z | implementacion | sesion:0c02a589-064f-41b2-898e-aecc1141ae21 | 46 | 55785 | 3936888 | 19369 | 12 | 4012088 | - | - |
| 2026-11-15T15:21:52Z | revision | agente:aee0dbc80a635524c | 26 | 132947 | 1365623 | 12761 | 2 | 1511357 | 1 | rust-wasm-reviewer |
| 2026-11-15T15:21:52Z | revision | agente:acda122e8d090f708 | 42 | 127883 | 2206439 | 14146 | 2 | 2348510 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — c121de2
- code-quality-reviewer — c121de2

## Verificacion ligera

- 2026-11-15T15:24:19Z: ronda 1; commit revisado f7f9446522807d5cea787a20dc1286abdf525a6d; commit verificado b82ba1bf6d54241566e624ea15d0ddf6b152c3a0; ficheros: crates/tc2/src/alta.rs, crates/tc2/src/lib.rs; corregidos: MENOR-1, MENOR-2, MEN-1, MEN-2; pendientes: ninguno

## Duracion

- Estimada: 60 min
- Real (de la rama abierta al cierre con merge, sin pausas): 23 min
- Cierre (finish): 130 s, de 2026-11-15T15:24:42Z a 2026-11-15T15:26:52Z
