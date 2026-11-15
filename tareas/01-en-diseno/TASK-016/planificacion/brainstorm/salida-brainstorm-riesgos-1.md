# Salida brainstorm riesgos — TASK-016 (ronda 1)

> El esqueleto es en Rust (ADR F1-14; confirmado por la persona). Lo verificado lleva la fuente entre paréntesis.

## Modos de fallo, ordenados por gravedad

1. **Reproducibilidad comprobada solo en Windows.** Sin `/Brepro` MSVC da sha256 distinto en dos builds en limpio; con `/Brepro` da el mismo, pero solo medido en win-x64, misma carpeta y máquina. Linux y macOS nunca compilados (F1-14, PoC B y X, «No verificado»). Pueden romperlo las rutas absolutas de `~/.cargo/registry` y del workspace, y en macOS el UUID de Mach-O. Escenario: CI verde en los tres y dos builds del mismo commit con hashes distintos; el `cli.lock` de F2-12 pierde base. Mitigación: job por sistema que compile dos veces en limpio, en rutas distintas, y compare sha256; `--remap-path-prefix` y `/Brepro` en `.cargo/config.toml` versionado, no en variables de CI.
2. **Toolchain sin fijar.** Sin `rust-toolchain.toml` con versión exacta (no `stable`), un runner que actualiza rustc cambia bytes, puede romper `clippy -D warnings` sin tocar código y desalinea la clave de caché de F1-7 (D5a: rustc en la huella). Mitigación: versión exacta + componentes `rustfmt` y `clippy`; subirla es una tarea.
3. **`Cargo.lock` ignorado en CI.** Sin `--locked` el runner reescribe el lock. Mitigación: `--locked` en todos los pasos; el lock no cambia tras la CI.
4. **`ring` / C en Windows** (si entra TLS con ureq+rustls): necesita MSVC Build Tools (F1-14 punto 1). Si F2-1 no necesita red, no meter TLS en el esqueleto: el riesgo baja a F2-6.
5. **Convivencia con la 0.16 al declarar `verificacion`.** `.taskcode/config.yml` lo deja para F2-1 junto con `rutas_documentacion` (verificado). Hook `stop-verificacion` (F1-12 §C): con `cargo test --locked`, cada tarea de documentación pagaría una compilación (38 s en limpio en la PoC) o fallaría sin toolchain. Mitigación: declarar `verificacion` y `rutas_documentacion` en el mismo commit y comprobar con una tarea de documentación que no compila.
6. **`target/` sin ignorar.** `.gitignore` tiene `dist/` y `node_modules/` pero no `target/` (verificado). Mitigación: añadir `target/` en el mismo commit que `Cargo.toml`.
7. **CRLF en Windows.** `.gitattributes` solo fuerza LF en `*.sh` (verificado). Con `core.autocrlf=true`, `.rs` con CRLF: `cargo fmt --check` distinto por sistema e `include_str!` rompe reproducibilidad entre sistemas. Mitigación: `* text=auto eol=lf` o al menos `*.rs`, `*.toml`, `Cargo.lock` con LF.

## Estados intermedios y fallos parciales

- «Verde desde el primer commit»: si `Cargo.toml`, toolchain, lint y CI llegan en commits separados, los intermedios quedan sin CI o en rojo. Un commit atómico, o declarar que se mide en el merge.
- Uno de tres en rojo: tentación de `continue-on-error` o de quitar macOS → criterio falso en silencio.
- Caché de `target/` o `~/.cargo` de otra rustc: verde falso o fallo de enlace. Clave con rustc, lock y SO.
- Descarga de crates.io/toolchain que falla: rojo intermitente; distinguir red de compilación.

## Compatibilidad hacia atrás

- `ci.yml`: el job `comprobar-repo` se mantiene; se añaden jobs. Acciones nuevas fijadas por SHA, `contents: read`, `persist-credentials: false` (F1-8 lo recoge).
- Binario `tc2`, nunca `taskctl` (F1-12): en el PATH ya hay dos `taskctl` de la 0.16.1.
- No tocar `tareas/`, `logs/` ni scripts de Git-Flow.

## Vuelta atrás

- Merge `--no-ff` revertible entero. No se deshace una clave `verificacion` ya leída por tareas en curso ni un crate publicado (no publicar en F2-1).

## El riesgo que más me preocupa

El 1: «build reproducible» se daría por cumplido con el verde de la CI, que no lo mide.

## Suposiciones no verificadas

- Que `stop-verificacion` ejecuta `verificacion` en cada Stop y que `rutas_documentacion` lo evita para documentación (mirar plugin 0.16.1).
- Que el esqueleto incluye TLS; si no, el riesgo 4 baja a F2-6.
- Que `/Brepro` y `--remap-path-prefix` bastan en Linux y macOS.
- La fila F2-1 del plan sigue diciendo TypeScript (línea 327): corregirla.
