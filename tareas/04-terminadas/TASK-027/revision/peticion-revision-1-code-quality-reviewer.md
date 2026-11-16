# Peticion de revision — TASK-027 (ronda 1)

- Tarea: TASK-027 — F2-12 taskctl actualizar --cli y aviso de version nueva
- Rama revisada: feature/task-027-f2-12-taskctl-actualizar-cli-y-aviso-de
- Rama base: develop
- Commit revisado (HEAD): 991b24cb3fbdf8c89c7441744cea1e1d03cc4f96
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-027 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-027/tarea.md
- tareas/03-en-revision/TASK-027/planificacion/
- .github/scripts/probar-actualizar.sh
- .github/workflows/release.yml
- docs/distribucion/instalar.md
- docs/distribucion/releases.md

## Commits a revisar (git log develop..HEAD)

````
991b24c chore(TASK-027): autorrevision con rust-wasm-reviewer y code-quality-reviewer
94fd72e fix(TASK-027): descarga del binario robusta y correcciones de autorrevision
562ac0c feat(TASK-027): cargo xtask cli-lock, job actualizar-ensayo y documentacion
183f822 feat(TASK-027): tc2 actualizar --cli sustituye el binario con vuelta atras
4573706 feat(TASK-027): descarga de la release y tc2 actualizar --cli --comprobar
2d329f6 feat(TASK-027): PROTOCOLO, tc2 version --json y tc2 compatible
6b61fd9 refactor(TASK-027): caja distribucion compartida por xtask y tc2
7177e5e chore(TASK-027): tarea en curso
````

