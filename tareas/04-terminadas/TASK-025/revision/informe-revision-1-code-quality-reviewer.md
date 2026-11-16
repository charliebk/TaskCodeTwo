# Informe de revision — TASK-025 (ronda 1)

- Commit revisado: e531238f359fa059a30904b61c3017ec947a5318
- Revisor: code-quality-reviewer (general-purpose, sonnet)
- Veredicto: aprobada con correcciones

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| MEN-1 | MENOR | abierto | .github/workflows/release.yml:328 | plan |
| MEN-2 | MENOR | abierto | .github/workflows/release.yml:370 | implementacion |
| MEN-3 | MENOR | abierto | .github/workflows/release.yml:292 | plan |
| MEN-4 | MENOR | abierto | .github/scripts/humo.sh:85 | implementacion |
| MEN-5 | MENOR | abierto | docs/distribucion/releases.md:453 | implementacion |
| MEN-6 | MENOR | aceptado | .github/workflows/release.yml:221 | implementacion |
| MEN-7 | MENOR | abierto | .github/workflows/release.yml:314 | implementacion |
| MEN-8 | MENOR | abierto | xtask/src/main.rs:138 | implementacion |

Sin hallazgos CRITICO ni IMPORTANTE. Evidencia: el ensayo 43784921536 y la prerelease 43785561970, los dos en verde.

## Reproduccion

### Suite y puerta determinista

