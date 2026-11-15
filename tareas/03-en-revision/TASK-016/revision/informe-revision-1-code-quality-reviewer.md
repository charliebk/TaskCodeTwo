# Informe de revision — TASK-016 (ronda 1)

- Commit revisado: 9d72eae2729bfc231921d0eb9a25373365dc10b3
- Revisor: code-quality-reviewer
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | .cargo/config.toml:7 | implementacion |
| MEN-2 | MENOR | abierto | .taskcode/config.yml:5 | plan |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Debajo, la reproduccion de cada hallazgo, o "sin hallazgos" explicito.

Veredicto propuesto: aprobada (solo hay MENOR; el orquestador lo escribe con taskctl).

## Puerta y reproduccion

- Clon temporal en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-016-r1-calidad`, en el commit 0743f63 (HEAD de la rama; el codigo es el de 9d72eae/e904ef8).
- `cargo xtask verificar --perfil rapida`: verde (fmt, clippy `-D warnings`, 4 tests: 2 de `tc2/tests/cli.rs` y 2 de xtask; la lib no tiene tests propios). `--perfil completa` en Windows local: verde, mismo sha256 en las dos rutas.
- CI del primer commit de codigo (run 43712405534, sha e904ef8): 4 jobs `success` (comprobar-repo y codigo en ubuntu, macos y windows). En el log de windows: dos sha256 identicos (14c60c45...) y "build reproducible".
- Mutantes (todos detectados): sustituir `/Brepro` por otro flag da `ROJO: build NO reproducible` (hashes distintos); incrustar `env!("CARGO_MANIFEST_DIR")` en el binario da `NO reproducible` (la prueba con dos rutas discrimina rutas absolutas); definir `RUSTFLAGS` da `ROJO ... quitalo`; mala formato en `main.rs` da rojo en `fmt`.
- `git ls-files` no contiene `target/` ni `.exe`; el worktree queda limpio tras la suite.
- Las claves `comando_verificacion` y `rutas_documentacion` existen en taskctl 0.16.1 (`dist/src/commands/doctor.js`).

### MEN-1: falta `--remap-path-prefix` que el plan pedia en `.cargo/config.toml`

El plan (paso 2 y suposicion 1) lo incluye junto a `/Brepro`; `grep -rn remap .cargo Cargo.toml xtask crates` no encuentra nada. Hoy no tiene efecto (binario sin dependencias, `strip = true`, la prueba de dos rutas sale verde), pero el dia que entre una dependencia con rutas de `~/.cargo/registry` en mensajes de panic, el binario dependera de `CARGO_HOME` y de la ruta del checkout. La comprobacion de las dos rutas solo cubre el checkout, no el registro, y la reproducibilidad se mide solo en un runner. No bloquea: se anade con la primera dependencia o se anota como decision (aceptado).

### MEN-2: el perfil `rapida` ejecuta tests; el plan hablaba de `rapida` sin tests

Plan, riesgo 5: "perfil `rapida` sin tests" para que `stop-verificacion` no compile en tareas de documentacion. `xtask` ejecuta `cargo test` en `rapida` y el comentario de `.taskcode/config.yml` lo describe como "rapida + tests". Es coherente con "Cambios de la persona tras aprobar" (fmt, clippy, build y tests), asi que el plan era ambiguo. Medido: 3,4 s con cache, y `rutas_documentacion` mantiene fuera a las tareas de documentacion; sin consecuencia hoy. Conviene aclarar el texto del plan o dejarlo aceptado.

## Lecturas fuera del contexto permitido

ninguna
