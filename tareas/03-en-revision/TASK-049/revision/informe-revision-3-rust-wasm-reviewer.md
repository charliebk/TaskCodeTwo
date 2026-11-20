# Informe de revision — TASK-049 (ronda 3)

- Commit revisado: ca4de580f20504840ec08a8d48b2aae9340b3b30
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | xtask/src/workflows.rs:1502 y :1506 | implementacion |
| MEN-1 | MENOR | corregido | xtask/src/workflows.rs:1534-1548 | implementacion |

Origen, uno por hallazgo: `implementacion` si el codigo no hace lo que el plan pedia; `plan` si el
plan era ambiguo o equivocado y el codigo lo siguio; `especificacion` si el fallo esta en el
enunciado o en los criterios de aceptacion de la tarea.

Debajo, la reproduccion de cada hallazgo, o "sin hallazgos" explicito.

Nota de commit: el scaffold trae `ca4de58` como commit revisado. La punta de la rama es `3c54cf4`, y entre `9c14787` y la punta solo cambian ficheros de `tareas/` (peticion e informes). El codigo de `xtask/` y del workflow es identico desde `9c14787`, que es el commit de correcciones que se verifica.

### IMP-1 (IMPORTANTE) — corregido

Fallo original: el test `llamador_de_revision_ia_hereda_solo_hacia_revision_ia` usaba `expect` (`xtask/src/workflows.rs:1502` y `:1506`) y rompia `cargo clippy -p xtask --all-targets --locked -- -D warnings`. Ahora usa `let ... else` con `panic!` con mensaje (`:1502-1507`).

Reproduccion: `cargo clippy -p xtask --all-targets --locked -- -D warnings` en el clon, en la punta de la rama: "No issues found", exit 0. Verificado.

### MEN-1 (MENOR) — corregido

Asserts nuevos en `xtask/src/workflows.rs:1534-1548`. Cada uno se comprueba con una mutacion temporal en el clon (restaurada con `git checkout` tras cada una):

- Mutacion 1: `if false && escritura && perfil.sin_escritura` (linea 563, lint de escritura con GITHUB_TOKEN). El test pasa a rojo en la linea 1544, que es el assert nuevo de `contents: write` en el llamador.
- Mutacion 3: `herencia_admitida: Some("charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml")` solo en el perfil `REVISION` (linea 457). El test pasa a rojo en la linea 1536, que es el assert nuevo de `secrets: inherit` contra el reutilizable.
- Mutacion 2 (admitir la herencia en cualquier perfil, linea 491): rojo en la linea 1529, un assert anterior. Muerde, pero no demuestra el assert nuevo; eso lo hace la mutacion 3.

Ninguna mutacion deja el test en verde.

### Hallazgos nuevos

Sin hallazgos nuevos. Lo que el diff de `9c14787` podia romper y se comprobo:

- Filtro `jq` del mensaje de error (`.github/workflows/revision-ia.yml:365`). Con la clave definida enmascara la clave (`invalid x-api-key ***`) y recorta a 200 caracteres. Con la clave vacia desfigura el mensaje (`a***u***t***...`), pero esa rama no es alcanzable: el paso corta antes con `[ -z "$ANTHROPIC_API_KEY" ]` (linea 341) y no llama a la API. La variable viene siempre de `secrets.ANTHROPIC_API_KEY` en el `env:` del paso (linea 314). Nota, no hallazgo.

## Alcance
- Ficheros revisados: `xtask/src/workflows.rs` (el unico `.rs` del diff de la ronda). `docs/` y `.github/workflows/revision-ia.yml` cambian en esa ronda, pero no casan con los patrones de este revisor; solo se miraron como contexto del filtro `jq`.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada. `xtask` es una herramienta de host, sin `#[wasm_bindgen]`.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-049-r3-a`, rama `feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom`, `CARGO_TARGET_DIR` dentro del clon. Carpeta borrada al terminar.
- Build: `cargo test -p xtask --locked` compila sin errores (nativo; sin objetivo `wasm32`, no aplica al alcance).
- Formato y lints: `cargo fmt --all -- --check`, exit 0. `cargo clippy -p xtask --all-targets --locked -- -D warnings`, "No issues found", exit 0.
- Suite: `cargo test -p xtask --locked`, 29 passed, 0 failed, incluido `llamador_de_revision_ia_hereda_solo_hacia_revision_ia`. No se ejecuto la suite del workspace completo: el diff de la ronda solo toca `xtask/` y la documentacion y el workflow que `xtask` valida.
- Casos construidos: filtro `jq` con clave definida, vacia y no definida, sobre una respuesta de error de API simulada (ver nota de `### Hallazgos nuevos`).
- Mutacion: tres mutaciones descritas en `### MEN-1`. Las tres ponen el test en rojo.
- Medidas: no aplican.

## Revisado sin hallazgos
- `xtask/src/workflows.rs:1502-1507` (`let ... else` que sustituye a los `expect`): clippy limpio y test en verde.
- `xtask/src/workflows.rs:1534-1548` (asserts nuevos): cada uno muerde (ver `### MEN-1`).
- `docs/adr/F1-8-seguridad-modelo-de-amenazas.md` y `docs/ci/revision-ia.md`: cambios solo de texto; no son codigo.

## Lecturas fuera del contexto permitido

ninguna
