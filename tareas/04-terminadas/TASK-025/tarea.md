---
id: TASK-025
titulo: "F2-10 Build de ejecutables por sistema en la CI, firmados"
tipo: feature
sprint: 2
etiquetas: []
complejidad: null
modelo_sugerido: sonnet
estado: terminada
plan_aprobado: true
rama: feature/task-025-f2-10-build-de-ejecutables-por-sistema-e
asignado_a: charlie.bk@gmail.com
agente_revisor: general-purpose
skills_recomendados: []
regla_seleccion_skill: null
ultimo_commit_revisado: null
revision_codex: false
tokens_diseno: 571325
tokens_implementacion: 8835453
tokens_revision: 8333584
creado: 2026-11-14
actualizado: 2026-11-16
dependencias: [TASK-014, TASK-016]
---
## Objetivo

Build de ejecutables por sistema en la CI, firmados, con su hash, publicados en cada release (RR-16)

Tarea F2-10 del plan (docs/plan/PLAN-TASKCODE-2.md). Dependencias en el plan: F1-14, F2-1.

## Criterios de aceptacion
- [x] El ejecutable arranca en Windows, macOS y Linux limpios sin Node
- [x] Cumple lo que pide la tarea F2-10 del plan y los requisitos que cita

## Resultado

### Implementado

- **`xtask`** (`xtask/src/dist.rs` nuevo y `main.rs`):
  - `cargo xtask empaquetar --target <triple> --salida <dir>`: compila `tc2` en release para uno de los 6 targets de P12 y lo copia como `tc2-<version>-<alias>[.exe]`. En macOS lo firma ad hoc (`codesign --sign -` y `--verify --strict`). Deja al lado `<fichero>.sha256`.
  - `cargo xtask sumas <dir>`: escribe `SHA256SUMS` con el formato de `sha256sum`.
  - `cargo xtask comprobar-tag <tag>`: escribe `estable` o `prerelease`, o falla si el tag no casa con la version.
  - `cargo xtask version`.
  - Tests en `dist.rs` y `main.rs`: tabla de alias y triples, formato y bordes de `SHA256SUMS`, regla tag-version, ordenes y `dir_target`.
- **`.github/workflows/release.yml`** (nuevo):
  - `preparar`: comprueba version y tag, y exige que un tag estable este sobre `main`.
  - `build`: matriz de 6 targets en runners nativos (`windows-11-arm`, `macos-15-intel`, `ubuntu-24.04-arm`, entre otros), con musl estatico en Linux.
  - `humo`: arranque sin Node.
  - `publicar`: solo con tag; borrador primero, attestation de Sigstore, 6 binarios y `SHA256SUMS`, y al final se publica.
  - `verificar-release`: descarga anonima desde la URL publica, sha256, `gh attestation verify --signer-workflow` y humo.
  - Ensayo sin publicar con `workflow_dispatch` o con push a una rama que toque lo que entra en la release.
  - Actions fijadas por SHA, `permissions: {}` y permisos por job.
- **`.github/scripts/humo.sh`**: en Windows y macOS quita `node` del PATH y exige que no se encuentre; en Linux ejecuta el binario dentro de alpine, fijado por digest. Comprueba que `--version` da exactamente `tc2 <version>` y que sin argumentos sale con 2 y el texto de uso.
- **Documentacion**: `docs/distribucion/releases.md` dice como se publica, como recuperarse y como se verifica en los tres sistemas. La fila F2-10 de `docs/plan/PLAN-TASKCODE-2.md` recoge la divergencia 4 de ADR F1-14 (D4a).
- **Fuera de alcance, decidido por la persona**: arreglo de `crates/tc2/tests/git_binario.rs`. El test heredaba `LANG=en_US` del runner de macOS y tenia la CI de develop en rojo desde TASK-024.

### Evidencia

- **Criterio 1**: ensayo 43784921536 (d4889a3). En las 6 celdas de `humo`, «node no esta en el PATH» (o en alpine), `tc2 0.0.0`, codigo 2 sin argumentos y sha256 OK. darwin-x64 corrio en un runner Intel nativo, asi que ninguna celda queda como «no verificado». `codesign --verify` pasa en los dos macOS.
- **Criterio 2**: prerelease `v0.0.0-prueba.1` (run 43785561970), con los 21 jobs en verde: build, humo, publicar y 6 `verificar-release`.
  - La release tenia los 6 binarios y `SHA256SUMS`, y estaba marcada como prerelease.
  - Comprobado tambien en local: descarga anonima, `sha256sum -c` OK y attestation SLSA v1 de `release.yml` en el tag, con los 6 sujetos.
  - Un binario manipulado se rechaza (404, sin attestation). Con `--signer-workflow` apuntando a `ci.yml` tambien se rechaza.
  - Los sha256 de la prerelease coinciden con los del ensayo: mismo commit, distinto run.
  - La release y el tag se borraron con permiso de la persona.
