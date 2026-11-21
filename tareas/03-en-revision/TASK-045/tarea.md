---
id: TASK-045
titulo: "F5-5 Runner self-hosted Windows por perfil"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-revision
plan_aprobado: true
rama: feature/task-045-f5-5-runner-self-hosted-windows-por-perf
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 391695
tokens_implementacion: 31339240
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-21
dependencias: [TASK-009, TASK-041]
---
## Objetivo

Runner self-hosted Windows por perfil

Tarea F5-5 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F5-1, F1-8.

## Criterios de aceptacion
- [ ] La pesada de ToolsaWeb corre en la máquina de Carlos
- [x] Cumple lo que pide la tarea F5-5 del plan y los requisitos que cita

## Resultado

Runner propio Windows solo para la pesada de la release (F5-5), con la regla de F1-8 en dos sitios.
- **`buscar`, paso «Runner propio»:** un `runner-<perfil>` con `self-hosted` (o un grupo) solo pasa en un `pull_request` de `develop` a `main` de un repo privado con el head en el mismo repo. Si no, sale con `exit 2`, el motivo (con evento, base y origen) y qué hacer.
- **`ejecutar`, paso «aislamiento del runner propio»:** antes del checkout y solo con `runner.environment == 'self-hosted'`. Repite la regla del evento, que no depende de la etiqueta, y falla si el runner no es Windows, si la cuenta es SYSTEM o Administradores, o si faltan `bash` o `jq`. Corre con `powershell -NoProfile -ExecutionPolicy Bypass`.
- **Lint (`xtask/src/workflows.rs`):** `clean:` distinto de `true` en un checkout es error. Las guardas del aislamiento se exigen enteras y acabadas en `exit 2`, con sus mutaciones. `sha256sum` solo por la entrada estándar.
- **Documentación:** sección «Runner propio (F5-5)» de `docs/ci/verificacion-reutilizable.md` (alta real, etiquetas, retirada, qué está comprobado) y adenda F5-5 en F1-8.

Decisiones de Carlos (2026-11-20): todas las recomendadas, sin trocear; D2a (registra el runner en su máquina, en un repo privado de pruebas).

Evidencia (2026-11-21, repo privado `charliebk/taskcodetwo-e2e-privado`, runner `NULLCAD-e2e` con el usuario `tc2-runner` sin administración):
- **Run 44922050486:** release en verde; la completa en `ubuntu-latest`, la pesada en el runner propio («usuario nullcad\tc2-runner sin administracion»); el stub lee los datos y no puede escribir en ellos («bien: los datos son de solo lectura»).
- **El E2E encontró tres fallos, corregidos con test:** la directiva `AllSigned` bloqueaba el `.ps1` del paso (run 44894375522); faltaba `jq` (44901386742); la huella local salía con `\` delante en Windows y la caché no acertaría nunca (corregido en addfcb2, comprobado en 44922284948).
- **Tras la revisión:** release en verde con la regla en el aislamiento (44926606100). Un `workflow_dispatch` con `runner-pesada: '["windows","tc2-e2e"]'`, sin `self-hosted`, pasa `buscar` y el aislamiento lo para en la máquina antes del checkout (44926601827).
- **Negativo en el repo público** (`taskcodetwo-e2e-github`, run 44831249131): el runner propio se rechaza en `buscar` sin encolar nada.
- **Tests:** `cargo test -p xtask` 34 en verde. Verificación completa en verde sobre addfcb2; la de `finish` cubre el resto.

**Criterio 1, parcial (como dice el plan):** el mecanismo corre de verdad en la máquina de Carlos, pero con un stub en el repo de pruebas; la pesada de ToolsaWeb no, porque ToolsaWeb aún no usa el reutilizable (F9) y aquí no se toca.

**No verificado:** un acierto de la caché por árbol en el runner propio (en el E2E el diff tocaba el workflow y la regla de rutas no reutiliza, por diseño); que `config.cmd` pida la contraseña sin `--windowslogonpassword`.

Revisión por pares:
- **Ronda 1:** rust-wasm-reviewer aprobada con correcciones (MEN-1 corregido; MEN-2 aceptado: la aserción de `sha256sum < "$HUELLA"` cubre el riesgo). code-quality-reviewer pidió cambios por IMPORTANTE-1 (un `runs-on` sin `self-hosted` saltaba la guarda de `buscar`) y MENOR-1 a 5: todo corregido en e141ed1.
- **Ronda 2:** las dos aprobadas. Menores residuales (guardas con `exit 0` sin test, valores vacíos en los mensajes) corregidos en 76daad5 y reconocidos por `verificar-menores` (621db95).

Pendiente de Carlos: retirar el runner con los comandos de la guía si ya no lo quiere, y, si quiere, borrar el repo privado de pruebas y las ramas `worktree-agent-*` subidas por error al remoto de TaskCodeTwo.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-20T22:26:01Z | plan | automatico | persona | 0.16.1 |
| 2026-11-20T22:29:58Z | approve | automatico | persona | 0.16.1 |
| 2026-11-20T22:29:59Z | start | automatico | persona | 0.16.1 |
| 2026-11-21T16:37:34Z | review | automatico | persona | 0.16.1 |
| 2026-11-21T16:50:40Z | review | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-20T22:29:57Z | diseno | agente:a2118c1f6b37e52ec | 18 | 67968 | 307643 | 16066 | 15 | 391695 | - | - |
| 2026-11-21T16:37:34Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 266 | 439131 | 23046412 | 55698 | 15 | 23541507 | - | - |
| 2026-11-21T16:37:34Z | implementacion | agente:acc34b893ea0f56b6 | 112 | 178038 | 7574467 | 45116 | 22 | 7797733 | - | - |

## Verificacion ligera

- 2026-11-21T16:58:07Z: ronda 2; commit revisado e141ed154087ea00e2e7fa9536bed2fb42c093f7; commit verificado 76daad5d9ff5b51e44c69e6991f86b019ee3e191; ficheros: .github/workflows/verificacion.yml, xtask/src/workflows.rs; corregidos: MENOR-6, MEN-1 (ronda 1), MENOR-1 (nuevo, residual de MEN-1), MEN-2 (ronda 1); pendientes: ninguno
