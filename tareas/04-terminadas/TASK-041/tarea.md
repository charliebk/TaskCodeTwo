---
id: TASK-041
titulo: "F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab"
tipo: feature
sprint: 5
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-041-f5-1-workflow-reutilizable-de-github-act
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 771181
tokens_implementacion: 11176940
tokens_revision: 6084340
creado: 2026-11-14
actualizado: 2026-11-19
dependencias: [TASK-008]
---
## Objetivo

Workflow reutilizable de GitHub Actions y plantilla de GitLab CI con perfiles (rápida, impacto, completa, pesada) que llaman al comando del proyecto (RR-15)

Tarea F5-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-7.

## Criterios de aceptacion
- [x] ToolsaWeb y OpenDockTrading lo usan sin scripts nuevos
- [x] la plantilla de GitLab pasa en un proyecto de pruebas
- [x] Cumple lo que pide la tarea F5-1 del plan y los requisitos que cita

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-19T14:53:00Z | plan | automatico | persona | 0.16.1 |
| 2026-11-19T14:58:44Z | approve | automatico | persona | 0.16.1 |
| 2026-11-19T14:58:46Z | start | automatico | persona | 0.16.1 |
| 2026-11-19T15:46:43Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T16:48:47Z | review | automatico | persona | 0.16.1 |
| 2026-11-19T17:29:20Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-19T14:58:43Z | diseno | agente:ade25fedce807b750 | 32 | 66135 | 689594 | 15420 | 29 | 771181 | - | - |
| 2026-11-19T15:46:43Z | implementacion | sesion:ebd8dc9f-6c20-4b9d-bc57-489282394590 | 20 | 14435 | 2585134 | 5525 | 0 | 2605114 | - | - |
| 2026-11-19T15:46:43Z | implementacion | agente:a454c6cbdcabaa5a2 | 114 | 181933 | 8330536 | 59243 | 13 | 8571826 | - | - |
| 2026-11-19T16:53:16Z | revision | agente:a74dc6fa307eb28c7 | 16 | 117239 | 712533 | 9649 | 1 | 839437 | 1 | rust-wasm-reviewer |
| 2026-11-19T16:53:16Z | revision | agente:ae40903db1904b800 | 20 | 126094 | 960325 | 13129 | 4 | 1099568 | 1 | code-quality-reviewer |
| 2026-11-19T16:53:16Z | revision | agente:a896616dc9606b773 | 38 | 142123 | 2117215 | 27025 | 4 | 2286401 | 2 | rust-wasm-reviewer |
| 2026-11-19T16:53:16Z | revision | agente:a5f97a6afdd9245c1 | 34 | 134035 | 1701988 | 22877 | 4 | 1858934 | 2 | code-quality-reviewer |

## Resultado

Workflow reutilizable `.github/workflows/verificacion.yml` (`workflow_call`; jobs `buscar`, `ejecutar` por perfil y `verificacion` publicador sin checkout, que crea el check-run `verificacion` con `GITHUB_TOKEN`) y plantilla `plantillas/gitlab/verificacion.gitlab-ci.yml` (`include: remote` por SHA, `spec:inputs`, un job por perfil elegido con `rules`). Ambos solo llaman al comando del proyecto: entradas `comando` y `argumentos-<perfil>` (por defecto `--perfil <p>`), `runner`/`tags` por perfil, `shell`, `perfiles`. Lint de lista blanca de F1-8 en `xtask/src/workflows.rs` (falla en cerrado; raiz, formas con corchetes, `uses:` sin SHA), dentro de `cargo xtask verificar --perfil rapida`. Guia y divergencias en `docs/ci/verificacion-reutilizable.md`. `.gitattributes` fija LF en `*.yml`.

Evidencia (2026-11-19):
- Verificacion completa en verde sobre 648720a; las correcciones con `cargo test -p xtask` (23/23, tambien en clon fresco de Windows con autocrlf). `finish` vuelve a verificar.
- C1 (D5: demostrado en el repo E2E con stubs del contrato real, sin tocar ToolsaWeb ni OpenDockTrading): stub con las lineas 68-79 de `ToolsaWeb\scripts\verificar.bat` en `windows-latest`/`cmd` en verde (run 44551304303) y forzado a fallar en rojo con el check `verificacion` en `failure` (run 44552422966); stub con la forma de ODT en verde (run 44551304248); `workflow_dispatch` con `completa pesada` en verde (run 44551659290). El check-run `verificacion` sale con `app_id` 15368. Los fragmentos que pondria cada proyecto estan en la guia: un YAML, sin scripts nuevos.
- C2: pipeline real en `taskcodetwo-e2e/taskcodetwo-e2e-gitlab` con la plantilla en 944a974: 3090338668 en verde (`verificacion-rapida`, `--perfil rapida`) y 3090341084 en rojo. El primer intento (3089899840) no corrio: GitLab.com exigia verificar la identidad de la cuenta, que hizo Carlos.
- Ramas de prueba borradas y PR/MR cerrados en los dos repos E2E. Para el E2E se subio a GitHub solo la rama de esta tarea (decision de Carlos, D7).

Revision por pares:
- Ronda 1: rust-wasm-reviewer aprobada-con-correcciones (IMP-1 lector que fallaba en abierto, IMP-2 falsos negativos, IMP-3 CRLF, MEN-1 `uses:` sin SHA); code-quality-reviewer cambios-solicitados (IMPORTANTE-1 CRLF, IMPORTANTE-2 GitLab sin ejecutar y `rules:if` con variable de job, IMPORTANTE-3 concurrencia entre PR, MENOR-1 aviso de interpolacion). Corregido en 944a974 y 5fe9c4f.
- Ronda 2: las dos aprobadas. MENOR nuevos aceptados como seguimiento: push a rama de PR y `pull_request` en grupos de concurrencia distintos; `uses: ./...` local rechazado por no llevar SHA (falso positivo conservador).

Divergencias (en la guia): argumentos por perfil frente a F1-7 decision 1 (D1); `carril`, `revision-ia` y `ci_config_path` fuera de la plantilla, frente a F1-13 (D4); linea de cache de GitLab en artefacto (D3, F5-2); un solo llamador por repo (dos llamadores publican dos check-runs `verificacion`); la concurrencia va por PR o ref.

## Duracion

- Estimada: 100 min
- Real (de la rama abierta al cierre con merge, sin pausas): 151 min
- Paso de la hora: 151 min reales frente a 100 estimados
- Cierre (finish): 2124 s, de 2026-11-19T16:53:56Z a 2026-11-19T17:29:20Z
