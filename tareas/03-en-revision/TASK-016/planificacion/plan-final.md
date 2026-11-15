# Plan — TASK-016: F2-1 Esqueleto: Rust, build reproducible

(Lo consolida el agente unificador a partir de 4 roles de brainstorm
lanzados en paralelo: arquitectura, riesgos, testing, dominio. Cuatro roles por
heurística de complejidad «crítica» (19 dependencias). Los desacuerdos entre
roles se señalan, no se promedian. Este plan no vale hasta `taskctl approve`.)

## Enfoque propuesto

Duracion estimada: 150 min
Señales: unos 16 ficheros nuevos o tocados (workspace, `crates/tc2` con lib, main y test, `xtask`, toolchain, `.cargo/config.toml`, `ci.yml`, `config.yml`, `.gitignore`, `.gitattributes`, plan y BOARD); 3 capas (binario, xtask, CI); 3 criterios; CI en tres sistemas con reproducibilidad no medida en Linux/macOS (riesgos 1), así que hay que contar con varias vueltas de CI.
No se trocea: decisión de la persona, «no quebramos ninguna tarea». Además «verde desde el primer commit» obliga a que workspace, toolchain, xtask y CI entren juntos (arquitectura, riesgos, testing).
1. Corregir título/objetivo de TASK-016, fila F2-1 de `docs/plan/PLAN-TASKCODE-2.md` (línea 327) y `docs/BOARD.md` si la nombra, con el texto de la divergencia 1 de F1-14 (dominio, testing, riesgos; decidido por la persona).
2. Primer commit de código, atómico y empujado solo: `Cargo.toml` (workspace virtual, `crates/tc2` + `xtask`, `[workspace.lints]`, `[profile.release]` de F1-14 punto 3), `Cargo.lock`, `rust-toolchain.toml` (1.100.0, rustfmt + clippy), `.cargo/config.toml` (`/Brepro` en targets msvc, `--remap-path-prefix`, alias `xtask`), `.gitignore` con `/target/`, `.gitattributes` LF para `*.rs`, `*.toml` y `Cargo.lock`, `crates/tc2` (`tc2 --version`, una línea sin ANSI), `xtask verificar --perfil rapida|completa` y el job `codigo` en `ci.yml` (arquitectura).
3. `.taskcode/config.yml`: verificación local = perfil `rapida` + `rutas_documentacion`, después de comprobar los nombres de las claves en el plugin taskctl 0.16.1 (riesgos 5, dominio). Se actualiza el comentario de `ci.yml:1-2` (testing).

## Desacuerdos entre roles, y como se resuelven

- Ruta de los dos builds — testing: misma ruta con `cargo clean` / arquitectura y riesgos: dos rutas distintas con `--remap-path-prefix` — ganan arquitectura y riesgos — lo decidió la persona: con una sola ruta saldría verde aunque el binario lleve rutas absolutas incrustadas (arquitectura).
- Cuándo se mide «verde desde el primer commit» — riesgos: commit atómico «o declarar que se mide en el merge» / dominio y testing: en el SHA del primer commit de código — ganan dominio y testing — decisión de la persona: commit atómico empujado solo, evidencia en ese SHA y además en el merge.
- Caché en CI — riesgos: caché con clave rustc + lock + SO / arquitectura: `actions/cache` fuera de F2-1 — gana arquitectura — con caché ya no se prueba «en limpio», y riesgos mismo presenta la caché como fuente de verdes falsos.
- Nombre del paquete Cargo — dominio: paquete `taskcodetwo` con binario `tc2` (F1-12 D1a) / arquitectura: paquete `tc2` en `crates/tc2` — gana arquitectura — la persona eligió `crates/tc2` con binario `tc2`; queda como suposición qué entiende F1-12 por «paquete».
- Clave de verificación en `.taskcode/config.yml` — arquitectura: `comando_verificacion` / riesgos: `verificacion` — sin decidir aquí — la persona manda comprobarlo en el plugin 0.16.1 antes de escribirla.
- Cómo se corrige el plan — arquitectura: con `taskctl` (D9a de F1-14), «no editar el plan a mano» / persona: corregirlo dentro de esta tarea — se corrige dentro de la tarea a mano: `taskctl --help` (0.16.1) no tiene ningún comando que edite filas del plan ni títulos (comprobado); el cambio va en un commit propio de la rama y pasa por la revisión por pares, que es lo que protege D9a (opción recomendada, delegada por la persona).

## Riesgos aceptados y que los contiene

