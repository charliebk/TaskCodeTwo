# Peticion de revision — TASK-026 (ronda 1)

- Tarea: TASK-026 — F2-11 Instaladores de una linea
- Rama revisada: feature/task-026-f2-11-instaladores-de-una-linea
- Rama base: develop
- Commit revisado (HEAD): 6609149e64b240c697b4eba4036d5e8006865279
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-026 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-026/tarea.md
- tareas/03-en-revision/TASK-026/planificacion/
- .github/scripts/probar-instalador.sh
- .github/workflows/release.yml
- docs/distribucion/instalar.md
- docs/distribucion/releases.md
- docs/plan/PLAN-TASKCODE-2.md
- instalar/install.ps1
- instalar/install.sh

## Commits a revisar (git log develop..HEAD)

````
6609149 chore(TASK-026): constancia de autorrevision
f309ec2 fix(TASK-026): TC2_INSTALL_DIR relativo se hace absoluto antes de ir al PATH
4e365f9 fix(TASK-026): el coste de PowerShell del runner entra en la medida
29ebd59 fix(TASK-026): medir en Windows con el PSModulePath de un Windows limpio
a359612 test(TASK-026): desglose de la carga del modulo de irm en Windows
15042a8 fix(TASK-026): el arnes no cambia LOCALAPPDATA al medir en Windows
c886987 test(TASK-026): desglose del tiempo de instalacion en Windows
5db2dfe fix(TASK-026): el primer arranque de powershell.exe queda fuera de la medida
ab7a9cb feat(TASK-026): instaladores de una linea install.sh e install.ps1
fe585a6 chore(TASK-026): tarea en curso
````

## Diff de tu dominio (7 fichero(s) de 9; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/scripts/probar-instalador.sh b/.github/scripts/probar-instalador.sh
new file mode 100644
index 0000000..6395bc0
--- /dev/null
+++ b/.github/scripts/probar-instalador.sh
@@ -0,0 +1,207 @@
+#!/bin/sh
+# Prueba de los instaladores de una linea (TASK-026, F2-11). POSIX sh: corre
+# en los runners (Windows con el sh de Git) y dentro de un contenedor alpine.
+#
+# Uso: probar-instalador.sh local|publico <url> <TC2_VERSION> <version-del-binario>
+#
+# - <url>: donde estan install.sh e install.ps1.
+# - local: la release servida en 127.0.0.1 (<url> tiene tambien SHA256SUMS y
+#   el binario, y <url>/malo los mismos con un SHA256SUMS alterado). Mide la
+#   instalacion, comprueba que un hash que no casa no instala nada y que el
+#   PATH queda una sola vez aunque se instale dos veces.
+# - publico: la linea real contra la release publicada. Mide la instalacion y
+#   comprueba que sin TC2_VERSION instala la ultima estable o, si no la hay,
+#   falla diciendo que se use TC2_VERSION.
+#
+# La medida va desde antes de lanzar la linea hasta que un proceso nuevo, que
+# lee el PATH como una terminal nueva (sh -l en Unix, el registro en Windows),
+# encuentra tc2 y escribe exactamente "tc2 <version>". Limite: 60 s.
+set -eu
+
+if [ "$#" -ne 4 ]; then
+  echo "Uso: $0 local|publico <url> <TC2_VERSION> <version-del-binario>" >&2
+  exit 2
+fi
+modo=$1
+url=${2%/}
+export TC2_VERSION="$3"
+version_bin=$4
+LIMITE=60
+tmp=$(mktemp -d)
+unset TC2_BASE_URL TC2_INSTALL_DIR TC2_NO_MODIFY_PATH || true
+
+rojo() {
+  echo "probar: ROJO: $1" >&2
+  exit 1
+}
+
+case "$(uname -s)" in
+  MINGW* | MSYS* | CYGWIN*) so=windows ;;
+  *) so=unix ;;
+esac
+
+# --- Unix: la linea de sh con un HOME nuevo (sin perfil) -------------------
+
+linea_unix() { # <home>
+  if command -v curl >/dev/null 2>&1; then
+    curl -fsSL "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
+  else
+    wget -qO- "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
+  fi
+}
+
+# Un proceso nuevo, con el entorno vacio, que lee el perfil como un login.
+terminal_nueva_unix() { # <home>
+  env -i HOME="$1" PATH=/usr/bin:/bin sh -l -c 'command -v tc2 && tc2 --version'
+}
+
+# --- Windows: la linea de PowerShell 5.1 ----------------------------------
+#
+# Sin cambiar LOCALAPPDATA: ahi guarda PowerShell su cache de modulos, y con
+# uno vacio la reconstruye en cada proceso (unos 40 s en windows-11-arm), algo
+# que no le pasa a nadie que instala. La medida usa el destino por defecto; el
+# resto de pruebas aislan el destino con TC2_INSTALL_DIR.
+
+linea_windows() { # [dir de instalacion en formato Windows]
+  if [ -n "${1:-}" ]; then
+    TC2_INSTALL_DIR="$1" powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm '$url/install.ps1' | iex"
+  else
+    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm '$url/install.ps1' | iex"
+  fi
+}
+
+# Un proceso nuevo que lee el PATH del registro, como una ventana nueva.
+terminal_nueva_windows() {
+  powershell.exe -NoProfile -Command "\$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User'); \$c = Get-Command tc2 -ErrorAction Stop; \$c.Source; & \$c.Source --version" | tr -d '\r'
+}
+
+veces_en_path_usuario() { # <dir en formato Windows>
+  powershell.exe -NoProfile -Command "@(([Environment]::GetEnvironmentVariable('Path','User') -split ';') | Where-Object { \$_.TrimEnd('\\') -ieq '$1' }).Count" | tr -d '\r'
+}
+
+dir_windows() { # <ruta de sh>
+  cygpath -w "$1"
+}
+
+# --- Las pruebas -----------------------------------------------------------
+
+medir() {
+  if [ "$so" = windows ]; then
+    # La persona escribe la linea en una ventana de PowerShell ya abierta: el
+    # primer arranque de powershell.exe en un runner recien creado (unos 12 s
+    # en windows-11-arm) no es parte de la instalacion. Se paga aqui, fuera de
+    # la medida, y se deja en el log.
+    frio=$(date +%s)
+    powershell.exe -NoProfile -Command 'exit 0'
+    echo "probar: primer arranque de powershell.exe: $(($(date +%s) - frio)) s (fuera de la medida)"
+    # Solo informativo, y DENTRO de la medida: lo que tarda en este runner un
+    # proceso nuevo de PowerShell que usa un cmdlet de
+    # Microsoft.PowerShell.Utility (irm lo es), sin el instalador. En
+    # windows-11-arm son 42-50 s (TASK-026; no depende de LOCALAPPDATA ni de
+    # los modulos de Azure del PSModulePath); en windows-latest, 0-1 s.
+    t=$(date +%s)
+    powershell.exe -NoProfile -Command "\$null = Get-Date" >/dev/null
+    echo "probar: coste del runner: un proceso nuevo de PowerShell con un cmdlet de Utility tarda $(($(date +%s) - t)) s (incluido en la medida)"
+  fi
+  inicio=$(date +%s)
+  if [ "$so" = windows ]; then
+    linea_windows
+    encontrado=$(terminal_nueva_windows)
+    esperado_dir="$LOCALAPPDATA\\taskcodetwo\\bin\\tc2.exe"
+  else
+    mkdir -p "$tmp/home-medida"
+    linea_unix "$tmp/home-medida"
+    encontrado=$(terminal_nueva_unix "$tmp/home-medida")
+    esperado_dir="$tmp/home-medida/.local/bin/tc2"
+  fi
+  fin=$(date +%s)
+  echo "$encontrado"
+  ruta=$(printf '%s\n' "$encontrado" | head -n 1)
+  salida=$(printf '%s\n' "$encontrado" | tail -n 1)
+  [ "$(printf '%s' "$ruta" | tr '[:upper:]' '[:lower:]')" = "$(printf '%s' "$esperado_dir" | tr '[:upper:]' '[:lower:]')" ] ||
+    rojo "la terminal nueva encuentra tc2 en '$ruta', se esperaba '$esperado_dir'"
+  [ "$salida" = "tc2 $version_bin" ] || rojo "tc2 --version escribio '$salida', se esperaba 'tc2 $version_bin'"
+  segundos=$((fin - inicio))
+  echo "probar: instalacion en ${segundos} s (limite ${LIMITE} s)"
+  [ "$segundos" -lt "$LIMITE" ] || rojo "la instalacion tardo ${segundos} s"
+  echo "probar: VERDE: instalacion medida"
+}
+
+hash_alterado() {
+  export TC2_BASE_URL="$url/malo"
+  if [ "$so" = windows ]; then
+    if linea_windows "$(dir_windows "$tmp/dest-malo")"; then rojo "con un hash alterado el instalador salio con 0"; fi
+    [ ! -e "$tmp/dest-malo/tc2.exe" ] || rojo "con un hash alterado quedo tc2.exe instalado"
+  else
+    mkdir -p "$tmp/home-malo"
+    if linea_unix "$tmp/home-malo"; then rojo "con un hash alterado el instalador salio con 0"; fi
+    [ ! -e "$tmp/home-malo/.local/bin/tc2" ] || rojo "con un hash alterado quedo tc2 instalado"
+  fi
+  export TC2_BASE_URL="$url"
+  echo "probar: VERDE: un hash que no casa no instala nada"
+}
+
+path_idempotente() {
+  if [ "$so" = windows ]; then
+    dest=$(dir_windows "$tmp/dest-doble")
+    linea_windows "$dest"
+    linea_windows "$dest"
+    n=$(veces_en_path_usuario "$dest")
+    [ "$n" = 1 ] || rojo "tras instalar dos veces, el dir esta $n veces en el PATH del usuario"
+    dest=$(dir_windows "$tmp/dest-sin-path")
+    (export TC2_NO_MODIFY_PATH=1; linea_windows "$dest")
+    [ -e "$tmp/dest-sin-path/tc2.exe" ] || rojo "con TC2_NO_MODIFY_PATH=1 no se instalo tc2.exe"
+    n=$(veces_en_path_usuario "$dest")
+    [ "$n" = 0 ] || rojo "con TC2_NO_MODIFY_PATH=1 el dir esta $n veces en el PATH del usuario"
+  else
+    h="$tmp/home-doble"
+    mkdir -p "$h"
+    linea_unix "$h"
+    linea_unix "$h"
+    n=$(grep -cF '# >>> tc2 (TaskCodeTwo) >>>' "$h/.profile") || n=0
+    [ "$n" = 1 ] || rojo "tras instalar dos veces, .profile tiene $n bloques de tc2"
+    h="$tmp/home-sin-path"
+    mkdir -p "$h"
+    (export TC2_NO_MODIFY_PATH=1; linea_unix "$h")
+    [ ! -e "$h/.profile" ] || rojo "con TC2_NO_MODIFY_PATH=1 se escribio $h/.profile"
+    [ -x "$h/.local/bin/tc2" ] || rojo "con TC2_NO_MODIFY_PATH=1 no se instalo tc2"
+  fi
+  echo "probar: VERDE: el PATH queda una sola vez y TC2_NO_MODIFY_PATH=1 no lo toca"
+}
+
+sin_version() {
+  unset TC2_VERSION
+  set +e
+  if [ "$so" = windows ]; then
+    salida=$(linea_windows "$(dir_windows "$tmp/dest-ultima")" 2>&1)
+  else
+    mkdir -p "$tmp/home-ultima"
+    salida=$(linea_unix "$tmp/home-ultima" 2>&1)
+  fi
+  codigo=$?
+  set -e
+  echo "$salida"
+  if [ "$codigo" -ne 0 ]; then
+    case "$salida" in
+      *"no hay ninguna release estable"*TC2_VERSION*) ;;
+      *) rojo "sin TC2_VERSION fallo sin decir que se use TC2_VERSION" ;;
+    esac
+  fi
+  echo "probar: VERDE: sin TC2_VERSION, ultima estable o error con el arreglo"
+}
+
+case "$modo" in
+  local)
+    export TC2_BASE_URL="$url"
+    medir
+    hash_alterado
+    path_idempotente
+    ;;
+  publico)
+    medir
+    sin_version
+    ;;
+  *) rojo "modo desconocido: $modo" ;;
+esac
+rm -rf "$tmp"
+echo "probar: VERDE: instalador de $so ($modo)"
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index 1c846cd..285df2e 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -3,7 +3,11 @@
 # - Con tag `v*` (anotado sobre el merge en main): compila los 6 targets de
 #   P12 en runners nativos, prueba que arrancan sin Node, publica la release
 #   con los 6 binarios, `SHA256SUMS` y attestation de procedencia (Sigstore),
