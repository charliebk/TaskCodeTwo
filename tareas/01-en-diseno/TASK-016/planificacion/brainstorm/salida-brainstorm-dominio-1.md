# Salida brainstorm dominio — TASK-016 (ronda 1)

> El esqueleto es un CLI en Rust (ADR F1-14; confirmado por la persona).

## Vocabulario e invariantes que el enfoque tiene que respetar

- **Lenguaje Rust**: F1-14 «Decisión» punto 1. El título de la tarea, `docs/BOARD.md` y `docs/plan/PLAN-TASKCODE-2.md:327` siguen diciendo «TypeScript»; F1-14 «Divergencias» punto 1 deja el texto nuevo propuesto.
- **«Sin `dist` versionado»** en Rust: ningún binario en el repo, `target/` ignorado y `Cargo.lock` versionado (F1-14, divergencia 1).
- **«Build reproducible»**: dos builds en limpio con el mismo sha256 por target; en Windows exige `-C link-arg=/Brepro` (PoC B). Solo medido en win-x64, misma máquina y carpeta.
- **«Lint»**: `cargo fmt --check` y `clippy` (F1-14, divergencia 1).
- **Perfil release**: `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `strip = true`, `panic = "abort"` (F1-14 punto 3).
- **Nombres**: CLI `tc2`, paquete/plugin `taskcodetwo` (F1-12 D1a). Nunca `taskctl` (binario de la 0.16, RR-14).
- **Dependencias en ejecución**: solo `git`; nada de Node (F1-4, F1-14 puntos 3 y 4).
- **CI en tres sistemas, no seis**: la matriz de 6 targets, `SHA256SUMS`, firma y releases son de F2-10; workflow reutilizable y caché por árbol de F5-1/F5-2.
- **Lo que ya existe se conserva**: job `comprobar-repo`, `permissions: contents: read`, actions fijadas por SHA (F1-8). `.taskcode/config.yml` dice que el comando de verificación y `rutas_documentacion` «se declaran con la primera tarea de código (F2-1)»: parte de la tarea aunque los criterios no lo nombren.

## Dónde una solución técnicamente correcta sería incorrecta aquí

- Seguir el enunciado al pie de la letra (TypeScript, `dist`, eslint): contradice una decisión aceptada; y sin el criterio «dos builds en limpio con el mismo sha256 por target» la reproducibilidad queda sin comprobar.
- Declarar «reproducible» en Linux y macOS sin compararlo allí.
- Llamar `taskctl` al binario o crate: choca en el PATH con la 0.16.
- Adelantar contratos de otras tareas con forma inventada (`tc2 version --json` de F2-12, lanzador, `cli.lock`). Si el binario muestra la versión, no fijar un JSON distinto de `{"version","protocolo","target","sha256"}` (F1-14 punto 5).
- Entender «verde desde el primer commit» como «verde al final de la rama»: workflow de código y esqueleto que compila llegan juntos en el primer commit de código.
- Declarar la verificación en `.taskcode/config.yml` con el modelo de la 2.0 (`--perfil`, F1-7 D1): ese fichero lo lee la 0.16 con sus propias claves.

## Lo que hace falta preguntarle a una persona antes de implementar

1. ¿Se corrige ya el plan con `taskctl` (título y criterio de F2-1, quizá F2-10..F2-12 con `tc2`) o en el cierre (D9a de F1-14)?
2. ¿Criterio «dos builds en limpio con el mismo sha256 por target»? ¿En los tres sistemas o solo Windows? ¿Mismo runner o ejecuciones distintas?
3. Versión exacta de la toolchain (`rust-toolchain.toml`; la PoC usó Rust 1.100.0), edición y MSRV.
4. Nombre y forma del crate: paquete `taskcodetwo` con binario `tc2`, o workspace.
5. Qué comando de verificación y qué `rutas_documentacion` se declaran en `.taskcode/config.yml`, y qué corre en local y qué solo en CI.
6. Licencia del repo público (campo `license` de `Cargo.toml`, RR-16).