## Diff de tu dominio (4 fichero(s) de 26; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/scripts/probar-actualizar.sh b/.github/scripts/probar-actualizar.sh
new file mode 100644
index 0000000..0e0cd26
--- /dev/null
+++ b/.github/scripts/probar-actualizar.sh
@@ -0,0 +1,127 @@
+#!/bin/sh
+# Ensayo de `tc2 actualizar --cli` (TASK-027, F2-12). POSIX sh: corre en los
+# runners (Windows con el sh de Git).
+#
+# Uso: probar-actualizar.sh <dir-vx> <version-x> <dir-vy> <version-y>
+#
+# - <dir-vx>: el binario del build (`tc2-<version-x>-<alias>[.exe]` y su
+#   `.sha256`), la version que se instala.
+# - <dir-vy>: el mismo build recompilado con la version <version-y> (mayor),
+#   la version a la que se actualiza.
+#
+# Sirve las dos releases en 127.0.0.1 (y una tercera con el hash alterado),
+# instala vX con el instalador de una linea y comprueba:
+# 1. `actualizar --cli --comprobar` da `actualizar ok actual=X disponible=Y`
+#    sin tocar el binario;
+# 2. con el hash alterado, `rechazada` (3), el binario no cambia y no quedan
+#    `.descarga` ni `.old`;
+# 3. `actualizar --cli` sustituye vX por vY: despues `tc2 --version` da vY.
+set -eu
+
+if [ "$#" -ne 4 ]; then
+  echo "Uso: $0 <dir-vx> <version-x> <dir-vy> <version-y>" >&2
+  exit 2
+fi
+vx_dir=$1
+vx=$2
+vy_dir=$3
+vy=$4
+PUERTO=8766
+raiz=$(pwd)
+tmp=$(mktemp -d)
+unset TC2_BASE_URL TC2_INSTALL_DIR TC2_NO_MODIFY_PATH TC2_VERSION || true
+
+rojo() {
+  echo "actualizar: ROJO: $1" >&2
+  exit 1
+}
+
+case "$(uname -s)" in
+  MINGW* | MSYS* | CYGWIN*) so=windows; exe_nombre=tc2.exe ;;
+  *) so=unix; exe_nombre=tc2 ;;
+esac
+
+suma() {
+  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
+}
+
+# --- Las tres releases -------------------------------------------------------
+mkdir -p "$tmp/srv/x" "$tmp/srv/y" "$tmp/srv/malo"
+cp "$vx_dir"/tc2-* "$tmp/srv/x/"
+rm -f "$tmp"/srv/x/*.sha256
+cat "$vx_dir"/*.sha256 >"$tmp/srv/x/SHA256SUMS"
+cp "$raiz/instalar/install.sh" "$raiz/instalar/install.ps1" "$tmp/srv/x/"
+cp "$vy_dir"/tc2-* "$tmp/srv/y/"
+rm -f "$tmp"/srv/y/*.sha256
+cat "$vy_dir"/*.sha256 >"$tmp/srv/y/SHA256SUMS"
+cp "$tmp"/srv/y/tc2-* "$tmp/srv/malo/"
+awk '{ c = substr($1, 1, 1); print (c == "0" ? "1" : "0") substr($1, 2) "  " $2 }' \
+  "$tmp/srv/y/SHA256SUMS" >"$tmp/srv/malo/SHA256SUMS"
+! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
+
+py=python
+command -v python >/dev/null 2>&1 || py=python3
+"$py" -m http.server "$PUERTO" --bind 127.0.0.1 --directory "$tmp/srv" >/dev/null 2>&1 &
+servidor=$!
+trap 'kill "$servidor" 2>/dev/null || true' EXIT
+for _ in $(seq 1 30); do
+  curl -fsS -o /dev/null "http://127.0.0.1:$PUERTO/x/SHA256SUMS" && break
+  sleep 1
+done
+url="http://127.0.0.1:$PUERTO"
+
+# --- Instalar vX con la linea de instalacion ---------------------------------
+inst="$tmp/inst"
+mkdir -p "$inst"
+if [ "$so" = windows ]; then
+  TC2_BASE_URL="$url/x" TC2_VERSION="$vx" TC2_NO_MODIFY_PATH=1 TC2_INSTALL_DIR="$(cygpath -w "$inst")" \
+    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm '$url/x/install.ps1' | iex"
+else
+  curl -fsSL "$url/x/install.sh" | TC2_BASE_URL="$url/x" TC2_VERSION="$vx" TC2_NO_MODIFY_PATH=1 TC2_INSTALL_DIR="$inst" sh
+fi
+exe="$inst/$exe_nombre"
+[ -x "$exe" ] || rojo "no se instalo $exe"
+v=$("$exe" --version | tr -d '\r')
+[ "$v" = "tc2 $vx" ] || rojo "la instalacion da '$v', se esperaba 'tc2 $vx'"
+antes=$(suma "$exe")
+
+# --- 1. --comprobar no toca nada ---------------------------------------------
+salida=$(TC2_BASE_URL="$url/y" "$exe" actualizar --cli --comprobar | tr -d '\r')
+[ "$salida" = "actualizar ok actual=$vx disponible=$vy" ] || rojo "--comprobar dio '$salida'"
+[ "$(suma "$exe")" = "$antes" ] || rojo "--comprobar cambio el binario"
+echo "actualizar: VERDE: --comprobar"
+
+# --- 2. hash alterado ----------------------------------------------------------
+set +e
+salida=$(TC2_BASE_URL="$url/malo" "$exe" actualizar --cli 2>&1)
+codigo=$?
+set -e
+echo "$salida"
+[ "$codigo" -eq 3 ] || rojo "con el hash alterado salio con $codigo, se esperaba 3"
+case "$salida" in
+  *"motivo=sha256-no-casa"*) ;;
+  *) rojo "con el hash alterado fallo, pero no por el hash" ;;
+esac
+[ "$(suma "$exe")" = "$antes" ] || rojo "con el hash alterado cambio el binario"
+restos=$(find "$inst" -mindepth 1 ! -name "$exe_nombre")
+[ -z "$restos" ] || rojo "con el hash alterado quedaron: $restos"
+echo "actualizar: VERDE: un hash que no casa no toca nada"
+
+# --- 3. vX -> vY ---------------------------------------------------------------
+salida=$(TC2_BASE_URL="$url/y" "$exe" actualizar --cli | tr -d '\r')
+echo "$salida"
+case "$salida" in
+  "actualizar ok anterior=$vx version=$vy sha256="*) ;;
+  *) rojo "actualizar --cli dio '$salida'" ;;
+esac
+v=$("$exe" --version | tr -d '\r')
+[ "$v" = "tc2 $vy" ] || rojo "tras actualizar da '$v', se esperaba 'tc2 $vy'"
+# En Windows, el .old lo borra la siguiente ejecucion (la de --version).
+restos=$(find "$inst" -mindepth 1 ! -name "$exe_nombre")
+[ -z "$restos" ] || rojo "tras actualizar quedaron: $restos"
+echo "actualizar: VERDE: vX -> vY sin reinstalar"
+
+kill "$servidor" 2>/dev/null || true
+wait "$servidor" 2>/dev/null || true
+rm -rf "$tmp"
+echo "actualizar: VERDE: $so"
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index 285df2e..8d9dfb3 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -7,7 +7,10 @@
 #   tambien los instaladores de una linea (`instalar/`, TASK-026, F2-11) y
 #   ejecuta la linea real en cada sistema, midiendo que tarda menos de 60 s.
 # - Antes de publicar nada, `instaladores-ensayo` prueba los instaladores
-#   contra los binarios del build servidos en 127.0.0.1.
+#   contra los binarios del build servidos en 127.0.0.1, y
+#   `actualizar-ensayo` (TASK-027, F2-12) instala ese binario y lo actualiza
+#   con `tc2 actualizar --cli` a otro de la version siguiente, tambien en
+#   127.0.0.1.
 # - Con `workflow_dispatch`, o con un push a una rama que toque lo que entra
 #   en la release (este workflow, el humo, xtask, Cargo): ensayo. Compila y
 #   prueba, pero no publica. (Los filtros de rutas no se aplican a los tags.)
@@ -25,8 +28,10 @@ on:
       - .github/workflows/release.yml
       - .github/scripts/humo.sh
       - .github/scripts/probar-instalador.sh
+      - .github/scripts/probar-actualizar.sh
       - instalar/**
       - xtask/**
+      - crates/distribucion/**
       - Cargo.toml
       - Cargo.lock
       - rust-toolchain.toml
@@ -228,9 +233,68 @@ jobs:
             sh .github/scripts/probar-instalador.sh local http://127.0.0.1:8765 "$VERSION" "$VERSION"
           fi
 
+  # `tc2 actualizar --cli` de punta a punta (TASK-027, F2-12): instala el
+  # binario del build (vX) con el instalador de una linea y lo actualiza a vY,
+  # el mismo codigo recompilado con la version siguiente; las dos releases (y
+  # una con el hash alterado) servidas en 127.0.0.1
+  # (.github/scripts/probar-actualizar.sh). vY solo existe aqui: no se sube ni
+  # se publica.
+  actualizar-ensayo:
+    name: actualizar-ensayo (${{ matrix.alias }})
+    needs: [preparar, build]
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
+      - name: musl-tools (Linux)
+        if: runner.os == 'Linux'
+        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends musl-tools
+      - uses: actions/download-artifact@9000827ccba6bdab643e8b6fd33ac0654aef8333 # v8.0.2
+        with:
+          name: bin-${{ matrix.alias }}
+          path: vx
+      - name: vY, el mismo codigo con la version siguiente
+        env:
+          TARGET: ${{ matrix.target }}
+          VERSION: ${{ needs.preparar.outputs.version }}
+        run: |
+          base="${VERSION%%-*}"
+          vy="$(echo "$base" | awk -F. '{ print $1 "." $2 "." $3 + 1 }')"
+          sed -i.orig "s/^version = \"$VERSION\"$/version = \"$vy\"/" Cargo.toml
+          ! cmp -s Cargo.toml Cargo.toml.orig
+          rm Cargo.toml.orig
+          # Solo cambian las entradas de los miembros del workspace.
+          cargo update --workspace
+          cargo xtask empaquetar --target "$TARGET" --salida vy
+          echo "VY=$vy" >> "$GITHUB_ENV"
+      - name: Instalar vX y actualizar a vY en 127.0.0.1
+        env:
+          VERSION: ${{ needs.preparar.outputs.version }}
+        run: sh .github/scripts/probar-actualizar.sh vx "$VERSION" vy "$VY"
+
   publicar:
     name: publicar
-    needs: [preparar, build, humo, instaladores-ensayo]
+    needs: [preparar, build, humo, instaladores-ensayo, actualizar-ensayo]
     if: startsWith(github.ref, 'refs/tags/v')
     runs-on: ubuntu-latest
     timeout-minutes: 15
diff --git a/docs/distribucion/instalar.md b/docs/distribucion/instalar.md
index 2a70a88..11c52b0 100644
--- a/docs/distribucion/instalar.md
+++ b/docs/distribucion/instalar.md
@@ -70,8 +70,71 @@ curl -fsSL https://github.com/charliebk/TaskCodeTwo/releases/download/v0.0.0-pru
    Un directorio con `;` (el separador del PATH) se rechaza.
 
 Volver a ejecutar la línea reinstala (o cambia de versión con `TC2_VERSION`).
-Actualizar con `tc2 actualizar --cli` es de F2-12. No toca `taskctl` ni nada
-de TaskCode 0.16.
+Para actualizar sin reinstalar está `tc2 actualizar --cli` (abajo). No toca
+`taskctl` ni nada de TaskCode 0.16.
+
+## Actualizar y comprobar la versión
+
+Lo construyó TASK-027 (F2-12; ADR F1-14, puntos 4 y 5).
+
+```bash
+tc2 actualizar --cli --comprobar   # ¿hay una versión nueva? No toca nada
+tc2 actualizar --cli               # sustituye el tc2 instalado por la última estable
+tc2 actualizar --cli --version 2.1.0-rc.1   # una versión concreta (también hacia atrás)
+```
+
+- **De dónde sale la versión disponible**: del `SHA256SUMS` de la última
+  release estable (`releases/latest/download/SHA256SUMS`), o de la de
+  `--version`. Una sola petición, la misma fuente que los instaladores.
+- **`--comprobar`** escribe `actualizar ok actual=X disponible=Y` si hay una
+  más nueva y `actualizar sin-cambios actual=X disponible=Y` si no; código 0
+  en los dos casos. Es el aviso de versión nueva: lo llaman `doctor` y, si se
+  quiere, un hook de inicio de sesión. Nunca se consulta la red por sí solo.
+- **Cómo sustituye**: baja el binario junto al actual
+  (`.tc2-<versión>-<pid>.descarga`), calcula su sha256 (lo calcula el `tc2`
+  viejo, nunca el binario nuevo sobre sí mismo) y lo compara con
+  `SHA256SUMS`; le pide `version --json` y comprueba versión, sistema y hash;
+  y solo entonces lo pone en su sitio. En Windows lo hace por renombrados
+  (el actual pasa a `tc2.exe.old`, el nuevo a `tc2.exe`) con vuelta atrás si
+  el segundo falla; el `tc2.exe.old` lo borra la siguiente ejecución de `tc2`.
+  Sale `actualizar ok anterior=X version=Y sha256=...`.
+- **Si algo falla, no se cambia nada** y no quedan ficheros a medias:
+  - sin red: `actualizar sinred url=...` (código 4);
+  - el sha256 no casa: `rechazada motivo=sha256-no-casa` (3);
+  - sin permiso de escritura en la carpeta del binario: `rechazada
+    motivo=sin-permiso ruta=...` (3), antes de bajar nada;
+  - el binario nuevo no arranca o no dice lo esperado: `rechazada
+    motivo=binario-no-arranca` (3);
+  - no se pudo renombrar (otro `tc2` abierto, un antivirus):
+    `error motivo=no-se-sustituye` (1), con el binario anterior en su sitio.
+- **Lo que no actualiza**: el `tc2` de la caché del plugin
+  (`.../taskcodetwo/cli/<versión>/`), que gestiona el plugin con su
+  `cli.lock` (`rechazada motivo=gestionado-por-plugin`: se actualiza el
+  plugin), ni uno instalado con un gestor de paquetes (npm, Homebrew, winget,
+  Scoop, cargo: `rechazada motivo=gestionado-por-gestor`).
+- Sin `--cli`, `tc2 actualizar` da `uso`: actualizar un proyecto es de F9-2.
+- `TC2_BASE_URL` vale igual que en los instaladores (solo ensayos): la carpeta
+  con los ficheros de una release; aquí, https o http a `127.0.0.1`.
+- Como en los instaladores, el hash prueba que el fichero llegó entero, no
+  quién lo construyó: sin Authenticode (D4a), comprometer la release
+  compromete binario y `SHA256SUMS` a la vez. La attestation se comprueba con
+  `gh attestation verify` (ver [releases.md](releases.md)).
+
+## Compatibilidad con el plugin
+
+El plugin pregunta al `tc2` que encuentra (ADR F1-14, punto 5):
+
+```bash
+tc2 version --json
+# {"version":"2.0.0","protocolo":1,"target":"win-x64","sha256":"<64 hex>"}
+tc2 compatible --protocolo 1-1
+# compatible ok protocolo=1 rango=1-1
+```
+
+Si el protocolo no cae en el rango, `compatible` sale con `rechazada` (3),
+`motivo=protocolo-incompatible` y el arreglo `tc2 actualizar --cli`. El
+`cli.lock` del plugin lo genera `cargo xtask cli-lock <SHA256SUMS> [--tag
+<tag>]` desde el `SHA256SUMS` de la release, nunca desde un build local.
 
 ## Variables
 
@@ -110,5 +173,19 @@ Al publicar, `verificar-instaladores` ejecuta la línea real contra la release
 en los mismos sistemas, con la misma medida, y comprueba la attestation donde
 hay `gh`.
 
+`tc2 actualizar --cli` se prueba en dos sitios:
+
+- `cargo test -p tc2` (`crates/tc2/tests/actualizar_cli.rs`): una copia del
+  `tc2` compilado se actualiza a un doble de la versión 9.9.9
+  (`examples/tc2_falso.rs`) servido en `127.0.0.1`; el hash alterado, sin red,
+  `--comprobar` y la caché del plugin. El fallo del segundo renombrado se
+  prueba con un fallo inyectado.
+- el job `actualizar-ensayo` de `release.yml`
+  (`.github/scripts/probar-actualizar.sh`): en los 6 runners nativos, instala
+  el binario del build con la línea de instalación y lo actualiza a otro
+  compilado con la versión siguiente, con `--comprobar` y el hash alterado.
+  Hasta el primer push con el job, **no verificado** en CI (TASK-027); su
+  paso en Windows x64 se ensayó en local.
+
 Los runners no son máquinas limpias (traen curl, gh y Node). La prueba en
 máquinas limpias de verdad es de F9-9a (TASK-071).
diff --git a/docs/distribucion/releases.md b/docs/distribucion/releases.md
index 97ede57..8eed7bc 100644
--- a/docs/distribucion/releases.md
+++ b/docs/distribucion/releases.md
@@ -81,7 +81,17 @@ cargo xtask comprobar-tag v0.0.0
 Un build local no da el mismo sha256 que el del runner (TASK-016: rutas del
 checkout distintas). El hash que vale es **siempre** el del `SHA256SUMS` de la
 release, y `cli.lock` (F2-12) se rellena desde ese fichero, nunca desde un
-build local.
+build local:
+
+```bash
+gh release download v2.0.0 --pattern SHA256SUMS --dir rel
+cargo xtask cli-lock rel/SHA256SUMS --tag v2.0.0 > cli.lock
+```
+
+`cli-lock` exige los 6 targets y que el tag case con la versión del workspace
+(como `comprobar-tag`), y escribe el rango de protocolo del CLI
+(`[PROTOCOLO, PROTOCOLO]`, de la caja `crates/distribucion`). Dónde vive el
+`cli.lock` y cómo lo usa el lanzador del plugin es de una tarea posterior.
 
 ## Cómo se verifica un binario descargado
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                |  8 ++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-027/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-027/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-027/tarea.md                                    | 16 +++++++++++++++-
 4 files changed, 23 insertions(+), 1 deletion(-)
````
