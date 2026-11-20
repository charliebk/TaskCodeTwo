# Informe de revision — TASK-049 (ronda 2)

- Commit revisado: fc65d9e024c04921628c2087ca6622c7efaaade0
- Revisor: rust-wasm-reviewer
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | xtask/src/workflows.rs:1502 y :1506 | implementacion |
| MEN-1 | MENOR | corregido | xtask/src/workflows.rs:461-469 y :1496-1533 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

### IMP-1 — el test nuevo rompe `clippy --all-targets -D warnings`

El test `llamador_de_revision_ia_hereda_solo_hacia_revision_ia` usaba dos `.expect(...)` sobre `Option`, y el workspace tiene `expect_used = "warn"`. `cargo clippy -p xtask --all-targets --locked -- -D warnings` (la invocacion del paso `clippy` de `cargo xtask verificar`) fallaba con dos errores `expect_used` en `workflows.rs:1502:18` y `:1506:24`. En 17ad62c salia limpio: la regresion entra en esta ronda. `cargo test -p xtask` pasaba porque no aplica los lints.

Corregido en 9c14787: `let ... else { panic!(...) }` en lugar de `expect`. `cargo clippy -p xtask --all-targets --locked -- -D warnings`: sin avisos.

### MEN-1 — huecos de los perfiles nuevos sin test

Mutaciones en el clon: quitar `&& !hereda_admitido(...)`, la comparacion `destino == admitido` o `es_sha(version)` pone el test en rojo. Pero dar `herencia_admitida` al perfil `REVISION`, o `sin_escritura: false` al perfil `LLAMADOR_REVISION`, deja 29 en verde: nada vigila que el reutilizable siga rechazando `secrets: inherit` ni que el llamador rechace `contents: write`.

Corregido en 9c14787: dos asserts mas en el mismo test (`lint_revision` sobre el llamador da `secrets: inherit`; el llamador con `contents: write` da error de escritura).

## Alcance y reproduccion

- Revisados: `xtask/src/workflows.rs` (perfil `LLAMADOR_REVISION`, `herencia_admitida`, `hereda_admitido`, test nuevo), y en `.github/workflows/revision-ia.yml` el motivo de error de la API y el llamador de ejemplo de la cabecera. Contrastado con `docs/ci/revision-ia.md`.
- No se ha vuelto a revisar `crates/tc2` (fuera del foco pedido). Limite JS-WASM: sin cambios.
- Clon en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-049-r2-a`, con `CARGO_TARGET_DIR` propio, borrado al terminar.
- `cargo fmt --check -p xtask`: limpio. `cargo test -p xtask`: 29 ok.
- Casos probados: el llamador de la guia con `@<sha>` = 40 ceros; las variantes otro `owner/repo`, `@main` y perfil de verificacion; `jq` sobre un cuerpo de error, un fichero ausente y una respuesta no JSON.

## Revisado sin hallazgos

- `hereda_admitido`: exige la igualdad exacta del destino y la SHA de 40 hex. Falla en cerrado si el perfil no admite herencia o si el job no tiene `uses`. Una capitalizacion distinta del owner se rechaza (falla en cerrado, no es un agujero).
- Motivo de error: con `pipefail` nunca aborta, sin cuerpo cae a `sin-cuerpo` y empieza por `$rid:`, asi que no puede iniciar un comando `::` del workflow.

## Lecturas fuera del contexto permitido

ninguna