-#   y la vuelve a descargar desde la URL publica para comprobarla.
+#   y la vuelve a descargar desde la URL publica para comprobarla. Publica
+#   tambien los instaladores de una linea (`instalar/`, TASK-026, F2-11) y
+#   ejecuta la linea real en cada sistema, midiendo que tarda menos de 60 s.
+# - Antes de publicar nada, `instaladores-ensayo` prueba los instaladores
+#   contra los binarios del build servidos en 127.0.0.1.
 # - Con `workflow_dispatch`, o con un push a una rama que toque lo que entra
 #   en la release (este workflow, el humo, xtask, Cargo): ensayo. Compila y
 #   prueba, pero no publica. (Los filtros de rutas no se aplican a los tags.)
@@ -20,6 +24,8 @@ on:
     paths:
       - .github/workflows/release.yml
       - .github/scripts/humo.sh
+      - .github/scripts/probar-instalador.sh
+      - instalar/**
       - xtask/**
       - Cargo.toml
       - Cargo.lock
@@ -157,9 +163,74 @@ jobs:
           VERSION: ${{ needs.preparar.outputs.version }}
         run: bash .github/scripts/humo.sh "dist/$FICHERO" "$VERSION"
 
+  # Los instaladores contra los binarios del build, servidos en 127.0.0.1:
+  # instalacion medida, hash alterado y PATH idempotente
+  # (.github/scripts/probar-instalador.sh). Windows en PowerShell 5.1; Linux
+  # tambien en alpine (sin curl, sin bash), con la imagen de humo.sh.
+  instaladores-ensayo:
+    name: instaladores-ensayo (${{ matrix.alias }}${{ matrix.alpine && ', alpine' || '' }})
+    needs: [preparar, build]
+    runs-on: ${{ matrix.runner }}
+    timeout-minutes: 10
+    permissions:
+      contents: read
+    strategy:
+      fail-fast: false
+      matrix:
+        include:
+          - { alias: win-x64, runner: windows-latest, alpine: false }
+          - { alias: win-arm64, runner: windows-11-arm, alpine: false }
+          - { alias: darwin-x64, runner: macos-15-intel, alpine: false }
+          - { alias: darwin-arm64, runner: macos-latest, alpine: false }
+          - { alias: linux-x64, runner: ubuntu-latest, alpine: false }
+          - { alias: linux-arm64, runner: ubuntu-24.04-arm, alpine: false }
+          - { alias: linux-x64, runner: ubuntu-latest, alpine: true }
+          - { alias: linux-arm64, runner: ubuntu-24.04-arm, alpine: true }
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+          sparse-checkout: |
+            .github/scripts
+            instalar
+      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
+        with:
+          name: bin-${{ matrix.alias }}
+          path: srv
+      - name: Release de ensayo (y otra con el hash alterado)
+        working-directory: srv
+        run: |
+          cat ./*.sha256 > SHA256SUMS
+          rm ./*.sha256
+          cp ../instalar/install.sh ../instalar/install.ps1 .
+          mkdir malo
+          cp tc2-* malo/
+          awk '{ c = substr($1, 1, 1); print (c == "0" ? "1" : "0") substr($1, 2) "  " $2 }' SHA256SUMS > malo/SHA256SUMS
+          ! cmp -s SHA256SUMS malo/SHA256SUMS
+      - name: Instalador servido en 127.0.0.1
+        env:
+          VERSION: ${{ needs.preparar.outputs.version }}
+          ALPINE: ${{ matrix.alpine }}
+        run: |
+          py=python
+          command -v python >/dev/null 2>&1 || py=python3
+          "$py" -m http.server 8765 --bind 127.0.0.1 --directory srv >/dev/null 2>&1 &
+          for _ in $(seq 1 30); do
+            curl -fsS -o /dev/null http://127.0.0.1:8765/SHA256SUMS && break
+            sleep 1
+          done
+          if [ "$ALPINE" = "true" ]; then
+            imagen="$(sed -n 's/^imagen="\(.*\)"$/\1/p' .github/scripts/humo.sh)"
+            test -n "$imagen"
+            docker run --rm --network host -v "$PWD:/w:ro" -w /w "$imagen" \
+              sh .github/scripts/probar-instalador.sh local http://127.0.0.1:8765 "$VERSION" "$VERSION"
+          else
+            sh .github/scripts/probar-instalador.sh local http://127.0.0.1:8765 "$VERSION" "$VERSION"
+          fi
+
   publicar:
     name: publicar
-    needs: [preparar, build, humo]
+    needs: [preparar, build, humo, instaladores-ensayo]
     if: startsWith(github.ref, 'refs/tags/v')
     runs-on: ubuntu-latest
     timeout-minutes: 15
@@ -186,9 +257,15 @@ jobs:
           diff <(cat dist/*.sha256 | LC_ALL=C sort -k 2) dist/SHA256SUMS
           rm dist/*.sha256
           test "$(ls dist/tc2-* | wc -l)" -eq 6
+          # Los instaladores van en la release y en la attestation, pero no en
+          # SHA256SUMS: es el contrato de los 6 binarios (cli.lock, F2-12).
+          cp instalar/install.sh instalar/install.ps1 dist/
       - uses: actions/attest-build-provenance@4d101475d8b20a2381f78447822ac1eab6504dd8 # v4.2.2
         with:
-          subject-path: dist/tc2-*
+          subject-path: |
+            dist/tc2-*
+            dist/install.sh
+            dist/install.ps1
       - name: Crear la release
         env:
           GH_TOKEN: ${{ github.token }}
@@ -200,10 +277,10 @@ jobs:
         run: |
           extra=()
           if [ "$CLASE" = "prerelease" ]; then extra+=(--prerelease); fi
-          gh release create "$TAG" dist/tc2-* dist/SHA256SUMS --draft \
+          gh release create "$TAG" dist/tc2-* dist/SHA256SUMS dist/install.sh dist/install.ps1 --draft \
             --repo "$GITHUB_REPOSITORY" --verify-tag --title "$TAG" "${extra[@]}" \
-            --notes "Binarios de tc2 para los 6 sistemas de P12, con SHA256SUMS y attestation de procedencia. Verificacion: docs/distribucion/releases.md"
-          test "$(gh release view "$TAG" --repo "$GITHUB_REPOSITORY" --json assets --jq '.assets | length')" -eq 7
+            --notes "Binarios de tc2 para los 6 sistemas de P12, con SHA256SUMS, attestation de procedencia e instaladores de una linea. Instalacion: docs/distribucion/instalar.md. Verificacion: docs/distribucion/releases.md"
+          test "$(gh release view "$TAG" --repo "$GITHUB_REPOSITORY" --json assets --jq '.assets | length')" -eq 9
           gh release edit "$TAG" --repo "$GITHUB_REPOSITORY" --draft=false
 
   verificar-release:
@@ -252,3 +329,49 @@ jobs:
           FICHERO: tc2-${{ needs.preparar.outputs.version }}-${{ matrix.alias }}${{ matrix.ext }}
           VERSION: ${{ needs.preparar.outputs.version }}
         run: bash .github/scripts/humo.sh "pub/$FICHERO" "$VERSION"
+
+  # La linea real contra la release publicada, en cada sistema: mide que la
+  # instalacion tarda menos de 60 s hasta que una terminal nueva encuentra tc2
+  # (criterio 1 de TASK-026). Con gh y token, comprueba tambien la
+  # attestation; en alpine no hay gh y se comprueba el aviso.
+  verificar-instaladores:
+    name: verificar-instaladores (${{ matrix.alias }}${{ matrix.alpine && ', alpine' || '' }})
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
+          - { alias: win-x64, runner: windows-latest, alpine: false }
+          - { alias: win-arm64, runner: windows-11-arm, alpine: false }
+          - { alias: darwin-x64, runner: macos-15-intel, alpine: false }
+          - { alias: darwin-arm64, runner: macos-latest, alpine: false }
+          - { alias: linux-x64, runner: ubuntu-latest, alpine: false }
+          - { alias: linux-arm64, runner: ubuntu-24.04-arm, alpine: false }
+          - { alias: linux-x64, runner: ubuntu-latest, alpine: true }
+          - { alias: linux-arm64, runner: ubuntu-24.04-arm, alpine: true }
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+          sparse-checkout: .github/scripts
+      - name: La linea de instalacion real
+        env:
+          GH_TOKEN: ${{ github.token }}
+          URL: https://github.com/${{ github.repository }}/releases/download/${{ github.ref_name }}
+          TAG: ${{ github.ref_name }}
+          VERSION: ${{ needs.preparar.outputs.version }}
+          ALPINE: ${{ matrix.alpine }}
+        run: |
+          if [ "$ALPINE" = "true" ]; then
+            imagen="$(sed -n 's/^imagen="\(.*\)"$/\1/p' .github/scripts/humo.sh)"
+            test -n "$imagen"
+            docker run --rm -v "$PWD:/w:ro" -w /w "$imagen" \
+              sh .github/scripts/probar-instalador.sh publico "$URL" "${TAG#v}" "$VERSION"
+          else
+            sh .github/scripts/probar-instalador.sh publico "$URL" "${TAG#v}" "$VERSION"
+          fi
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
new file mode 100644
index 0000000..81accf8
--- /dev/null
+++ b/docs/distribucion/instalar.md
@@ -0,0 +1,106 @@
+# Instalar `tc2` con una línea
+
+Lo construyó TASK-026 (F2-11). Aplica lo decidido en
+[ADR F1-14](../adr/F1-14-distribucion-e-instalacion.md) (D4a y punto 6). Cómo
+se publican y verifican las releases: [releases.md](releases.md).
+
+## Las dos líneas
+
+Windows (PowerShell 5.1 o 7, sin permisos de administrador):
+
+```powershell
+irm https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.ps1 | iex
+```
+
+macOS y Linux (cualquier `sh`, con `curl` o `wget`):
+
+```bash
+curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.sh | sh
+```
+
+`latest` es la última release **estable**: mientras no haya ninguna (antes de
+la 2.0.0), el instalador lo dice y pide fijar la versión. Una versión concreta,
+o una prerelease, se instala con el instalador de esa release y `TC2_VERSION`:
+
+```powershell
+$env:TC2_VERSION = '0.0.0-prueba.1'; irm https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-prueba.1/install.ps1 | iex
+```
+
+```bash
+curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-prueba.1/install.sh | TC2_VERSION=0.0.0-prueba.1 sh
+```
+
+## Qué hace
+
+1. Detecta el sistema y la arquitectura: uno de los 6 alias de P12. En macOS
+   bajo Rosetta, y en una PowerShell x64 emulada en Windows ARM64, elige el
+   binario nativo arm64.
+2. Baja `SHA256SUMS` y el binario `tc2-<version>-<alias>[.exe]` de la release.
+3. Exige que `SHA256SUMS` tenga exactamente una línea para ese fichero y que el
+   sha256 case. Si no casa, sale con error y no instala nada.
+4. Si hay `gh` con sesión, comprueba también la attestation de procedencia
+   (`gh attestation verify --signer-workflow .../release.yml`). Sin `gh`, lo
+   avisa en una línea y sigue: el hash solo prueba que el fichero llegó entero;
+   que lo construyó este repositorio lo prueba la attestation.
+5. Comprueba que el binario arranca (`tc2 --version`) antes de dejarlo en su
+   sitio: nunca queda en el destino un binario sin comprobar.
+6. Lo deja en el PATH:
+   - **Windows**: `%LOCALAPPDATA%\taskcodetwo\bin\tc2.exe` y ese directorio al
+     final del PATH del usuario (registro, sin tocar las `%VARIABLES%` que ya
+     tenga), además de en la sesión actual.
+   - **macOS y Linux**: `~/.local/bin/tc2`. Si ese directorio no está en el
+     PATH, añade este bloque a `~/.profile` y, según `$SHELL`, a `~/.zshrc` o
+     a `~/.bashrc` (y a `~/.bash_profile` si existe):
+
+     ```sh
+     # >>> tc2 (TaskCodeTwo) >>>
+     case ":$PATH:" in *":/home/ana/.local/bin:"*) ;; *) export PATH="/home/ana/.local/bin:$PATH" ;; esac
+     # <<< tc2 (TaskCodeTwo) <<<
+     ```
+
+     Instalar otra vez no repite el bloque. `tc2 desinstalar` (F2-14) lo quita
+     por sus marcas. Con fish, el instalador dice qué ejecutar.
+
+Volver a ejecutar la línea reinstala (o cambia de versión con `TC2_VERSION`).
+Actualizar con `tc2 actualizar --cli` es de F2-12. No toca `taskctl` ni nada
+de TaskCode 0.16.
+
+## Variables
+
+| variable | qué hace |
+|---|---|
+| `TC2_VERSION` | versión que se instala (`2.0.0`, `2.0.0-rc.1`...); por defecto, la última estable |
+| `TC2_INSTALL_DIR` | directorio del binario |
+| `TC2_NO_MODIFY_PATH=1` | no toca el PATH del usuario ni los perfiles del shell |
+| `TC2_BASE_URL` | solo para ensayos: URL con los ficheros de la release (https, o http a `127.0.0.1` o `localhost`); exige `TC2_VERSION` y no comprueba la attestation |
+
+Código de salida: 0 instalado; distinto de 0 con una línea `ERROR` y otra
+`arreglo`. El script de PowerShell nunca usa `exit` (con `irm | iex` cerraría
+la ventana): lanza el error.
+
+## Cómo se prueba
+
+En cada cambio, el job `instaladores-ensayo` de `release.yml` sirve en
+`127.0.0.1` los binarios recién compilados y ejecuta
+`.github/scripts/probar-instalador.sh local` en los 6 runners nativos
+(Windows con `powershell.exe` 5.1) y en alpine (sin curl ni bash):
+
+- instalación medida: desde la línea hasta que una terminal nueva encuentra
+  `tc2` y escribe `tc2 <version>`, en menos de 60 s. En Windows, el primer
+  arranque de `powershell.exe` en un runner recién creado (unos 12 s en
+  `windows-11-arm`) queda fuera: la persona escribe la línea en una ventana
+  ya abierta. Se mide aparte y queda en el log. Sí entra en la medida lo que
+  tarda PowerShell en un proceso nuevo en encontrar `irm`: en
+  `windows-11-arm` son 42-50 s (en `windows-latest`, 0-1 s), con lo que la
+  instalación allí mide unos 44 s, aunque el instalador en sí tarda menos de
+  1 s. El log lo deja aparte para verlo;
+- con un `SHA256SUMS` alterado, sale con error y no queda `tc2`;
+- dos instalaciones dejan una sola entrada en el PATH, y
+  `TC2_NO_MODIFY_PATH=1` no lo toca.
+
+Al publicar, `verificar-instaladores` ejecuta la línea real contra la release
+en los mismos sistemas, con la misma medida, y comprueba la attestation donde
+hay `gh`.
+
+Los runners no son máquinas limpias (traen curl, gh y Node). La prueba en
+máquinas limpias de verdad es de F9-9a (TASK-071).
diff --git a/docs/distribucion/releases.md b/docs/distribucion/releases.md
index a74a96a..97ede57 100644
--- a/docs/distribucion/releases.md
+++ b/docs/distribucion/releases.md
@@ -4,6 +4,8 @@ Lo construyó TASK-025 (F2-10). Aplica lo decidido en
 [ADR F1-14](../adr/F1-14-distribucion-e-instalacion.md) (D4a) y en
 [ADR F1-3](../adr/F1-3-flujo-de-ramas-worktrees-release.md) (D3a).
 
+Para instalar `tc2` con una línea, ver [instalar.md](instalar.md) (TASK-026).
+
 ## Qué trae cada release
 
 - Un binario por cada uno de los 6 targets de P12, con el nombre
@@ -22,6 +24,10 @@ Lo construyó TASK-025 (F2-10). Aplica lo decidido en
 - `SHA256SUMS`, con el formato de `sha256sum`.
 - Una attestation de procedencia de GitHub (Sigstore) por binario.
 - En macOS, firma ad hoc (`codesign --sign -`).
+- Los instaladores de una línea, `install.sh` e `install.ps1`
+  ([instalar.md](instalar.md)). Tienen attestation, pero no van en
+  `SHA256SUMS`, que es el contrato de los 6 binarios del que se rellena
+  `cli.lock` (F2-12). En total, 9 assets.
 
 "Firmado" significa eso (D4a). Authenticode y Developer ID con notarización
 llegan con winget y Homebrew, así que en esta versión Windows puede mostrar
@@ -36,8 +42,8 @@ El workflow `.github/workflows/release.yml` se lanza de dos formas:
   de `Cargo.toml`, y un tag estable tiene que estar sobre un commit de `main`:
   si no se cumple, el job `preparar` falla y no se publica nada.
 - **`workflow_dispatch`, o un push a una rama que toque lo que entra en la
-  release** (el workflow, el humo, `xtask`, Cargo): ensayo. Compila y prueba
-  los 6 binarios, pero no publica.
+  release** (el workflow, el humo, los instaladores, `xtask`, Cargo): ensayo.
+  Compila y prueba los 6 binarios y los instaladores, pero no publica.
 
 Jobs, en orden:
 
@@ -45,12 +51,16 @@ Jobs, en orden:
    `cargo xtask empaquetar`.
 2. `humo`: comprueba que cada binario arranca sin Node, con
    `.github/scripts/humo.sh`.
-3. `publicar`: genera `SHA256SUMS`, crea la attestation y la release, primero
+3. `instaladores-ensayo`: prueba los instaladores de una línea contra los
+   binarios del build servidos en `127.0.0.1` ([instalar.md](instalar.md)).
+4. `publicar`: genera `SHA256SUMS`, crea la attestation y la release, primero
    como borrador y después la publica.
-4. `verificar-release`: vuelve a descargar los binarios desde la URL pública,
+5. `verificar-release`: vuelve a descargar los binarios desde la URL pública,
    sin credenciales, y comprueba todo otra vez.
+6. `verificar-instaladores`: ejecuta la línea de instalación real contra la
+   release en cada sistema y mide que tarda menos de 60 s.
 
-Si un solo target falla, no se publica la release.
+Si un solo target o un instalador falla, no se publica la release.
 
 Si `publicar` se corta a medias, queda como mucho un borrador, que no ven los
 demás. Para reintentar, borra el borrador (`gh release delete <tag>`, sin
diff --git a/docs/plan/PLAN-TASKCODE-2.md b/docs/plan/PLAN-TASKCODE-2.md
index 063c021..96c24f2 100644
--- a/docs/plan/PLAN-TASKCODE-2.md
+++ b/docs/plan/PLAN-TASKCODE-2.md
@@ -335,7 +335,7 @@ dependencias. Las fases 1 a 9 son tareas de TaskCodeTwo; la 0, del repo TaskCode
 | F2-7 | Configuración mínima con esquema validado y valores por defecto | ≤ 12 claves; esquema publicado; tests | F1-10 |
 | F2-8 | i18n de mensajes (es, en) | Test que falla si falta una traducción | F2-1 |
 | F2-10 | Build de ejecutables por sistema en la CI, con su hash, publicados en cada release (RR-16); firmados desde la primera release con attestation de procedencia y firma ad hoc en macOS, Authenticode y Developer ID al entrar en winget y Homebrew (ADR F1-14, D4a) | El ejecutable arranca en Windows, macOS y Linux limpios sin Node | F1-14, F2-1 |
-| F2-11 | Instaladores de una línea (PowerShell y shell) que descargan, verifican la firma o el hash y ponen `taskctl` en el PATH | Instalación en una máquina limpia de cada sistema en menos de un minuto | F2-10 |
+| F2-11 | Instaladores de una línea (PowerShell y shell), publicados en cada release, que descargan, comprueban el hash y, si hay `gh`, la attestation, y ponen `tc2` en el PATH (ADR F1-14, divergencias 3 y 5) | Instalación en menos de un minuto en cada sistema, medida en la CI en los 6 runners nativos y en alpine; las máquinas limpias de verdad, en F9-9a | F2-10 |
 | F2-12 | `taskctl actualizar --cli` y aviso de versión nueva; el plugin del marketplace comprueba la versión del ejecutable | Actualización sin reinstalar; incompatibilidad avisada con el comando que la arregla | F2-10 |
 | F2-13 | Identidad de la persona: el «quién» de una tarea es el usuario de GitHub o GitLab autenticado, no un texto libre (RR-12) | Dos personas no pueden figurar con el mismo nombre; el agente figura como agente | F1-6, F2-6, F2-9 |
 | F2-14 | `taskctl desinstalar`: quita TaskCodeTwo de forma limpia (plugin, CLI, status line, hooks, worktree de la rama de estado); en un proyecto, la config solo con confirmación y nunca las tareas ni su historial sin confirmación explícita; con `--simular` (RR-16) | Tras desinstalar no queda nada de TaskCodeTwo en la máquina ni en el proyecto salvo lo que la persona decide conservar | F2-3, F2-10, F2-12 |
diff --git a/instalar/install.ps1 b/instalar/install.ps1
new file mode 100644
index 0000000..ee7f5ea
--- /dev/null
+++ b/instalar/install.ps1
@@ -0,0 +1,251 @@
+# Instalador de una linea de tc2 (TaskCodeTwo) para Windows
+# (TASK-026, F2-11; ADR F1-14).
+#
+#   irm https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.ps1 | iex
+#
+# Baja de la release el binario de este sistema y SHA256SUMS, comprueba el
+# sha256 y, si hay `gh` con sesion, la attestation de procedencia. Despues deja
+# `tc2.exe` en %LOCALAPPDATA%\taskcodetwo\bin y anade ese directorio al PATH
+# del usuario, sin permisos de administrador. Nunca deja en el destino un
+# binario sin comprobar. Funciona en Windows PowerShell 5.1 (el de un Windows
+# limpio) y en PowerShell 7. No toca `taskctl` ni nada de TaskCode 0.16.
+#
+# Variables (todas opcionales):
+#   TC2_VERSION         version que se instala (2.0.0, 2.0.0-rc.1...); por
+#                       defecto, la ultima release estable.
+#   TC2_INSTALL_DIR     directorio del binario.
+#   TC2_NO_MODIFY_PATH  con 1, no se toca el PATH del usuario.
+#   TC2_BASE_URL        solo para ensayos: URL con los ficheros de la release
+#                       (https, o http a 127.0.0.1 o localhost). Exige
+#                       TC2_VERSION y no comprueba la attestation.
+#
+# Nunca usa `exit`: con `irm | iex` cerraria la ventana de la persona. Los
+# errores se lanzan con throw (codigo 1 con -File o -Command). Todo va dentro
+# de Install-Tc2, que se llama en la ultima linea: una descarga cortada no
+# ejecuta medio script.
+
+function Install-Tc2 {
+    Set-StrictMode -Version 2.0
+    $ErrorActionPreference = 'Stop'
+    # Sin barra de progreso, Invoke-WebRequest de la 5.1 es mucho mas rapido.
+    $ProgressPreference = 'SilentlyContinue'
+    $repo = 'charliebk/TaskCodeTwo'
+
+    function Decir([string]$texto) {
+        Write-Host "tc2-instalar: $texto"
+    }
+
+    function Fallo([string]$problema, [string]$arreglo) {
+        throw "tc2-instalar: ERROR: $problema`ntc2-instalar: arreglo: $arreglo"
+    }
+
+    # Un ejecutable externo, sin que su stderr se vuelva un error terminante
+    # (en la 5.1 lo hace con ErrorActionPreference = Stop).
+    function Invoke-Externo([string]$exe, [string[]]$argumentos) {
+        $ErrorActionPreference = 'Continue'
+        $salida = (& $exe @argumentos 2>&1 | ForEach-Object { "$_" }) -join "`n"
+        return @{ Codigo = $LASTEXITCODE; Salida = $salida.Trim() }
+    }
+
+    function Get-Descarga([string]$url, [string]$fichero) {
+        for ($intento = 1; $intento -le 3; $intento++) {
+            try {
+                Invoke-WebRequest -Uri $url -OutFile $fichero -UseBasicParsing
+                return
+            } catch {
+                if ($intento -eq 3) {
+                    Fallo "no se pudo bajar $url ($($_.Exception.Message))" "comprueba la version en https://github.com/$repo/releases, la conexion y el proxy"
+                }
+                Start-Sleep -Seconds 2
+            }
+        }
+    }
+
+    # La ultima release estable, sin la API (tiene limite anonimo): la
+    # redireccion de /releases/latest a /releases/tag/v<version>. Sin ninguna
+    # release estable, GitHub redirige a /releases. Si no hay respuesta, el
+    # fallo es de red (conexion, proxy, certificado), no de version.
+    function Get-UltimaVersion {
+        $url = "https://github.com/$repo/releases/latest"
+        $destino = $null
+        try {
+            $peticion = [Net.HttpWebRequest]::Create($url)
+            $peticion.Method = 'HEAD'
+            $peticion.AllowAutoRedirect = $false
+            $respuesta = $peticion.GetResponse()
+            $destino = $respuesta.Headers['Location']
+            $respuesta.Close()
+        } catch [Net.WebException] {
+            if ($null -eq $_.Exception.Response) {
+                Fallo "no se pudo consultar $url ($($_.Exception.Message))" "comprueba la conexion, el proxy y los certificados, o fija la version con `$env:TC2_VERSION = '<version>'"
+            }
+            $destino = $_.Exception.Response.Headers['Location']
+        }
+        if ("$destino" -match '/releases/tag/v([0-9A-Za-z.\-]+)$') {
+            return $Matches[1]
+        }
+        Fallo 'no hay ninguna release estable de tc2 todavia' "elige una version de https://github.com/$repo/releases: `$env:TC2_VERSION = '<version>'; irm <url de install.ps1> | iex"
+    }
+
+    function Get-AliasP12 {
+        # El valor nativo del registro: una PowerShell x64 emulada en un
+        # Windows ARM64 baja igualmente el binario arm64.
+        $clave = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
+        $nativa = (Get-ItemProperty -Path $clave -Name PROCESSOR_ARCHITECTURE).PROCESSOR_ARCHITECTURE
+        switch ($nativa) {
+            'AMD64' { return 'win-x64' }
+            'ARM64' { return 'win-arm64' }
+            default { Fallo "arquitectura no soportada: $nativa" 'tc2 se publica para Windows en x64 y arm64' }
+        }
+    }
+
+    function Add-PathUsuario([string]$dir) {
+        $normal = $dir.TrimEnd('\')
+        if ($env:TC2_NO_MODIFY_PATH -eq '1') {
+            Decir "TC2_NO_MODIFY_PATH=1: no se toca el PATH; anade tu $dir"
+            return
+        }
+        # Se lee y se escribe en crudo para no expandir las %VARIABLES% que ya
+        # tenga el PATH de la persona.
+        $clave = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
+        try {
+            $actual = [string]$clave.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
+            $tipo = [Microsoft.Win32.RegistryValueKind]::ExpandString
+            if ($clave.GetValueNames() -contains 'Path') {
+                $tipo = $clave.GetValueKind('Path')
+            }
+            $entradas = @($actual -split ';' | Where-Object { $_ -ne '' })
+            $ya = @($entradas | Where-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') -ieq $normal })
+            if ($ya.Count -gt 0) {
+                Decir "$dir ya esta en el PATH del usuario"
+            } else {
+                $clave.SetValue('Path', (($entradas + $dir) -join ';'), $tipo)
+                # Avisa a las ventanas abiertas (WM_SETTINGCHANGE): .NET lo
+                # difunde al cambiar una variable de usuario.
+                [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', '1', 'User')
+                [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', $null, 'User')
+                Decir "anadido $dir al PATH del usuario"
+            }
+        } finally {
+            $clave.Close()
+        }
+        # Y a esta sesion, para usar tc2 sin abrir otra ventana.
+        if (-not (@($env:Path -split ';') | Where-Object { $_.TrimEnd('\') -ieq $normal })) {
+            $env:Path = "$env:Path;$dir"
+        }
+    }
+
+    if ($PSVersionTable.PSEdition -eq 'Core' -and -not $IsWindows) {
+        Fallo 'install.ps1 es para Windows' "en macOS y Linux: curl -fsSL https://github.com/$repo/releases/latest/download/install.sh | sh"
+    }
+    # La 5.1 no ofrece TLS 1.2 por defecto en todos los sistemas.
+    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
+
+    $comprobarAttestation = $true
+    if ($env:TC2_BASE_URL) {
+        if ($env:TC2_BASE_URL -notmatch '^(https://|http://(127\.0\.0\.1|localhost)(:|/))') {
+            Fallo "TC2_BASE_URL tiene que ser https, o http a 127.0.0.1 o localhost: $env:TC2_BASE_URL" 'corrige TC2_BASE_URL o quitala'
+        }
+        if (-not $env:TC2_VERSION) {
+            Fallo 'TC2_BASE_URL exige TC2_VERSION' "define `$env:TC2_VERSION = '<version>'"
+        }
+        $comprobarAttestation = $false
+    }
+
+    $version = $env:TC2_VERSION
+    if (-not $version) {
+        $version = Get-UltimaVersion
+    }
+    $version = $version -replace '^v', ''
+    if ($version -notmatch '^[0-9A-Za-z.\-]+$') {
+        Fallo "version no valida: '$version'" 'usa una version como 2.0.0 o 2.0.0-rc.1'
+    }
+    # Un tag de prerelease es v<version>-<sufijo>; el binario lleva <version>.
+    $versionBin = ($version -split '-', 2)[0]
+    $base = $env:TC2_BASE_URL
+    if (-not $base) {
+        $base = "https://github.com/$repo/releases/download/v$version"
+    }
+    $base = $base.TrimEnd('/')
+
+    $aliasP12 = Get-AliasP12
+    $fichero = "tc2-$versionBin-$aliasP12.exe"
+    $destino = $env:TC2_INSTALL_DIR
+    if (-not $destino) {
+        $destino = Join-Path $env:LOCALAPPDATA 'taskcodetwo\bin'
+    }
+    # Absoluto (respecto al directorio actual de PowerShell): el directorio
+    # acaba en el PATH del usuario.
+    $destino = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($destino)
+
+    $tmp = Join-Path ([IO.Path]::GetTempPath()) ('tc2-instalar-' + [guid]::NewGuid().ToString('N'))
+    New-Item -ItemType Directory -Path $tmp | Out-Null
+    try {
+        Decir "instalando tc2 $version ($aliasP12) desde $base"
+        $sumas = Join-Path $tmp 'SHA256SUMS'
+        Get-Descarga "$base/SHA256SUMS" $sumas
+        $esperados = @()
+        foreach ($linea in [IO.File]::ReadAllLines($sumas)) {
+            $partes = $linea -split '  ', 2
+            if ($partes.Count -eq 2 -and $partes[1] -ceq $fichero) {
+                $esperados += $partes[0]
+            }
+        }
+        if ($esperados.Count -ne 1) {
+            Fallo "SHA256SUMS no trae una sola linea para $fichero ($($esperados.Count))" "esa release no tiene binario para ${aliasP12}: prueba otra TC2_VERSION"
+        }
+        $descargado = Join-Path $tmp $fichero
+        Get-Descarga "$base/$fichero" $descargado
+        $real = (Get-FileHash -LiteralPath $descargado -Algorithm SHA256).Hash.ToLowerInvariant()
+        if ($real -cne $esperados[0]) {
+            Fallo "el sha256 de $fichero no casa con SHA256SUMS (esperado $($esperados[0]), obtenido $real); no se ha instalado nada" "repite la descarga; si sigue, avisa en https://github.com/$repo/issues"
+        }
+        Decir "sha256 correcto: $real"
+
+        if (-not $comprobarAttestation) {
+            Decir 'TC2_BASE_URL: no se comprueba la attestation (solo ensayos)'
+        } elseif ((Get-Command gh -ErrorAction SilentlyContinue) -and (Invoke-Externo 'gh' @('auth', 'status')).Codigo -eq 0) {
+            $r = Invoke-Externo 'gh' @('attestation', 'verify', $descargado, '--repo', $repo, '--signer-workflow', "$repo/.github/workflows/release.yml")
+            if ($r.Codigo -ne 0) {
+                Fallo "la attestation de procedencia de $fichero no es valida; no se ha instalado nada ($($r.Salida))" "avisa en https://github.com/$repo/issues"
+            }
+            Decir "attestation correcta: lo construyo $repo con release.yml"
+        } else {
+            Decir 'aviso: sin gh con sesion no se comprueba la attestation (el sha256 si); ver docs/distribucion/instalar.md'
+        }
+
+        # Se prueba en el temporal: un .exe recien ejecutado puede quedar
+        # bloqueado un momento (el antivirus lo analiza) y no se podria mover.
+        $r = Invoke-Externo $descargado @('--version')
+        if ($r.Codigo -ne 0 -or $r.Salida -cne "tc2 $versionBin") {
+            Fallo "tc2 --version escribio '$($r.Salida)' (codigo $($r.Codigo)), se esperaba 'tc2 $versionBin'; no se ha instalado nada" "avisa en https://github.com/$repo/issues"
+        }
+        $nuevo = Join-Path $destino ".tc2.nuevo.$PID.exe"
+        try {
+            New-Item -ItemType Directory -Force -Path $destino | Out-Null
+            Copy-Item -LiteralPath $descargado -Destination $nuevo -Force
+        } catch {
+            Fallo "no se pudo escribir en $destino ($($_.Exception.Message))" 'usa $env:TC2_INSTALL_DIR con un directorio con permiso de escritura'
+        }
+        $final = Join-Path $destino 'tc2.exe'
+        for ($intento = 1; $true; $intento++) {
+            try {
+                Move-Item -LiteralPath $nuevo -Destination $final -Force
+                break
+            } catch {
+                if ($intento -ge 10) {
+                    Remove-Item -LiteralPath $nuevo -Force -ErrorAction SilentlyContinue
+                    Fallo "no se pudo dejar tc2.exe en $destino ($($_.Exception.Message))" 'cierra lo que este usando tc2.exe y repite'
+                }
+                Start-Sleep -Milliseconds 500
+            }
+        }
+
+        Add-PathUsuario $destino
+        Decir "instalado: $final ($($r.Salida))"
+    } finally {
+        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
+    }
+}
+
+Install-Tc2
diff --git a/instalar/install.sh b/instalar/install.sh
new file mode 100644
index 0000000..5de519f
--- /dev/null
+++ b/instalar/install.sh
@@ -0,0 +1,258 @@
+#!/bin/sh
+# Instalador de una linea de tc2 (TaskCodeTwo) para macOS y Linux
+# (TASK-026, F2-11; ADR F1-14).
+#
+#   curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.sh | sh
+#
+# Baja de la release el binario de este sistema y SHA256SUMS, comprueba el
+# sha256 y, si hay `gh` con sesion, la attestation de procedencia. Despues deja
+# `tc2` en ~/.local/bin y, si ese directorio no esta en el PATH, anade al
+# perfil del shell un bloque entre marcas (lo quita `tc2 desinstalar`, F2-14).
+# Nunca deja en el destino un binario sin comprobar. Solo POSIX sh: ni bash ni
+# Node. No toca `taskctl` ni nada de TaskCode 0.16.
+#
+# Variables (todas opcionales):
+#   TC2_VERSION         version que se instala (2.0.0, 2.0.0-rc.1...); por
+#                       defecto, la ultima release estable.
+#   TC2_INSTALL_DIR     directorio del binario (por defecto ~/.local/bin).
+#   TC2_NO_MODIFY_PATH  con 1, no se toca el perfil del shell.
+#   TC2_BASE_URL        solo para ensayos: URL con los ficheros de la release
+#                       (https, o http a 127.0.0.1 o localhost). Exige
+#                       TC2_VERSION y no comprueba la attestation.
+#
+# Codigo de salida: 0 instalado; 1 error, con una linea de arreglo.
+# Todo va dentro de `main`, que se llama en la ultima linea: una descarga
+# cortada no ejecuta medio script.
+
+set -eu
+
+REPO="charliebk/TaskCodeTwo"
+MARCA_INICIO="# >>> tc2 (TaskCodeTwo) >>>"
+MARCA_FIN="# <<< tc2 (TaskCodeTwo) <<<"
+
+decir() {
+  printf 'tc2-instalar: %s\n' "$1"
+}
+
+fallo() {
+  printf 'tc2-instalar: ERROR: %s\n' "$1" >&2
+  printf 'tc2-instalar: arreglo: %s\n' "$2" >&2
+  exit 1
+}
+
+hay() {
+  command -v "$1" >/dev/null 2>&1
+}
+
+# descargar <url> <fichero>: solo se aceptan redirecciones a https.
+descargar() {
+  if hay curl; then
+    curl -fsSL --proto-redir '=https' --retry 3 -o "$2" "$1"
+  elif hay wget; then
+    wget -q -O "$2" "$1"
+  else
+    fallo "no hay curl ni wget" "instala curl (apt-get install curl, apk add curl...) y repite"
+  fi
+}
+
+# La ultima release estable, sin la API (tiene limite anonimo): la redireccion
+# de /releases/latest a /releases/tag/v<version>.
+#
+# Sin ninguna release estable, GitHub redirige a /releases. Si no hay
+# respuesta, el fallo es de red (conexion, proxy, certificado), no de version.
+ultima_version() {
+  url="https://github.com/$REPO/releases/latest"
+  final=""
+  conectado=0
+  if hay curl; then
+    if final=$(curl -fsSLI --proto-redir '=https' --retry 3 -o /dev/null -w '%{url_effective}' "$url"); then
+      conectado=1
+    fi
+  elif hay wget; then
+    cabeceras=$(wget -S --spider "$url" 2>&1) || true
+    case "$cabeceras" in
+      *HTTP/*) conectado=1 ;;
+    esac
+    final=$(printf '%s\n' "$cabeceras" | sed -n 's/^ *[Ll]ocation: *//p' | tail -n 1)
+  else
+    fallo "no hay curl ni wget" "instala curl (apt-get install curl, apk add curl...) y repite"
+  fi
+  if [ "$conectado" = 0 ]; then
+    fallo "no se pudo consultar $url" "comprueba la conexion, el proxy y los certificados, o fija la version con TC2_VERSION=<version>"
+  fi
+  final=$(printf '%s' "$final" | tr -d '\r')
+  case "$final" in
+    */releases/tag/v*) printf '%s\n' "${final##*/releases/tag/v}" ;;
+    *) fallo "no hay ninguna release estable de tc2 todavia" \
+      "elige una version de https://github.com/$REPO/releases: curl -fsSL <url de install.sh> | TC2_VERSION=<version> sh" ;;
+  esac
+}
+
+detectar_alias() {
+  so=$(uname -s)
+  arq=$(uname -m)
+  # Un shell x64 bajo Rosetta en un Mac arm64: se instala el binario nativo.
+  if [ "$so" = Darwin ] && [ "$arq" = x86_64 ]; then
+    if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = 1 ]; then
+      arq=arm64
+    fi
+  fi
+  case "$so/$arq" in
+    Linux/x86_64 | Linux/amd64) echo linux-x64 ;;
+    Linux/aarch64 | Linux/arm64) echo linux-arm64 ;;
+    Darwin/x86_64) echo darwin-x64 ;;
+    Darwin/arm64 | Darwin/aarch64) echo darwin-arm64 ;;
+    MINGW* | MSYS* | CYGWIN*)
+      fallo "install.sh es para macOS y Linux" \
+        "en Windows, desde PowerShell: irm https://github.com/$REPO/releases/latest/download/install.ps1 | iex" ;;
+    *) fallo "sistema no soportado: $so/$arq" "tc2 se publica para Windows, macOS y Linux en x64 y arm64" ;;
+  esac
+}
+
+sha256_de() {
+  if hay sha256sum; then
+    sha256sum "$1" | cut -d ' ' -f 1
+  elif hay shasum; then
+    shasum -a 256 "$1" | cut -d ' ' -f 1
+  else
+    fallo "no hay sha256sum ni shasum" "instala coreutils y repite"
+  fi
+}
+
+# escribir_bloque <perfil> <dir>: idempotente, entre marcas.
+escribir_bloque() {
+  if [ -f "$1" ] && grep -qF "$MARCA_INICIO" "$1"; then
+    decir "$1 ya tiene el bloque de tc2"
+    return 0
+  fi
+  # El $PATH del bloque se expande al leer el perfil, no ahora.
+  # shellcheck disable=SC2016
+  {
+    printf '\n%s\n' "$MARCA_INICIO"
+    printf 'case ":$PATH:" in *":%s:"*) ;; *) export PATH="%s:$PATH" ;; esac\n' "$2" "$2"
+    printf '%s\n' "$MARCA_FIN"
+  } >>"$1" || fallo "no se pudo escribir en $1" "anade tu mismo $2 al PATH, o repite con TC2_NO_MODIFY_PATH=1"
+  decir "anadido $2 al PATH en $1"
+}
+
+anadir_al_path() {
+  case ":${PATH:-}:" in
+    *":$1:"*) return 0 ;;
+  esac
+  if [ "${TC2_NO_MODIFY_PATH:-}" = 1 ]; then
+    decir "$1 no esta en el PATH y TC2_NO_MODIFY_PATH=1: anadelo tu (export PATH=\"$1:\$PATH\")"
+    return 0
+  fi
+  # .profile lo leen los shells de login POSIX (sh, dash, ash, bash sin
+  # .bash_profile); zsh y bash interactivo leen el suyo.
+  escribir_bloque "$HOME/.profile" "$1"
+  case "$(basename "${SHELL:-sh}")" in
+    zsh) escribir_bloque "$HOME/.zshrc" "$1" ;;
+    bash)
+      escribir_bloque "$HOME/.bashrc" "$1"
+      if [ -f "$HOME/.bash_profile" ]; then
+        escribir_bloque "$HOME/.bash_profile" "$1"
+      fi
+      ;;
+    fish) decir "fish no lee esos perfiles: ejecuta fish_add_path $1" ;;
+    *) ;;
+  esac
+  decir "abre otra terminal (o ejecuta: export PATH=\"$1:\$PATH\") para usar tc2"
+}
+
+main() {
+  comprobar_attestation=1
+  if [ -n "${TC2_BASE_URL:-}" ]; then
+    case "$TC2_BASE_URL" in
+      https://* | http://127.0.0.1:* | http://127.0.0.1/* | http://localhost:* | http://localhost/*) ;;
+      *) fallo "TC2_BASE_URL tiene que ser https, o http a 127.0.0.1 o localhost: $TC2_BASE_URL" "corrige TC2_BASE_URL o quitala" ;;
+    esac
+    if [ -z "${TC2_VERSION:-}" ]; then
+      fallo "TC2_BASE_URL exige TC2_VERSION" "define TC2_VERSION=<version>"
+    fi
+    comprobar_attestation=0
+  fi
+
+  version=${TC2_VERSION:-}
+  if [ -z "$version" ]; then
+    version=$(ultima_version)
+  fi
+  version=${version#v}
+  case "$version" in
+    '' | *[!0-9A-Za-z.-]*) fallo "version no valida: '$version'" "usa una version como 2.0.0 o 2.0.0-rc.1" ;;
+  esac
+  # Un tag de prerelease es v<version>-<sufijo>; el binario lleva <version>.
+  version_bin=${version%%-*}
+  base=${TC2_BASE_URL:-https://github.com/$REPO/releases/download/v$version}
+  base=${base%/}
+
+  alias_p12=$(detectar_alias)
+  fichero="tc2-$version_bin-$alias_p12"
+  if [ -n "${TC2_INSTALL_DIR:-}" ]; then
+    destino=$TC2_INSTALL_DIR
+  elif [ -n "${HOME:-}" ]; then
+    destino="$HOME/.local/bin"
+  else
+    fallo "no hay HOME" "define TC2_INSTALL_DIR=<directorio>"
+  fi
+  # Absoluto: el directorio acaba en el PATH y en el perfil del shell.
+  case "$destino" in
+    /*) ;;
+    *) destino="$(pwd)/$destino" ;;
+  esac
+
+  tmp=$(mktemp -d 2>/dev/null || mktemp -d -t tc2-instalar)
+  trap 'rm -rf "$tmp"' EXIT
+  trap 'rm -rf "$tmp"; exit 1' INT TERM
+
+  decir "instalando tc2 $version ($alias_p12) desde $base"
+  descargar "$base/SHA256SUMS" "$tmp/SHA256SUMS" ||
+    fallo "no se pudo bajar $base/SHA256SUMS" "comprueba la version en https://github.com/$REPO/releases y la conexion"
+  esperado=$(awk -v f="$fichero" '$2 == f { print $1 }' "$tmp/SHA256SUMS")
+  lineas=$(printf '%s\n' "$esperado" | grep -c .) || lineas=0
+  if [ "$lineas" -ne 1 ]; then
+    fallo "SHA256SUMS no trae una sola linea para $fichero ($lineas)" "esa release no tiene binario para $alias_p12: prueba otra TC2_VERSION"
+  fi
+  descargar "$base/$fichero" "$tmp/$fichero" ||
+    fallo "no se pudo bajar $base/$fichero" "repite; si sigue, comprueba la conexion o el proxy"
+  real=$(sha256_de "$tmp/$fichero")
+  if [ "$real" != "$esperado" ]; then
+    fallo "el sha256 de $fichero no casa con SHA256SUMS (esperado $esperado, obtenido $real); no se ha instalado nada" \
+      "repite la descarga; si sigue, avisa en https://github.com/$REPO/issues"
+  fi
+  decir "sha256 correcto: $real"
+
+  if [ "$comprobar_attestation" = 0 ]; then
+    decir "TC2_BASE_URL: no se comprueba la attestation (solo ensayos)"
+  elif hay gh && gh auth status >/dev/null 2>&1; then
+    gh attestation verify "$tmp/$fichero" --repo "$REPO" \
+      --signer-workflow "$REPO/.github/workflows/release.yml" >/dev/null ||
+      fallo "la attestation de procedencia de $fichero no es valida; no se ha instalado nada" \
+        "avisa en https://github.com/$REPO/issues"
+    decir "attestation correcta: lo construyo $REPO con release.yml"
+  else
+    decir "aviso: sin gh con sesion no se comprueba la attestation (el sha256 si); ver docs/distribucion/instalar.md"
+  fi
+
+  mkdir -p "$destino" || fallo "no se pudo crear $destino" "usa TC2_INSTALL_DIR=<directorio con permiso de escritura>"
+  nuevo="$destino/.tc2.nuevo.$$"
+  if ! { cp "$tmp/$fichero" "$nuevo" && chmod 755 "$nuevo"; }; then
+    rm -f "$nuevo"
+    fallo "no se pudo escribir en $destino" "usa TC2_INSTALL_DIR=<directorio con permiso de escritura>"
+  fi
+  salida=$("$nuevo" --version 2>&1) || salida="(no arranca: $salida)"
+  if [ "$salida" != "tc2 $version_bin" ]; then
+    rm -f "$nuevo"
+    fallo "tc2 --version escribio '$salida', se esperaba 'tc2 $version_bin'; no se ha instalado nada" \
+      "avisa en https://github.com/$REPO/issues"
+  fi
+  mv -f "$nuevo" "$destino/tc2" || {
+    rm -f "$nuevo"
+    fallo "no se pudo dejar tc2 en $destino" "cierra lo que este usando $destino/tc2 y repite"
+  }
+
+  anadir_al_path "$destino"
+  decir "instalado: $destino/tc2 ($salida)"
+}
+
+main "$@"
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-026/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-026/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-026/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
