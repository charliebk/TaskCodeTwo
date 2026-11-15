# Salida brainstorm arquitectura — TASK-016 (ronda 1)

## Enfoque propuesto, con rutas y nombres concretos

El esqueleto es en **Rust** (ADR F1-14, decisión 1 y divergencia 1; confirmado por la persona). `tarea.md:3` y `PLAN-TASKCODE-2.md:327` siguen diciendo «TypeScript».

- **`Cargo.toml` raíz como workspace virtual**, `members = ["crates/tc2", "xtask"]`: `[workspace.package]` (edición, `rust-version`), `[workspace.lints]` (rustc y clippy; `-D warnings` en CI), `[profile.release]` del punto 3 de F1-14: `opt-level=3`, `lto="thin"`, `codegen-units=1`, `strip=true`, `panic="abort"` (los perfiles solo valen en la raíz).
- **`crates/tc2/`**: paquete `tc2`, `[[bin]] name = "tc2"` (F1-12). `main.rs` mínimo que delega en `lib.rs`. Hoy solo `tc2 --version`, una línea sin ANSI (contrato F1-4). Las capas futuras (`git` F2-2, `yaml` F2-5) entran como módulos de esta lib; crates nuevos solo cuando haga falta.
- **`rust-toolchain.toml`**: versión exacta (1.100.0, la medida en F1-14) con `components = ["rustfmt", "clippy"]`; rustup ya viene en los runners (sin verificar), sin acciones de terceros.
- **`.cargo/config.toml`**: `[target.x86_64-pc-windows-msvc]` y `[target.aarch64-pc-windows-msvc]` con `rustflags = ["-C", "link-arg=/Brepro"]`; `[alias] xtask = "run -p xtask --"`.
- **`Cargo.lock` versionado** y `--locked` en todo. `rustfmt.toml` mínimo.
- **`xtask/`**: único punto de entrada de verificación, `cargo xtask verificar --perfil rapida|completa`. `rapida`: `fmt --check` + `clippy --locked`. `completa`: lo anterior + `test --locked` + reproducibilidad (dos `cargo build --release --locked` en limpio, en dos `--target-dir` distintos, comparando sha256). Adelanta el contrato de F1-7 (D1: un comando con `--perfil`, resultado en el código de salida); F5-1 y F2-10 lo llamarán sin cambiar la forma.
- **`.github/workflows/ci.yml` se amplía**: `comprobar-repo` igual; job nuevo `codigo` con matriz `ubuntu-latest`, `macos-latest`, `windows-latest`, `fail-fast: false`, `permissions: contents: read` (F1-8 D1), `actions/checkout` fijado por SHA, un paso: `cargo xtask verificar --perfil completa`.
- **`.taskcode/config.yml`**: `comando_verificacion` y `rutas_documentacion`, como anuncia su comentario.
- **`.gitignore`**: `/target/`.

## Qué se extiende y qué se crea

- Se extienden: `ci.yml`, `.taskcode/config.yml`, `.gitignore`.
- Se crean: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, `rustfmt.toml`, `crates/tc2/`, `xtask/`.
- `comprobar-repo.sh` no se toca (mirar si sus comprobaciones de enlaces recorren rutas nuevas).
- Orden: 1) workspace, toolchain, lock, perfil, `.gitignore`; 2) `tc2 --version` y lints; 3) `xtask verificar`; 4) job `codigo`; 5) `config.yml` y documentación. «Verde desde el primer commit» obliga a que 1-4 entren juntos.
- Descartado: `scripts/verificar.sh` (H16 cmd/sh, macOS sin `sha256sum`); pasos sueltos en YAML (CI y local comprobarían cosas distintas); workflow aparte (duplica disparadores y `concurrency`).

## Límites que cruza

- **F1-5/F5-1**: el job no debe llamarse `verificacion` (ese check lo define F1-5/F1-7 con su resumen `arbol= perfil= clave=` y caché).
- **F2-10**: perfil y `/Brepro` viven en el repo, no en el YAML. Un `RUSTFLAGS` en el entorno anula en silencio los `rustflags` de `.cargo/config.toml`: xtask debería negarse si está definido.
- **Dependencias**: `xtask` puede necesitar `sha2`; `tc2` empieza sin dependencias.
- **Plan y tarea**: aplicar la divergencia 1 con `taskctl` (D9a de F1-14): título y objetivo a Rust y criterio «dos builds en limpio con el mismo sha256 por target». No editar el plan a mano.
- **Coste**: `completa` hace dos builds release + tests por push y sistema; asumible con esqueleto vacío. `actions/cache` fuera de F2-1 (ensucia la prueba «en limpio»).

## La decisión de diseño que más me preocupa

Qué significa «build reproducible». Si xtask compara en el mismo runner con dos `--target-dir`, sale verde aunque el binario incruste rutas absolutas de `CARGO_HOME` o del checkout; F2-10 publicaría sha256 irreproducibles y el `cli.lock` de F1-14 perdería base. Propuesta: dos builds en rutas de checkout distintas, con `--remap-path-prefix` en `.cargo/config.toml` desde el primer commit. Ni el remapeo ni que basten en Linux/macOS están verificados.
