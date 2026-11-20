---
id: TASK-043
titulo: "F5-3 Verificacion por impacto como paso de CI y completa en la cola"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-043-f5-3-verificacion-por-impacto-como-paso
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 271058
tokens_implementacion: 12791463
tokens_revision: 1695513
creado: 2026-11-14
actualizado: 2026-11-20
dependencias: [TASK-041]
---
## Objetivo

Verificación por impacto como paso de CI y completa en la cola de merge

Tarea F5-3 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F5-1.

## Criterios de aceptacion
- [x] Tiempo de CI por PR antes y después
- [x] Cumple lo que pide la tarea F5-3 del plan y los requisitos que cita


## Resultado

Verificacion por impacto en el PR y completa en la cola (F5-3), condicional y explicita.
- **Entrada nueva:** `cola-de-merge` (por defecto `false`) en `.github/workflows/verificacion.yml` y en `plantillas/gitlab/verificacion.gitlab-ci.yml`.
- **Con `true`:** un PR o MR que no es la release corre `impacto`; `merge_group` o `merge_train` corre `completa`; la release `develop`->`main` sigue con `completa pesada`.
- **Con `false`:** todo sigue igual que antes, salvo la linea `cola-de-merge=` del resumen.
- **Cache:** la impacto nunca se cachea (no busca candidatos ni escribe linea `arbol=`).
- **Documentacion:** adenda F5-3 en F1-7 (D1c admitida con cola) y tabla de eventos en la guia.
- **Tests:** dos nuevos en `xtask/src/workflows.rs`, uno de ellos ejecuta con bash el `run:` real en 10 casos.

Decisiones de Carlos (2026-11-20): todas las recomendadas (D1 entrada explicita, D2 cola real en una organizacion, D3 sin cache para la impacto, D4 medicion con stub) y sin trocear.

Evidencia (2026-11-20; datos y guiones en `medicion/`):
- **C1, tiempo de CI por PR** (`charliebk/taskcodetwo-e2e-github`, stub con completa 120 s e impacto 20 s):
  - sin cola, PR 17-19: 139, 141 y 144 s, `perfiles=["completa"]` con linea `arbol=`;
  - con cola, PR 20-22: 42, 39 y 38 s, `perfiles=["impacto"]` sin linea `arbol=`;
  - unos 141 s pasan a unos 40 s por PR (-72 %).
  - La ganancia en ToolsaWeb y OpenDockTrading depende de su impacto real: **no medida**.
- **C2 en GitHub:** la release con cola (PR 23) corre `["completa","pesada"]` en 167 s.
- **C2 en GitLab** (`taskcodetwo-e2e/taskcodetwo-e2e-gitlab`, plantilla en f375b60): el MR 13, con cola, corre solo `verificacion-impacto` (pipeline 3096013062); el MR 14, sin cola, `verificacion-completa` (3096013478). La interpolacion del booleano funciona.
- **Tests:** `cargo test -p xtask` 31 en verde y mutaciones del `case`, del filtro de candidatos y del publicador en rojo. Verificacion completa en verde sobre 8cbac85.
- **Limpieza:** los repos E2E quedan limpios (PR y MR cerrados, ramas `med/*` borradas, workflows retirados de `develop`).

**No verificado:**
- **La cola de merge real de GitHub (`merge_group`).** Exige un repo de una organizacion; Carlos acepto crearla (D2), pero no estaba creada al cerrar. Cuando exista, se prueba que el `merge_group` publica `completa` con linea `arbol=` y que la release la reutiliza.
- **Los merge trains de GitLab:** son de pago.

Revision por pares:
- Ronda 1: rust-wasm-reviewer aprobada, sin hallazgos (seis mutaciones del workflow en rojo). code-quality-reviewer aprobada con MEN-1: la tabla de eventos decia «PR a cualquier rama» con cola, contradicha por la fila de release. Corregido en f375b60 y reconocido por `verificar-menores` (7469b1f).
- Queda una rama de prueba de TASK-137 en el repo E2E (`fix/task-137-e2e-informe-origen`, PR 15 cerrado). El modo automatico no deja borrar ramas remotas; la borra Carlos.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-20T21:00:42Z | plan | automatico | persona | 0.16.1 |
| 2026-11-20T21:03:58Z | approve | automatico | persona | 0.16.1 |
| 2026-11-20T21:04:01Z | start | automatico | persona | 0.16.1 |
| 2026-11-20T21:55:01Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-20T21:03:56Z | diseno | agente:a22796b055c926836 | 16 | 47445 | 213456 | 10141 | 14 | 271058 | - | - |
| 2026-11-20T21:55:01Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 24 | 18192 | 4400209 | 7911 | 0 | 4426336 | - | - |
| 2026-11-20T21:55:01Z | implementacion | agente:aee7714df37f141ba | 118 | 183476 | 8138175 | 43358 | 21 | 8365127 | - | - |
| 2026-11-20T22:02:43Z | revision | agente:ac7ad33df9ab486a0 | 12 | 99266 | 452157 | 3091 | 2 | 554526 | 1 | rust-wasm-reviewer |
| 2026-11-20T22:02:43Z | revision | agente:a7da3a26622295eef | 22 | 115866 | 1015977 | 9122 | 2 | 1140987 | 1 | code-quality-reviewer |

## Verificacion ligera

- 2026-11-20T22:02:45Z: ronda 1; commit revisado 23039496d45451cb132c8fd041a42333355bf9a1; commit verificado 99ab51995cc46b7d4f7758fe0e47aa95514447a1; ficheros: docs/ci/verificacion-reutilizable.md; corregidos: MEN-1; pendientes: ninguno
