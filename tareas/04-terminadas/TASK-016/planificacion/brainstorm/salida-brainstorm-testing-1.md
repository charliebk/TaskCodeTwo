# Salida brainstorm testing — TASK-016 (ronda 1)

> Conflicto previo: la fila F2-1 del plan (`docs/plan/PLAN-TASKCODE-2.md:327`) dice «TypeScript»; el ADR F1-14 (aceptado) decide Rust y propone el texto nuevo de F2-1 (`docs/adr/F1-14-distribucion-e-instalacion.md:602-606`): `Cargo.lock` versionado, sin binarios en el repo, `/Brepro` en Windows, `cargo fmt` y `clippy`, y dos builds en limpio con el mismo sha256 por target. Se planifica contra el ADR (confirmado por la persona: el lenguaje es Rust). Hay que corregir la fila del plan / la tarea antes de `approve`.

## Qué es observable sin llamar a ningún agente

- Criterio 1: run del workflow en el SHA del primer commit de código:
  `gh run list --workflow ci.yml --commit <sha> --json databaseId,conclusion` y
  `gh run view <id> --json jobs --jq '.jobs[]|[.name,.conclusion]'` → cuatro jobs `success`: `comprobar-repo` y uno por `ubuntu-latest`, `macos-latest`, `windows-latest`.
- «Primer commit»: `git log --reverse --diff-filter=A --format=%H develop..HEAD -- Cargo.toml`. Ese commit añade también la matriz de CI (si no, hay código sin CI).
- Empujarlo solo: `concurrency` con `cancel-in-progress` (`ci.yml:15-17`) y que GitHub solo corra el head de cada push hacen que un commit empujado en lote no tenga run.
- Después, el merge commit en `develop` también en verde. Los IDs de ambos runs van al `## Resultado`.
- Local: `cargo build --locked --release`, `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `git status --porcelain` vacío tras el build.

## Plan de pruebas (cada una con su mutación)

1. Build reproducible por sistema: build release, sha256, `cargo clean`, rebuild en la misma ruta, comparar. Mutación: quitar `-C link-arg=/Brepro` de `.cargo/config.toml` → rojo en Windows (PoC B).
2. El binario arranca: test de integración que ejecuta el binario (`--version`, código 0, versión = `Cargo.toml`) y argumento desconocido (código ≠ 0). Mutación: cambiar código/versión → rojo en los tres.
3. `Cargo.lock` fijado: build y tests con `--locked`. Mutación: dependencia nueva sin lock → rojo.
4. Sin artefactos versionados: `git status --porcelain` vacío y `git ls-files` sin `target/`, `*.exe`, `dist/`. Mutación: quitar `target/` de `.gitignore` → rojo.
5. Lint que falla de verdad: variable sin usar → rojo `clippy -D warnings`; mala indentación → rojo `fmt --check`.
6. Están los tres sistemas: la aserción externa cuenta tres jobs de matriz distintos con `runner.os` en el log. Mutación: matriz solo `ubuntu-latest` → falla el recuento.

## Aserciones trampa a evitar

- `cargo test` verde con cero tests (hace falta la prueba 2).
- `clippy` sin `-D warnings`.
- Jobs `skipped` o `cancelled` contados como verde: exigir `conclusion == success` en los cuatro.
- `continue-on-error: true` en la matriz.
- sha256 sin `cargo clean` (mide la caché); rutas distintas exigen `--remap-path-prefix`, no medido.
- Dar por bueno el verde de `develop` en vez del del SHA del primer commit de código.

## Qué pruebas existentes cambian de expectativa

- `comprobar-repo` (`.github/scripts/comprobar-repo.sh`) no cambia: sigue en `ci.yml` y en verde.
- El comentario de `ci.yml:1-2` («la CI de código llega con F2-1») deja de ser cierto.
- `.taskcode/config.yml` declara que no hay comando de verificación; F2-1 lo declara. La 0.16 lo ejecuta con `cmd.exe` en Windows (H16): mismo comando en local y en CI.

## Cómo envejece

- Toolchain sin fijar → lints nuevos ponen rojo sin cambiar código. Recomendación: `rust-toolchain.toml` fijo y subirlo en tarea propia.
- Matriz duplicada con F2-10 (seis targets) y F5-1 (workflow reutilizable, check `verificacion`): el paso de reproducibilidad como script único invocable desde cualquier workflow.
- `ring` compila C/asm: el job de Windows depende del MSVC de la imagen del runner.
- Coste: dos builds release con `lto = "thin"` y `codegen-units = 1` doblan el tiempo por sistema.

## Desacuerdos previstos

- Con dominio, sobre el criterio 2: no es comprobable hasta corregir el plan.
- Con arquitectura, si propone un workflow `verificacion` ya en F2-1: adelanta F5-1 y cambia lo que asevera F1-5.

## Suposiciones no verificadas

- Que `macos-latest` y `windows-latest` tienen toolchain y MSVC listos.
- Que `/Brepro` basta en el runner de Windows de GitHub.
- Que Linux y macOS son reproducibles sin flags en la misma ruta (F1-14, «No verificado»).
- Que el repo sigue sin rulesets (F1-5:552): el verde solo se comprueba con `gh run`.