- **Commit final** e368253: ensayo 43789739164 y ci 43789739270, los dos en verde en los tres sistemas. `cargo xtask verificar --perfil rapida` en verde en local.
- **Hashes de 0.0.0** (d4889a3), segun la CI:
  - win-x64 `729fbd4d…`
  - win-arm64 `a01e4407…`
  - darwin-x64 `cc6f8176…`
  - darwin-arm64 `294266fe…`
  - linux-x64 `36b01dbd…`
  - linux-arm64 `20ef8c99…`

### Revision (ronda 1: aprobada con correcciones por los dos revisores)

- **rust-wasm-reviewer**: 4 MENORES (triples sin fijar, `dir_target` sin test, bordes de `sumas` y `es_macos`, `leer_orden`). Todos corregidos en e368253.
- **code-quality-reviewer**: 8 MENORES.
  - Corregidos en e368253:
    - MEN-1: borrador antes de publicar.
    - MEN-2: `--signer-workflow`.
    - MEN-3: tag estable sobre main.
    - MEN-4: alpine por digest.
    - MEN-5: verificacion en macOS y Windows.
    - MEN-7: `LC_ALL=C`.
    - MEN-8: ensayo sobre el commit final.
  - **No se corrige MEN-6** (la tabla de targets repetida en las 3 matrices): las matrices de Actions son estaticas, y generarlas desde `xtask` anade otro punto de fallo por un riesgo que ya contiene la CI. Se reconsidera en F5-1.
- Correccion propia en la autorrevision: `CARGO_TARGET_DIR` relativo se resuelve desde la raiz (7527334).

### Riesgos aceptados y lo no verificado

- MEN-3 (tag estable fuera de main) y la recuperacion de un borrador a medias no se han probado en CI: haria falta un tag estable real. Se estrenan en TASK-072.
- La linea de verificacion para macOS de la doc no se ha ejecutado en un Mac.
- Sin Authenticode ni Developer ID (D4a): SmartScreen y la cuarentena de macOS siguen ahi.
- Los runners de GitHub no son maquinas limpias de verdad; esa prueba es de TASK-071.
- Sigue sin `--remap-path-prefix`: el hash de referencia es siempre el `SHA256SUMS` de la CI (lo dice la doc).
- **Observacion ajena al diff**: `crates/tc2/tests/plataforma.rs` y `plataforma_gitlab.rs` dan `sinred` de forma intermitente bajo carga. Lo vieron los dos revisores y tambien la suite local, y en solitario pasan. Es la fragilidad preexistente del mock HTTP en Windows.

## Transiciones

| fecha | fase | modo | decidido_por | version |
|---|---|---|---|---|
| 2026-11-16T00:44:28Z | plan | automatico | persona | 0.16.1 |
| 2026-11-16T01:04:04Z | approve | automatico | persona | 0.16.1 |
| 2026-11-16T01:04:14Z | start | automatico | persona | 0.16.1 |
| 2026-11-16T01:35:08Z | review | automatico | persona | 0.16.1 |
| 2026-11-16T02:15:12Z | finish | automatico | persona | 0.16.1 |

## Coste

| fecha | fase | fuente | entrada | cache_escritura | cache_lectura | salida | lecturas | total | ronda | revisor |
|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-16T00:50:33Z | diseno | agente:adeb100acb51b0731 | 22 | 71992 | 485910 | 13401 | 24 | 571325 | - | - |
| 2026-11-16T01:35:08Z | implementacion | sesion:44e4ecae-26d8-46b9-b8f7-1cf87902cbae | 102 | 80379 | 8714711 | 40261 | 13 | 8835453 | - | - |
| 2026-11-16T01:48:12Z | revision | agente:a421c053ad6987e2e | 60 | 132495 | 3386750 | 14941 | 9 | 3534246 | 1 | rust-wasm-reviewer |
| 2026-11-16T01:48:12Z | revision | agente:a5fb13610feaf1969 | 72 | 157097 | 4613910 | 28259 | 3 | 4799338 | 1 | code-quality-reviewer |

## Autorrevision

- rust-wasm-reviewer — 7527334
- code-quality-reviewer — 7527334

## Verificacion ligera

- 2026-11-16T02:07:12Z: ronda 1; commit revisado e531238f359fa059a30904b61c3017ec947a5318; commit verificado e368253c8f54f1ac0fb73dfd1e3144e67b8bfaa4; ficheros: .github/scripts/humo.sh, .github/workflows/release.yml, docs/distribucion/releases.md, xtask/src/dist.rs, xtask/src/main.rs; corregidos: MEN-1, MEN-2, MEN-3, MEN-4, MEN-5, MEN-7, MEN-8, MEN-1, MEN-2, MEN-3, MEN-4; pendientes: ninguno

## Duracion

- Estimada: 170 min
- Real (de la rama abierta al cierre con merge, sin pausas): 71 min
- Paso de la hora: 71 min reales frente a 170 estimados
- Cierre (finish): 425 s, de 2026-11-16T02:08:07Z a 2026-11-16T02:15:12Z
