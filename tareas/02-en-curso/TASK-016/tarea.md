---
id: TASK-016
titulo: "F2-1 Esqueleto: Rust, build reproducible"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: en-curso
plan_aprobado: true
rama: feature/task-016-f2-1-esqueleto-typescript-build-reproduc
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 1929241
tokens_implementacion: null
tokens_revision: null
creado: 2026-11-14
actualizado: 2026-11-15
dependencias: [TASK-001, TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008, TASK-009, TASK-010, TASK-011, TASK-012, TASK-013, TASK-014, TASK-015, TASK-073, TASK-074, TASK-075, TASK-094]
---
## Objetivo

Esqueleto en Rust (ADR F1-14, divergencia 1; el titulo original decia TypeScript): `Cargo.lock` versionado, sin binarios en el repo, `/Brepro` en Windows, `cargo fmt` y `clippy`, CI en Linux/macOS/Windows

Tarea F2-1 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-*.

## Criterios de aceptacion
- [ ] CI verde en los tres desde el primer commit: `gh run view <id> --json jobs` da 4 jobs `success` (`comprobar-repo` + `codigo` en ubuntu/macos/windows-latest) en el SHA del primer commit de codigo y en el merge en develop
- [ ] Cumple lo que pide la tarea F2-1 del plan (corregida a Rust) y los requisitos que cita: `cargo xtask verificar --perfil rapida` en verde y `tc2 --version` probado en `crates/tc2/tests/`
- [ ] Dos builds en limpio con el mismo sha256 en Windows (x86_64-pc-windows-msvc), en el job `codigo` de windows-latest (`cargo xtask verificar --perfil completa`); Linux y macOS no se miden (decision de la persona, 2026-11-15)

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-15T11:34:39Z | plan | automatico | persona | 0.16.1 |
| 2026-11-15T11:43:33Z | approve | automatico | persona | 0.16.1 |
| 2026-11-15T11:43:56Z | start | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-15T11:43:07Z | diseno | agente:a8a288a8c42a48bf7,ab502d31b1d6417c5,ada0a4fc8284f4f32,a3db606a8a071c5fd,a73d9aa8fad8d8075 | 94 | 264793 | 1621617 | 42737 | 66 | 1929241 | - | - |

## Autorrevision

- rust-wasm-reviewer — e904ef8
- code-quality-reviewer — e904ef8
