# Informe de revision — TASK-041 (ronda 2)

- Commit revisado: 5a73f44 (la peticion nombra 5fe9c4f; el clon se hizo en 5a73f44, que solo anade la peticion de ronda 2 y los informes. El codigo de la correccion es 944a974, incluido en ambos)
- Revisor: rust-wasm-reviewer
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | verificado | xtask/src/workflows.rs:80-109 (`leer_yaml`), 133 (`mapa`) | implementacion |
| IMP-2 | IMPORTANTE | verificado | xtask/src/workflows.rs:395-455 (raiz `env`/`defaults`), 342-357 y 458-490 (normalizacion y patrones) | implementacion |
| IMP-3 | IMPORTANTE | verificado | xtask/src/workflows.rs:627-639 (`leer`, `sin_crlf`), `.gitattributes` | implementacion |
| MEN-1 | MENOR | verificado | xtask/src/workflows.rs:458-465 (`uses` a nivel de job) | implementacion |
| MENOR-1 | MENOR | aceptado | xtask/src/workflows.rs:458-465 (`uses` a nivel de job) | implementacion |

No queda ningun CRITICO ni IMPORTANTE abierto. Los cuatro de la ronda 1 quedan verificados. MENOR-1 es nuevo y se acepta con su motivo.

### IMP-1 — verificado: el lector falla en cerrado

- Donde: `xtask/src/workflows.rs:80-109` (`leer_yaml`), `133` (`mapa`).
- Ronda 1: una continuacion de escalar plano o una sangria impar cortaba el analisis y el lint devolvia `[]`.
- Reproduccion (sonda temporal, revertida): sobre `BUENO`, la linea `- run: echo "$TITULO"` se sustituye por `- run: echo uno` seguida de `          dos`, y se anade un job `malo` con `permissions: write-all` e inyeccion por `github.event.issue.title`. Resultado: `["documento no entendido: linea 16: dos"]`. Segun el informe de la ronda 1, antes devolvia `[]`.
- Sangria impar (`         b`): no se sondeo aparte; lo cubre `el_lector_falla_en_cerrado`, que pasa.
- Mutacion: en `leer_yaml`, la rama de linea sobrante (`Some(l) => Err`) se cambio por `Ok(raiz)`. `el_lector_falla_en_cerrado` pasa a rojo (22 pasan, 1 falla). Revertido.

### IMP-2 — verificado: raiz y formas alternativas

- Donde: `xtask/src/workflows.rs:395-455` (raiz), `342-357` (`normalizar`, `usa_secretos`), `458-490` (patrones del job).
- Reproduccion (sondas temporales sobre `BUENO`, revertidas):
  - `github['event']['issue']['title']` en `run:` -> `["job ejecutar: github.event. en run"]`. Antes `[]` (informe ronda 1).
  - `BASE: ${{ secrets['TOKEN'] }}` en job de clase ejecuta -> `["job ejecutar: secreto en clase ejecuta"]`. Antes `[]`.
  - `env:` raiz con `${{ secrets.TOKEN }}` -> `["raiz: secreto en env"]`. Antes `[]`.
  - `echo '${{ toJSON( github) }}'` en `run:` -> `["job ejecutar: tojson(github en run"]`. Antes `[]`.
- Nota: `github['event']` en el `env:` de un job privilegiado no se marca. Es por diseno: la regla es que los datos del evento no van dentro de `run:`. Pasarlos por `env:` y leerlos como variable es el patron que usa el propio workflow (`TITULO`). No es un hallazgo.

### IMP-3 — verificado: CRLF

- Donde: `xtask/src/workflows.rs:627-639` (`leer`, `sin_crlf`), `.gitattributes` (`*.yml text eol=lf`, `*.yaml text eol=lf`).
- Comprobado en el clon: `git ls-files --eol` da `i/lf w/lf attr/text eol=lf` para `.github/workflows/verificacion.yml` y `plantillas/gitlab/verificacion.gitlab-ci.yml`, y 0 retornos de carro en disco, con `core.autocrlf=true` en el clon. El `eol=lf` de `.gitattributes` lo cubre.
- `leer_normaliza_crlf` pasa.

### MEN-1 — verificado: `uses:` de job exige SHA

- Donde: `xtask/src/workflows.rs:458-465`.
- Reproduccion (sonda temporal, revertida): job `v` con `uses: o/r/.github/workflows/x.yml@v1` -> `["job v: workflow reutilizable sin fijar por SHA: o/r/.github/workflows/x.yml@v1"]`. Antes `[]`. El control `BUENO` sigue en `[]`.
- `uses_a_nivel_de_job_exige_sha` pasa.

### MENOR-1 — llamada local sin SHA queda rechazada (nuevo, introducido por la correccion)