- Que en Linux o macOS no salga reproducible (rutas de `~/.cargo/registry`, UUID de Mach-O) — riesgos 1 — lo contiene: la comparación de sha256 en los tres sistemas; si falla, se documenta con evidencia y no se maquilla (persona).
- Que un `RUSTFLAGS` del entorno anule en silencio los `rustflags` de `.cargo/config.toml` — arquitectura — lo contiene: xtask se niega a correr si `RUSTFLAGS` está definido.
- Que `stop-verificacion` haga compilar a las tareas de documentación (38 s en limpio en la PoC) — riesgos 5 — lo contiene: perfil `rapida` sin tests + `rutas_documentacion`, y comprobarlo con una tarea de documentación.
- Coste: dos builds release con `lto=thin` y `codegen-units=1` por push y sistema — arquitectura, testing — se acepta con el esqueleto vacío.
- Tentación de `continue-on-error` o de quitar macOS si uno de los tres queda en rojo — riesgos — lo contiene: prohibido; `fail-fast: false` y recuento de los tres jobs (testing 6).
- Descargas de crates.io o de la toolchain que fallan: rojo intermitente — riesgos — lo contiene: volver a lanzar y distinguir red de compilación en el `## Resultado`.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):
- C1 «CI verde en los tres desde el primer commit»: 1 comprobación externa, pesada — `gh run list --workflow ci.yml --commit <sha> --json databaseId,conclusion` + `gh run view <id> --json jobs` → 4 jobs `success` (`comprobar-repo` + `codigo` en ubuntu/macos/windows-latest), en el SHA de `git log --reverse --diff-filter=A --format=%H develop..HEAD -- Cargo.toml` y en el merge de `develop` (testing).
- C2 «cumple F2-1»: 2 tests nuevos, ligeros — integración en `crates/tc2/tests/`: `tc2 --version` da código 0 y la versión de `Cargo.toml`; un argumento desconocido da código ≠ 0 (testing 2). Lint y lock sin tests nuevos: `cargo xtask verificar --perfil rapida` (`fmt --check`, `clippy --locked -D warnings`) y `--locked` en todo (testing 3 y 5, riesgos 3).
- C3 «mismo sha256 por target»: 1 test nuevo, pesado — paso de `xtask verificar --perfil completa`: dos `cargo build --release --locked` en limpio en dos rutas distintas, comparando sha256, en los tres runners (arquitectura, riesgos 1).
- Mutaciones, una vez y anotadas en el `## Resultado` (testing): quitar `/Brepro` → rojo en Windows; variable sin usar → rojo en clippy; mala indentación → rojo en fmt; quitar `/target/` de `.gitignore` → rojo; matriz solo con ubuntu → falla el recuento.
- Sin artefactos: `git status --porcelain` vacío tras el build y `git ls-files` sin `target/`, `*.exe` ni `dist/` (testing 4).
- Trampas que hay que evitar (testing): `cargo test` verde con 0 tests; contar `skipped`/`cancelled` como verde; dar por bueno el verde de `develop` en lugar del del primer commit.

## Sin cubrir

- `comprobar-repo.sh`: arquitectura pide mirar si sus comprobaciones de enlaces recorren las rutas nuevas; ningún rol lo verificó → lo comprueba el implementador.
- Corregir F2-10..F2-12 con `tc2` (dominio, pregunta 1): queda fuera; solo se toca F2-1.

## Salidas que faltaron

- ninguna (llegaron las cuatro). No se lanzó el rol de rendimiento.

## Suposiciones no verificadas

- Que `/Brepro` y `--remap-path-prefix` bastan en los runners — riesgos, testing, arquitectura — se confirma con el job `codigo` en los tres sistemas.
- Que `macos-latest` y `windows-latest` traen rustup y MSVC listos — testing, arquitectura — se confirma en el log del primer run.
- Que `stop-verificacion` ejecuta la verificación en cada Stop y que `rutas_documentacion` lo evita — riesgos — mirarlo en el plugin taskctl 0.16.1.
- Que xtask necesite `sha2` (tc2 sigue sin dependencias) — arquitectura — se decide al escribir xtask; entra en `Cargo.lock`.
- Que el repo sigue sin rulesets y el verde solo se comprueba con `gh run` — testing — `gh api repos/{owner}/{repo}/rulesets`.

## Lo que necesita decision de una persona

Ya decididas por la persona (Carlos delegó «opción recomendada» en todas):
- Lenguaje Rust (ADR F1-14); dentro de esta tarea se corrigen título/objetivo, fila F2-1 del plan y BOARD con la divergencia 1.
- Criterio nuevo: dos builds en limpio con el mismo sha256 por target, en ubuntu/macos/windows-latest, mismo runner, dos rutas, `--remap-path-prefix` y `/Brepro` en `.cargo/config.toml` versionado; si en algún sistema no es reproducible, se documenta con evidencia.
- Toolchain 1.100.0 exacta (rustfmt, clippy), edición 2024, `rust-version = "1.100"`, `license = "MIT"`.
- Workspace virtual `crates/tc2` (binario `tc2`, sin dependencias) + `xtask`; el job se llama `codigo`, no `verificacion`; `comprobar-repo` intacto; acciones por SHA, `contents: read`.
- Local = perfil `rapida` con `rutas_documentacion`, CI = perfil `completa`; las claves se verifican en el plugin 0.16.1 antes de declararlas.
- Sin TLS ni red; `/target/` en `.gitignore`; LF en `.gitattributes`; no se tocan `tareas/`, `logs/` ni los scripts de Git-Flow; no se publica nada.

Pendientes:
- Aprobar este plan entero con `taskctl approve` — ningún rol ni el unificador aprueban.

Resuelta con la opción recomendada (consigna de la persona): `taskctl` 0.16.1 no edita filas del plan, así que la fila F2-1, BOARD y el título/objetivo de la tarea se corrigen a mano en la rama, en commit propio y revisado.

## Cambios de la persona tras aprobar

- 2026-11-15, al abrir la rama: «Solo lo mediremos en Windows, ahorrate la version de Linux y mac». La comparacion de sha256 (C3) corre solo en windows-latest; en ubuntu y macos el job `codigo` hace fmt, clippy, build y tests (perfil `rapida` + tests) sin medir reproducibilidad. La reproducibilidad en Linux/macOS queda «no verificado».

