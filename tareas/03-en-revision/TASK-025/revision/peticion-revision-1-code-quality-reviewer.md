# Peticion de revision — TASK-025 (ronda 1)

- Tarea: TASK-025 — F2-10 Build de ejecutables por sistema en la CI, firmados
- Rama revisada: feature/task-025-f2-10-build-de-ejecutables-por-sistema-e
- Rama base: develop
- Commit revisado (HEAD): e531238f359fa059a30904b61c3017ec947a5318
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-025 (criterios de aceptacion y plan)
- Alcance: completa (motivo: primera ronda de revision)

## Instrucciones para el agente revisor

Eres un revisor INDEPENDIENTE del agente que implemento. Tu trabajo es
reproducir empiricamente, no leer el diff y opinar: clona el repo a un
directorio temporal, corre la suite tu mismo (salvo que esta
peticion ya traiga la verificacion de la ronda en verde para el commit revisado;
entonces vale ese resultado) y construye el caso que
rompe el codigo antes de reportarlo. Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. Vuelca tu salida en el
informe de esta ronda (informe-revision-1-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-025/tarea.md
- tareas/03-en-revision/TASK-025/planificacion/
- .github/scripts/humo.sh
- .github/workflows/release.yml
- docs/distribucion/releases.md
- docs/plan/PLAN-TASKCODE-2.md

## Commits a revisar (git log develop..HEAD)

````
e531238 chore(TASK-025): autorrevision ronda 1
7527334 fix(TASK-025): CARGO_TARGET_DIR relativo se resuelve desde la raiz del workspace
5e5348e fix(TASK-025): git_binario no hereda el idioma del runner (CI de macOS en rojo desde TASK-024)
d4889a3 feat(TASK-025): release con build por sistema, humo sin Node, hash y attestation
77b036f chore(TASK-025): tarea en curso
````

## Diff de tu dominio (4 fichero(s) de 7; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/scripts/humo.sh b/.github/scripts/humo.sh
new file mode 100644
index 0000000..77cc87f
--- /dev/null
+++ b/.github/scripts/humo.sh
@@ -0,0 +1,73 @@
+#!/usr/bin/env bash
+# Prueba de humo de un binario publicado (TASK-025, F2-10): arranca sin Node.
+#
+# - Linux: se ejecuta dentro de un contenedor alpine, sin Node y sin glibc
+#   (el binario es musl estatico).
+# - Windows y macOS: se quitan del PATH los directorios que tienen `node` y se
+#   exige que `node` ya no se encuentre (como la PoC S de ADR F1-14); los
+#   runners de GitHub traen Node de serie.
+#
+# Comprueba que `--version` escribe exactamente «tc2 <version>» con codigo 0
+# y que sin argumentos sale con 2 y el texto de uso. Solo usa builtins de bash
+# tras limpiar el PATH.
+#
+# Uso: .github/scripts/humo.sh <binario> <version>
+set -euo pipefail
+
+if [ "$#" -ne 2 ]; then
+  echo "Uso: $0 <binario> <version>" >&2
+  exit 2
+fi
+bin="$1"
+version="$2"
+dir="$(cd "$(dirname "$bin")" && pwd)"
+nombre="$(basename "$bin")"
+chmod +x "$dir/$nombre"
+imagen="alpine:3.22"
+
+case "$(uname -s)" in
+  Linux)
+    if docker run --rm "$imagen" sh -c 'command -v node' >/dev/null; then
+      echo "humo: ROJO: hay node en $imagen" >&2
+      exit 1
+    fi
+    echo "humo: node no esta en $imagen"
+    tc2() { docker run --rm -v "$dir:/w:ro" "$imagen" "/w/$nombre" "$@"; }
+    ;;
+  *)
+    limpio=""
+    IFS=':' read -r -a dirs <<< "$PATH"
+    for d in "${dirs[@]}"; do
+      if [ -e "$d/node" ] || [ -e "$d/node.exe" ]; then
+        echo "humo: fuera del PATH: $d"
+        continue
+      fi
+      limpio="${limpio:+$limpio:}$d"
+    done
+    PATH="$limpio"
+    if command -v node >/dev/null 2>&1; then
+      echo "humo: ROJO: node sigue en el PATH: $(command -v node)" >&2
+      exit 1
+    fi
+    echo "humo: node no esta en el PATH"
+    tc2() { "$dir/$nombre" "$@"; }
+    ;;
+esac
+
+salida="$(tc2 --version)"
+if [ "$salida" != "tc2 $version" ]; then
+  echo "humo: ROJO: --version escribio «$salida», se esperaba «tc2 $version»" >&2
+  exit 1
+fi
+echo "humo: $salida"
+
+set +e
+uso="$(tc2 2>&1)"
+codigo=$?
+set -e
+if [ "$codigo" -ne 2 ] || [[ "$uso" != *"tc2 --version"* ]]; then
+  echo "humo: ROJO: sin argumentos salio con $codigo y «$uso»" >&2
+  exit 1
+fi
+echo "humo: sin argumentos: codigo 2 y texto de uso"
+echo "humo: VERDE: $nombre arranca sin Node"
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
new file mode 100644
index 0000000..8250065
--- /dev/null
+++ b/.github/workflows/release.yml
@@ -0,0 +1,237 @@
+# Release de TaskCodeTwo (TASK-025, F2-10; ADR F1-14 D4a, F1-3 D3a, F1-8).
+#
+# - Con tag `v*` (anotado sobre el merge en main): compila los 6 targets de
+#   P12 en runners nativos, prueba que arrancan sin Node, publica la release
+#   con los 6 binarios, `SHA256SUMS` y attestation de procedencia (Sigstore),
+#   y la vuelve a descargar desde la URL publica para comprobarla.
+# - Con `workflow_dispatch`, o con un push a una rama que toque lo que entra
+#   en la release (este workflow, el humo, xtask, Cargo): ensayo. Compila y
+#   prueba, pero no publica. (Los filtros de rutas no se aplican a los tags.)
+#
+# "Firmados" es lo de D4a: hash, attestation y firma ad hoc en macOS.
+# Authenticode y Developer ID llegan con winget y Homebrew.
+# Cada job es de una sola clase (F1-8): ninguno compila y publica a la vez.
+name: release
+
+on:
+  push:
+    tags: ["v*"]
+    branches: ["**"]
+    paths:
+      - .github/workflows/release.yml
+      - .github/scripts/humo.sh
+      - xtask/**
+      - Cargo.toml
+      - Cargo.lock
+      - rust-toolchain.toml
+      - .cargo/**
+  workflow_dispatch:
+
+permissions: {}
+
+concurrency:
+  group: release-${{ github.ref }}
+  cancel-in-progress: false
+
+defaults:
+  run:
+    shell: bash
+
+jobs:
+  preparar:
+    name: preparar
+    runs-on: ubuntu-latest
+    timeout-minutes: 10
+    permissions:
+      contents: read
+    outputs:
+      version: ${{ steps.version.outputs.version }}
+      clase: ${{ steps.version.outputs.clase }}
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+      - name: Toolchain de rust-toolchain.toml
+        run: rustup toolchain install
+      - name: Version y tag
+        id: version
+        env:
+          TAG: ${{ github.ref_name }}
+          ES_TAG: ${{ startsWith(github.ref, 'refs/tags/v') }}
+        run: |
+          version="$(cargo xtask version)"
+          if [ "$ES_TAG" = "true" ]; then
+            clase="$(cargo xtask comprobar-tag "$TAG")"
+          else
+            clase="ensayo"
+          fi
+          echo "version=$version" >> "$GITHUB_OUTPUT"
+          echo "clase=$clase" >> "$GITHUB_OUTPUT"
+          echo "version $version, release: $clase"
+
+  build:
+    name: build (${{ matrix.alias }})
+    needs: preparar
+    runs-on: ${{ matrix.runner }}
+    timeout-minutes: 40
+    permissions:
+      contents: read
+    strategy:
+      fail-fast: false
+      matrix:
+        include:
+          - { alias: win-x64, target: x86_64-pc-windows-msvc, runner: windows-latest }
+          - { alias: win-arm64, target: aarch64-pc-windows-msvc, runner: windows-11-arm }
+          - { alias: darwin-x64, target: x86_64-apple-darwin, runner: macos-15-intel }
+          - { alias: darwin-arm64, target: aarch64-apple-darwin, runner: macos-latest }
+          - { alias: linux-x64, target: x86_64-unknown-linux-musl, runner: ubuntu-latest }
+          - { alias: linux-arm64, target: aarch64-unknown-linux-musl, runner: ubuntu-24.04-arm }
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+      - name: Toolchain y target
+        env:
+          TARGET: ${{ matrix.target }}
+        run: |
+          rustup toolchain install
+          rustup target add "$TARGET"
+          rustc --version
+      - name: musl-tools (Linux)
+        if: runner.os == 'Linux'
+        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends musl-tools
+      - name: cargo xtask empaquetar
+        env:
+          TARGET: ${{ matrix.target }}
+        run: cargo xtask empaquetar --target "$TARGET" --salida dist
+      - uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
+        with:
+          name: bin-${{ matrix.alias }}
+          path: dist/
+          if-no-files-found: error
+          retention-days: 7
+
+  humo:
+    name: humo (${{ matrix.alias }})
+    needs: [preparar, build]
+    runs-on: ${{ matrix.runner }}
+    timeout-minutes: 10
+    permissions:
+      contents: read
+    strategy:
+      fail-fast: false
+      matrix:
+        include:
+          - { alias: win-x64, runner: windows-latest, ext: .exe }
+          - { alias: win-arm64, runner: windows-11-arm, ext: .exe }
+          - { alias: darwin-x64, runner: macos-15-intel, ext: "" }
+          - { alias: darwin-arm64, runner: macos-latest, ext: "" }
+          - { alias: linux-x64, runner: ubuntu-latest, ext: "" }
+          - { alias: linux-arm64, runner: ubuntu-24.04-arm, ext: "" }
+    steps:
+      # Solo el script de humo: el binario no se compila aqui.
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+          sparse-checkout: .github/scripts
+      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
+        with:
+          name: bin-${{ matrix.alias }}
+          path: dist
+      - name: sha256 del artefacto
+        working-directory: dist
+        run: |
+          if command -v sha256sum >/dev/null; then sha256sum -c ./*.sha256; else shasum -a 256 -c ./*.sha256; fi
+      - name: Arranca sin Node
+        env:
+          FICHERO: tc2-${{ needs.preparar.outputs.version }}-${{ matrix.alias }}${{ matrix.ext }}
+          VERSION: ${{ needs.preparar.outputs.version }}
+        run: bash .github/scripts/humo.sh "dist/$FICHERO" "$VERSION"
+
+  publicar:
+    name: publicar
+    needs: [preparar, build, humo]
+    if: startsWith(github.ref, 'refs/tags/v')
+    runs-on: ubuntu-latest
+    timeout-minutes: 15
+    permissions:
+      contents: write
+      id-token: write
+      attestations: write
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+      - name: Toolchain de rust-toolchain.toml
+        run: rustup toolchain install
+      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
+        with:
+          pattern: bin-*
+          path: dist
+          merge-multiple: true
+      - name: SHA256SUMS
+        run: |
+          cargo xtask sumas dist
+          # Las sumas de la release tienen que ser las mismas que dejo cada build.
+          diff <(cat dist/*.sha256 | sort -k 2) dist/SHA256SUMS
+          rm dist/*.sha256
+          test "$(ls dist/tc2-* | wc -l)" -eq 6
+      - uses: actions/attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8 # v4.2.2
+        with:
+          subject-path: dist/tc2-*
+      - name: Crear la release
+        env:
+          GH_TOKEN: ${{ github.token }}
+          TAG: ${{ github.ref_name }}
+          CLASE: ${{ needs.preparar.outputs.clase }}
+        run: |
+          extra=()
+          if [ "$CLASE" = "prerelease" ]; then extra+=(--prerelease); fi
+          gh release create "$TAG" dist/tc2-* dist/SHA256SUMS \
+            --repo "$GITHUB_REPOSITORY" --verify-tag --title "$TAG" "${extra[@]}" \
+            --notes "Binarios de tc2 para los 6 sistemas de P12, con SHA256SUMS y attestation de procedencia. Verificacion: docs/distribucion/releases.md"
+
+  verificar-release:
+    name: verificar-release (${{ matrix.alias }})
+    needs: [preparar, publicar]
+    runs-on: ${{ matrix.runner }}
+    timeout-minutes: 10
+    permissions:
+      contents: read
+      attestations: read
+    strategy:
+      fail-fast: false
+      matrix:
+        include:
+          - { alias: win-x64, runner: windows-latest, ext: .exe }
+          - { alias: win-arm64, runner: windows-11-arm, ext: .exe }
+          - { alias: darwin-x64, runner: macos-15-intel, ext: "" }
+          - { alias: darwin-arm64, runner: macos-latest, ext: "" }
+          - { alias: linux-x64, runner: ubuntu-latest, ext: "" }
+          - { alias: linux-arm64, runner: ubuntu-24.04-arm, ext: "" }
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+          sparse-checkout: .github/scripts
+      - name: Descarga anonima desde la URL publica
+        env:
+          BASE: https://github.com/${{ github.repository }}/releases/download/${{ github.ref_name }}
+          FICHERO: tc2-${{ needs.preparar.outputs.version }}-${{ matrix.alias }}${{ matrix.ext }}
+        run: |
+          mkdir -p pub && cd pub
+          curl -fsSL --retry 3 -o SHA256SUMS "$BASE/SHA256SUMS"
+          curl -fsSL --retry 3 -o "$FICHERO" "$BASE/$FICHERO"
+          grep -F "  $FICHERO" SHA256SUMS > una.sha256
+          test "$(wc -l < una.sha256)" -eq 1
+          if command -v sha256sum >/dev/null; then sha256sum -c una.sha256; else shasum -a 256 -c una.sha256; fi
+      - name: gh attestation verify
+        env:
+          GH_TOKEN: ${{ github.token }}
+          FICHERO: tc2-${{ needs.preparar.outputs.version }}-${{ matrix.alias }}${{ matrix.ext }}
+        run: gh attestation verify "pub/$FICHERO" --repo "$GITHUB_REPOSITORY"
+      - name: Arranca sin Node
+        env:
+          FICHERO: tc2-${{ needs.preparar.outputs.version }}-${{ matrix.alias }}${{ matrix.ext }}
+          VERSION: ${{ needs.preparar.outputs.version }}
+        run: bash .github/scripts/humo.sh "pub/$FICHERO" "$VERSION"
diff --git a/docs/distribucion/releases.md b/docs/distribucion/releases.md
new file mode 100644
index 0000000..2b92f38
--- /dev/null
+++ b/docs/distribucion/releases.md
@@ -0,0 +1,77 @@
+# Releases de `tc2`: cómo se publican y cómo se verifican
+
+Lo construyó TASK-025 (F2-10). Aplica lo decidido en
+[ADR F1-14](../adr/F1-14-distribucion-e-instalacion.md) (D4a) y en
+[ADR F1-3](../adr/F1-3-flujo-de-ramas-worktrees-release.md) (D3a).
+
+## Qué trae cada release
+
+- Un binario por cada uno de los 6 targets de P12, con el nombre
+  `tc2-<version>-<alias>[.exe]`:
+
+  | alias | triple de Rust | runner |
+  |---|---|---|
+  | win-x64 | x86_64-pc-windows-msvc | windows-latest |
+  | win-arm64 | aarch64-pc-windows-msvc | windows-11-arm |
+  | darwin-x64 | x86_64-apple-darwin | macos-15-intel |
+  | darwin-arm64 | aarch64-apple-darwin | macos-latest |
+  | linux-x64 | x86_64-unknown-linux-musl | ubuntu-latest |
+  | linux-arm64 | aarch64-unknown-linux-musl | ubuntu-24.04-arm |
+
+  En Linux el binario es musl estático: no depende de la glibc del sistema.
+- `SHA256SUMS`, con el formato de `sha256sum`.
+- Una attestation de procedencia de GitHub (Sigstore) por binario.
+- En macOS, firma ad hoc (`codesign --sign -`).
+
+"Firmado" significa eso (D4a). Authenticode y Developer ID con notarización
+llegan con winget y Homebrew, así que en esta versión Windows puede mostrar
+SmartScreen y macOS pone en cuarentena lo que se descarga con el navegador.
+
+## Cómo se publica
+
+El workflow `.github/workflows/release.yml` se lanza de dos formas:
+
+- **Tag `v<version>`** (o `v<version>-<sufijo>`, que se publica como
+  prerelease), anotado sobre el merge en `main`. `<version>` tiene que ser la
+  de `Cargo.toml`: si no coinciden, el job `preparar` falla y no se publica
+  nada.
+- **`workflow_dispatch`, o un push a una rama que toque lo que entra en la
+  release** (el workflow, el humo, `xtask`, Cargo): ensayo. Compila y prueba
+  los 6 binarios, pero no publica.
+
+Jobs, en orden:
+
+1. `build`: compila cada target en un runner nativo con
+   `cargo xtask empaquetar`.
+2. `humo`: comprueba que cada binario arranca sin Node, con
+   `.github/scripts/humo.sh`.
+3. `publicar`: genera `SHA256SUMS`, crea la attestation y la release.
+4. `verificar-release`: vuelve a descargar los binarios desde la URL pública,
+   sin credenciales, y comprueba todo otra vez.
+
+Si un solo target falla, no se publica la release.
+
+Los mismos pasos se pueden ejecutar en local:
+
+```bash
+cargo xtask empaquetar --target x86_64-pc-windows-msvc --salida dist
+cargo xtask sumas dist
+cargo xtask comprobar-tag v0.0.0
+```
+
+## El hash de referencia es el de la CI
+
+Un build local no da el mismo sha256 que el del runner (TASK-016: rutas del
+checkout distintas). El hash que vale es **siempre** el del `SHA256SUMS` de la
+release, y `cli.lock` (F2-12) se rellena desde ese fichero, nunca desde un
+build local.
+
+## Cómo se verifica un binario descargado
+
+```bash
+sha256sum -c --ignore-missing SHA256SUMS
+```
+
+```bash
+gh attestation verify tc2-<version>-<alias> --repo charliebk/TaskCodeTwo
+```
diff --git a/docs/plan/PLAN-TASKCODE-2.md b/docs/plan/PLAN-TASKCODE-2.md
index 3a85d65..063c021 100644
--- a/docs/plan/PLAN-TASKCODE-2.md
+++ b/docs/plan/PLAN-TASKCODE-2.md
@@ -334,7 +334,7 @@ dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode
 | F2-9 | Adaptador GitLab (MR, pipelines, approvals, identidad) con la misma interfaz y la misma batería E2E (RR-15) | E2E contra un proyecto de pruebas en GitLab | F1-13, F2-2 |
 | F2-7 | Configuración mínima con esquema validado y valores por defecto | ≤ 12 claves; esquema publicado; tests | F1-10 |
 | F2-8 | i18n de mensajes (es, en) | Test que falla si falta una traducción | F2-1 |
-| F2-10 | Build de ejecutables por sistema en la CI, firmados, con su hash, publicados en cada release (RR-16) | El ejecutable arranca en Windows, macOS y Linux limpios sin Node | F1-14, F2-1 |
+| F2-10 | Build de ejecutables por sistema en la CI, con su hash, publicados en cada release (RR-16); firmados desde la primera release con attestation de procedencia y firma ad hoc en macOS, Authenticode y Developer ID al entrar en winget y Homebrew (ADR F1-14, D4a) | El ejecutable arranca en Windows, macOS y Linux limpios sin Node | F1-14, F2-1 |
 | F2-11 | Instaladores de una línea (PowerShell y shell) que descargan, verifican la firma o el hash y ponen `taskctl` en el PATH | Instalación en una máquina limpia de cada sistema en menos de un minuto | F2-10 |
 | F2-12 | `taskctl actualizar --cli` y aviso de versión nueva; el plugin del marketplace comprueba la versión del ejecutable | Actualización sin reinstalar; incompatibilidad avisada con el comando que la arregla | F2-10 |
 | F2-13 | Identidad de la persona: el «quién» de una tarea es el usuario de GitHub o GitLab autenticado, no un texto libre (RR-12) | Dos personas no pueden figurar con el mismo nombre; el agente figura como agente | F1-6, F2-6, F2-9 |
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-025/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-025/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-025/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
