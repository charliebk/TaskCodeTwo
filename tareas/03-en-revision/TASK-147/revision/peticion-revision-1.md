# Peticion de revision — TASK-147 (ronda 1)

- Tarea: TASK-147 — Auditoria 4D: A10 scripts de CI e instaladores (4 IMPORTANTE)
- Rama revisada: fix/task-147-auditoria-4d-a10-scripts-de-ci-e-instala
- Rama base: develop
- Commit revisado (HEAD): a633ae9ba48107bb986004ac3359e4cf1434a7b8
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-147 (criterios de aceptacion y plan)
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
informe de esta ronda (informe-revision-1.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-147/tarea.md
- tareas/03-en-revision/TASK-147/planificacion/
- .github/scripts/probar-actualizar.sh
- .github/scripts/probar-instalador.sh
- .github/workflows/release.yml

## Commits a revisar (git log develop..HEAD)

````
a633ae9 chore(TASK-147): resultado y autorrevision
4e94a65 fix(TASK-147): A10-02 sin_version exige el binario y la ultima version
dd74835 fix(TASK-147): A10-01 y A11-01 guardas cmp que paran el paso
c709a41 chore(TASK-147): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/.github/scripts/probar-actualizar.sh b/.github/scripts/probar-actualizar.sh
index 0e0cd26..4876f6b 100644
--- a/.github/scripts/probar-actualizar.sh
+++ b/.github/scripts/probar-actualizar.sh
@@ -57,7 +57,7 @@ cat "$vy_dir"/*.sha256 >"$tmp/srv/y/SHA256SUMS"
 cp "$tmp"/srv/y/tc2-* "$tmp/srv/malo/"
 awk '{ c = substr($1, 1, 1); print (c == "0" ? "1" : "0") substr($1, 2) "  " $2 }' \
   "$tmp/srv/y/SHA256SUMS" >"$tmp/srv/malo/SHA256SUMS"
-! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS"
+! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS" || rojo "el SHA256SUMS alterado es igual al bueno"
 
 py=python
 command -v python >/dev/null 2>&1 || py=python3
diff --git a/.github/scripts/probar-instalador.sh b/.github/scripts/probar-instalador.sh
index 9918c32..ca94473 100644
--- a/.github/scripts/probar-instalador.sh
+++ b/.github/scripts/probar-instalador.sh
@@ -10,8 +10,9 @@
 #   instalacion, comprueba que un hash que no casa no instala nada y que el
 #   PATH queda una sola vez aunque se instale dos veces.
 # - publico: la linea real contra la release publicada. Mide la instalacion y
-#   comprueba que sin TC2_VERSION instala la ultima estable o, si no la hay,
-#   falla diciendo que se use TC2_VERSION.
+#   comprueba que sin TC2_VERSION instala la ultima estable (el binario y su
+#   --version, la del tag de /releases/latest) o, si no la hay, falla
+#   diciendo que se use TC2_VERSION.
 #
 # La medida va desde antes de lanzar la linea hasta que un proceso nuevo, que
 # lee el PATH como una terminal nueva (sh -l en Unix, el registro en Windows),
@@ -42,14 +43,28 @@ esac
 
 # --- Unix: la linea de sh con un HOME nuevo (sin perfil) -------------------
 
-linea_unix() { # <home>
+bajar() { # <url>: a la salida estandar, con curl o, si no lo hay, wget
   if command -v curl >/dev/null 2>&1; then
-    curl -fsSL "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
+    curl -fsSL "$1"
   else
-    wget -qO- "$url/install.sh" | HOME="$1" SHELL=/bin/sh sh
+    wget -qO- "$1"
   fi
 }
 
+# Sin pipefail, la tuberia da el codigo de sh, que sale con 0 si la descarga
+# falla (lee una entrada vacia): el fallo se marca en un fichero, se da en
+# rojo y se devuelve el codigo de sh.
+linea_unix() { # <home>
+  rm -f "$tmp/descarga-fallo"
+  if { bajar "$url/install.sh" || : >"$tmp/descarga-fallo"; } | HOME="$1" SHELL=/bin/sh sh; then
+    codigo_sh=0
+  else
+    codigo_sh=$?
+  fi
+  [ ! -e "$tmp/descarga-fallo" ] || rojo "no se pudo bajar $url/install.sh"
+  return "$codigo_sh"
+}
+
 # Un proceso nuevo, con el entorno vacio, que lee el perfil como un login.
 terminal_nueva_unix() { # <home>
   env -i HOME="$1" PATH=/usr/bin:/bin sh -l -c 'command -v tc2 && tc2 --version'
@@ -265,16 +280,37 @@ desinstalar_maquina() {
   echo "probar: VERDE: tc2 desinstalar --maquina deja la maquina como antes de instalar"
 }
 
+# La version de la ultima release estable, como la saca install.sh
+# (ultima_version): la redireccion de /releases/latest a /releases/tag/v<v>.
+# En modo publico, <url> es .../releases/download/<tag>.
+ultima_estable() {
+  latest="${url%/releases/download/*}/releases/latest"
+  if command -v curl >/dev/null 2>&1; then
+    final=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "$latest") || rojo "no se pudo consultar $latest"
+  else
+    final=$(wget -S --spider "$latest" 2>&1 | sed -n 's/^ *[Ll]ocation: *//p' | tail -n 1)
+  fi
+  final=$(printf '%s' "$final" | tr -d '\r')
+  case "$final" in
+    */releases/tag/v*) printf '%s\n' "${final##*/releases/tag/v}" ;;
+    '') rojo "no se pudo consultar $latest" ;;
+    *) rojo "$latest no lleva a ninguna release estable ('$final')" ;;
+  esac
+}
+
 sin_version() {
   unset TC2_VERSION
   set +e
   if [ "$so" = windows ]; then
     salida=$(linea_windows "$(dir_windows "$tmp/dest-ultima")" 2>&1)
+    codigo=$?
+    instalado="$tmp/dest-ultima/tc2.exe"
   else
     mkdir -p "$tmp/home-ultima"
     salida=$(linea_unix "$tmp/home-ultima" 2>&1)
+    codigo=$?
+    instalado="$tmp/home-ultima/.local/bin/tc2"
   fi
-  codigo=$?
   set -e
   echo "$salida"
   if [ "$codigo" -ne 0 ]; then
@@ -282,8 +318,14 @@ sin_version() {
       *"no hay ninguna release estable"*TC2_VERSION*) ;;
       *) rojo "sin TC2_VERSION fallo sin decir que se use TC2_VERSION" ;;
     esac