- **Clon** en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-025-r1-calidad`, borrado al terminar.
- **`cargo test --workspace --locked`**: en verde salvo 2 tests de `plataforma` (`sinred`, 62 s), que no reproducen. La misma orden da 4 de 4 en `origin/develop` y en la rama. Se anota como fragilidad por carga, ajena al diff.
- **YAML del workflow**: sin actionlint ni shellcheck en la maquina, se valido con `yaml.safe_load` y a mano.
- **Inyeccion por expresiones**: ningun `run:` lleva `${{ }}`; todo pasa por `env:`.
- **Actions fijadas por SHA**: los 4 SHA coinciden con sus tags (`git ls-remote`).
- **Permisos**: `permissions: {}` global. Solo `publicar` escribe, y `verificar-release` tiene `attestations: read`.
- **`bash -e -o pipefail`** en todos los pasos.

### humo.sh en local, en Git Bash y con un `tc2` falso

- **Debe pasar**: el camino feliz, `node` en un directorio falso y un PATH con elementos vacios terminan en VERDE.
- **Debe fallar**:
  - Version equivocada: ROJO, codigo 1.
  - Sin argumentos con codigo 0: ROJO.
  - Un solo argumento: muestra el uso y sale con 2.
  - Binario inexistente: falla en `chmod`.
- La rama Linux (alpine) no se pudo probar en local por falta de Docker; queda cubierta por la CI.

### Logica de preparar y publicar, reproducida con `cargo xtask`

- **`comprobar-tag`**: estable y prerelease se aceptan. Se rechazan `v0.0.1`, `v0.0.0-`, `v0.0.00`, `0.0.0`, `v0.0.0-a b` y `v0.0.0-rc.1+x`.
- **SHA256SUMS en publicar**: con 6 binarios y sus `.sha256`, el `diff` sale identico y `sha256sum -c` da OK.
- **Fallo parcial de un target**: `publicar` y `verificar-release` quedan en `skipped`, asi que no sale una release a medias por un target caido.

### Mutacion del arreglo de git_binario.rs

Con `LANG=en_US.UTF-8` el test pasa con el arreglo. Sin el `env_remove` falla con «usage» frente a «uso».

### Evidencia de CI

- 43784921536 (ensayo, d4889a3) y 43785561970 (prerelease, d4889a3): todo en verde.
- darwin-x64 corrio en el runner Intel nativo `macos-15-intel`.
- `codesign` en los dos darwin y attestation de Sigstore para los 6 sujetos.
- Los 6 `verificar-release` verdes.

### Hallazgos

- **MEN-1 — Una release no idempotente ni atomica.** `gh release create` publica sin borrador y sube los assets en el mismo paso. Si el job se corta a medias, queda una release publica incompleta. Si se repite, falla con «already exists» y la doc no explica como recuperarse. Propuesta: crearla como `--draft` y publicarla al final. No verificado en CI.
- **MEN-2 — `gh attestation verify` laxo.** Solo lleva `--repo`, asi que acepta cualquier workflow del repositorio. Propuesta: `--signer-workflow` en el workflow y en la doc.
- **MEN-3 — Nada impone que un tag estable este sobre `main`.** La prueba es que `v0.0.0-prueba.1` se publico desde una rama `feature/`. Propuesta: `git merge-base --is-ancestor` contra `origin/main` para los tags sin sufijo.
- **MEN-4 — `alpine:3.22` es un tag mutable.** Propuesta: fijar la imagen por digest.
- **MEN-5 — La doc de verificacion solo vale en Linux.** `sha256sum` no existe en Windows; falta el equivalente en PowerShell y en macOS.
- **MEN-6 — La tabla de los 6 targets se repite** en las 3 matrices del workflow, en `dist.rs` y en la doc. Si se renombra un alias y se olvida una matriz, el humo o la verificacion quedan incompletos sin que nada falle.
- **MEN-7 — `sort -k 2` depende del locale.** Propuesta: `LC_ALL=C sort`.
- **MEN-8 — El HEAD revisado no tiene run de release.** 7527334 toca `xtask/src/main.rs`, una ruta que dispara el ensayo, pero no esta empujado. Hay que empujar y ver el ensayo en verde sobre el commit final.

## Cumplimiento del plan aprobado

Se cumple en todo lo esencial. Desviaciones menores sin impacto:

- `humo` hace un `sparse-checkout` de `.github/scripts` en lugar de no hacer checkout.
- Hay un `.sha256` por binario en lugar de un `SHA256SUMS` de los artefactos.

`comprobar-repo.sh`: 167 enlaces, 0 fallos.

## Lecturas fuera del contexto permitido

- `xtask/src/*.rs`: para reproducir `comprobar-tag` y `sumas`.
- `crates/tc2/tests/git_binario.rs`: parte del arreglo pedido.
- `.github/workflows/ci.yml`, `.github/scripts/comprobar-repo.sh`, `.cargo/config.toml` y `xtask/Cargo.toml`: SHA de las actions, enlaces y el alias `cargo xtask`.
- `docs/adr/F1-14-distribucion-e-instalacion.md` (lineas 46 y 608-620): D4a y la divergencia 4.
- `gh run view`, `git ls-remote` y `gh run list`: evidencia de CI.

## Tratamiento de los MENORES (commit e368253, quien implementa)

- **MEN-1**: `gh release create --draft`, luego se comprueba que hay 7 assets y `gh release edit --draft=false`. La doc dice como recuperarse de un borrador a medias y que relanzar sobre una release publicada falla sin cambiar nada.
- **MEN-2**: `--signer-workflow <repo>/.github/workflows/release.yml` en el workflow y en la doc. Comprobado en local con el binario descargado de la prerelease de prueba: con `release.yml` acepta (1 attestation, codigo 0) y con `ci.yml` rechaza (codigo 1).
- **MEN-3**: en `preparar`, un tag estable exige `git merge-base --is-ancestor "$GITHUB_SHA"` contra `origin/main` (checkout con `fetch-depth: 0`). No se ha probado en CI: haria falta un tag estable real.
- **MEN-4**: `alpine:3.22@sha256:49340b34...`, el digest del indice que bajo la CI en x64 y en arm64 (run 43785561970).
- **MEN-5**: la doc anade la verificacion en macOS (`shasum`) y en Windows (`Get-FileHash`), y deja claro que el hash prueba integridad y la attestation procedencia. La linea de macOS no se ha ejecutado en un Mac.
- **MEN-6**: aceptado. Las matrices de GitHub Actions son estaticas. Generarlas desde `xtask` (job previo con `fromJSON`) anade un salto y otro punto de fallo por un riesgo que hoy contiene la CI: un alias renombrado en `dist.rs` y no en la matriz rompe `build`, porque `rustup target add` usa el triple de la matriz y `humo` busca el fichero por alias. Se reconsidera en F5-1 (workflow reutilizable).
- **MEN-7**: `LC_ALL=C sort -k 2`.
- **MEN-8**: rama subida en e368253; el ensayo de release y la ci de ese commit se anotan en el Resultado.