- Donde: `xtask/src/workflows.rs:458-465`.
- Que pasa: `uses: ./.github/workflows/x.yml` (workflow reutilizable del mismo repo, sin `@`) se rechaza como "sin fijar por SHA". Para una llamada local el commit ya fija la version, asi que es un falso positivo del lint en sentido restrictivo: falla en cerrado, no abre un hueco.
- Reproduccion (sonda temporal, revertida): job `v` con `uses: ./.github/workflows/x.yml` -> `["job v: workflow reutilizable sin fijar por SHA: ./.github/workflows/x.yml"]`.
- Impacto: ninguno hoy. `verificacion.yml` no tiene `uses` a nivel de job (solo `actions/checkout` dentro de `steps`). Se bloquearia si alguien anade una llamada local.
- Sugerencia: no corregir en esta tarea. Dejar escrito en el docstring de `lint_github` que las llamadas locales quedan fuera de la lista blanca, o permitir rutas `./` sin `@`.
- Decision: aceptado. Motivo: el lint falla en cerrado y el repo no usa llamadas locales.

## Alcance
- Ficheros revisados: `xtask/src/workflows.rs` (el codigo de la correccion, commit 944a974) y `.gitattributes` (IMP-3). El commit 5a73f44 solo anade la peticion y los informes; el commit 5fe9c4f anade docs (`docs/ci/verificacion-reutilizable.md`), fuera de los patrones de este revisor y no revisado aqui.
- Contexto adicional pedido: ninguno.
- Limite JS-WASM: sin cambios de firma exportada. No hay `#[wasm_bindgen]` en el diff; el crate `xtask` es nativo.

## Lecturas fuera del contexto permitido
- `C:\Users\nullcad2025\.claude\plugins\cache\taskcode-marketplace\taskcode-plugin\0.16.2\skills\task-workflow\revision.md`: guia comun de revision que exige la skill.
- `tareas/03-en-revision/TASK-041/revision/informe-revision-1-rust-wasm-reviewer.md`: origen de los hallazgos a verificar, citado en la peticion.
- `.github/workflows/verificacion.yml` (busqueda de `uses:` y comprobacion de finales de linea): para confirmar que el caso de MENOR-1 no esta en el repo.
- `.gitattributes`: parte del diff de IMP-3.

## Reproduccion
- Clon: `C:\Users\nullcad2025\AppData\Local\pruebas-claude\taskcodetwo\task-041-r2-a`, rama de la tarea, commit 5a73f44. Sin commits ni push. Todas las sondas se revirtieron con `git checkout -- xtask/src/workflows.rs`; el arbol del clon queda limpio.
- Build: `cargo test -p xtask` compila el crate nativo. Sin objetivo `wasm32`: `xtask` no es WASM.
- Formato y lints: `cargo clippy -p xtask --all-targets -- -D warnings`: sin avisos. `cargo fmt --check` no se ejecuto (fuera de la peticion).
- Suite: `cargo test -p xtask`: 23 pasan, 0 fallan. Sin la suite completa ni `cargo xtask verificar`, segun la peticion. Esta pasada es la unica de la ronda.
- Finales de linea: `git ls-files --eol` da `i/lf w/lf attr/text eol=lf` para los `.yml` del clon. Ver IMP-3.
- Casos construidos: sondas con `lint_github` sobre `BUENO` (prueba temporal en `workflows.rs`, revertida). Los valores "antes" vienen del informe de la ronda 1; no se reejecuto el codigo anterior.

| Caso | Antes (ronda 1) | Ahora |
|---|---|---|
| R1 continuacion de escalar plano | `[]` | `["documento no entendido: linea 16: dos"]` |
| R2 `github['event']` en `run:` | `[]` | `["job ejecutar: github.event. en run"]` |
| R3 `secrets['TOKEN']` en clase ejecuta | `[]` | `["job ejecutar: secreto en clase ejecuta"]` |
| R4 `env:` raiz con `secrets.TOKEN` | `[]` | `["raiz: secreto en env"]` |
| R5 `toJSON( github)` en `run:` | `[]` | `["job ejecutar: tojson(github en run"]` |
| R6 `uses:` de job sin SHA | `[]` | `["job v: workflow reutilizable sin fijar por SHA: o/r/.github/workflows/x.yml@v1"]` |
| Control `BUENO` | `[]` | `[]` |

- Mutacion: en `leer_yaml`, la rama de linea sobrante devuelve `Ok(raiz)`: `el_lector_falla_en_cerrado` pasa a rojo. Revertido. Las demas pruebas nuevas no se mutaron; los casos R1 a R6 demuestran el comportamiento.
- Medidas: no aplican.

## Revisado sin hallazgos
- Lector: `leer_yaml`, `mapa` y `lista` propagan `Result` sin `unwrap` fuera de pruebas. Los `panic!` estan solo en `yaml` y `plantilla`, bajo `#[cfg(test)]`. Los numeros de linea del error son correctos: el lector no filtra lineas.
- Fail-closed: la comprobacion de linea sobrante es la que devuelve `Err`; la mutacion lo prueba.
- `normalizar` y `usa_secretos`: no dejan pasar `github.event` ni `secrets` en las formas probadas (R2, R3).
- `lee_variable` y la regla de `rules:if` de GitLab: revisados por lectura; `las_reglas_de_gitlab_no_leen_variables_de_job` pasa. No se sondearon casos aparte.
- `clippy` sin avisos nuevos.
- `github['event']` en `env:` de job privilegiado: no se marca por diseno (ver IMP-2).