+    echo "probar: VERDE: sin TC2_VERSION y sin release estable, error con el arreglo"
+    return 0
   fi
-  echo "probar: VERDE: sin TC2_VERSION, ultima estable o error con el arreglo"
+  [ -e "$instalado" ] || rojo "sin TC2_VERSION el instalador salio con 0, pero no se instalo $instalado"
+  ultima=$(ultima_estable)
+  v=$("$instalado" --version | tr -d '\r')
+  [ "$v" = "tc2 $ultima" ] || rojo "sin TC2_VERSION se instalo '$v', se esperaba 'tc2 $ultima' (la de /releases/latest)"
+  echo "probar: VERDE: sin TC2_VERSION, instala la ultima estable ($ultima)"
 }
 
 case "$modo" in
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index 8d9dfb3..ca082c1 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -281,7 +281,7 @@ jobs:
           base="${VERSION%%-*}"
           vy="$(echo "$base" | awk -F. '{ print $1 "." $2 "." $3 + 1 }')"
           sed -i.orig "s/^version = \"$VERSION\"$/version = \"$vy\"/" Cargo.toml
-          ! cmp -s Cargo.toml Cargo.toml.orig
+          ! cmp -s Cargo.toml Cargo.toml.orig || { echo "::error::el sed no cambio la version de Cargo.toml ($VERSION -> $vy)"; exit 1; }
           rm Cargo.toml.orig
           # Solo cambian las entradas de los miembros del workspace.
           cargo update --workspace
````

## Excluido del diff (8 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-147/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-147/planificacion/plan-final.md                 |   0
 tareas/02-en-curso/TASK-147/seguimiento/arneses/a10/d4-cmp.sh                             |  33 +++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-147/seguimiento/arneses/a10/d4-linea-unix.sh                      |  33 +++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-147/seguimiento/arneses/a10/d4-sin-version.sh                     | 103 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-147/seguimiento/arneses/a11/d4-release-284.sh                     |  33 +++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-147/seguimiento/arneses/a11/release-281-285.txt                   |   5 +++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-147/tarea.md                                    |  44 +++++++++++++++++++++++++++++++++++++++-----
 8 files changed, 246 insertions(+), 5 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

