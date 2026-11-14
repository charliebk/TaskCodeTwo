# Peticion de revision — TASK-001 (ronda 1)

- Tarea: TASK-001 — F1-0 Crear TaskCodeTwo
- Rama revisada: feature/task-001-f1-0-crear-taskcodetwo
- Rama base: develop
- Commit revisado (HEAD): bbc1447168d9d2e87e91cc470087ca097082ba57
- Fecha: 2026-11-14
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-001 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-001/tarea.md
- tareas/03-en-revision/TASK-001/planificacion/
- .gitattributes
- .github/scripts/comprobar-repo.sh
- .github/workflows/ci.yml

## Commits a revisar (git log develop..HEAD)

````
bbc1447 chore(TASK-001): constancia de autorrevision
5443f0b ci(TASK-001): la CI corre en push a cualquier rama; scripts .sh con LF
6661e1d feat(TASK-001): CI minima que valida tareas/ y los enlaces de docs
4650efe chore(TASK-001): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/.gitattributes b/.gitattributes
new file mode 100644
index 0000000..540d1e6
--- /dev/null
+++ b/.gitattributes
@@ -0,0 +1,2 @@
+# Los scripts de shell se ejecutan con bash tambien en Windows: siempre LF.
+*.sh text eol=lf
diff --git a/.github/scripts/comprobar-repo.sh b/.github/scripts/comprobar-repo.sh
new file mode 100755
index 0000000..cc12c5c
--- /dev/null
+++ b/.github/scripts/comprobar-repo.sh
@@ -0,0 +1,167 @@
+#!/usr/bin/env bash
+# CI minima de TaskCodeTwo (TASK-001, F1-0).
+#
+# Comprueba la estructura de tareas/ y los enlaces relativos de la
+# documentacion. Sale con codigo distinto de 0 y una linea por fallo.
+#
+# Uso: .github/scripts/comprobar-repo.sh [raiz-del-repo]
+#
+# Mientras las tareas vivan en las ramas de codigo (TaskCode 0.16), estas
+# comprobaciones tienen sentido aqui. Cuando la 2.0 las lleve a su rama de
+# estado (F1-1), se moveran o se retiraran.
+
+set -u
+shopt -s nullglob
+
+raiz="${1:-.}"
+cd "$raiz" || { echo "ERROR: no existe la raiz $raiz"; exit 2; }
+
+fallos=0
+fallo() {
+  echo "FALLO: $*"
+  fallos=$((fallos + 1))
+}
+
+# --- a) carpetas de estado -------------------------------------------------
+
+declare -A estado_de_carpeta=(
+  [00-planificadas]=planificada
+  [01-en-diseno]=en-diseno
+  [02-en-curso]=en-curso
+  [03-en-revision]=en-revision
+  [04-terminadas]=terminada
+)
+
+for carpeta in "${!estado_de_carpeta[@]}"; do
+  [[ -d "tareas/$carpeta" ]] || fallo "falta la carpeta tareas/$carpeta"
+done
+
+# --- b) frontmatter de cada tarea ------------------------------------------
+
+declare -A tarea_existe=()   # id -> ruta
+declare -A deps_de=()        # id -> dependencias separadas por espacios
+ntareas=0
+
+for carpeta in "${!estado_de_carpeta[@]}"; do
+  for dir in tareas/"$carpeta"/TASK-*/; do
+    dir="${dir%/}"
+    nombre="${dir##*/}"
+    fichero="$dir/tarea.md"
+    if [[ ! -f "$fichero" ]]; then
+      fallo "$dir no tiene tarea.md"
+      continue
+    fi
+    ntareas=$((ntareas + 1))
+
+    id="" estado="" deps="" en_bloque=0 linea_n=0 cerrado=0 en_deps_bloque=0
+    while IFS= read -r linea || [[ -n "$linea" ]]; do
+      linea="${linea%$'\r'}"
+      linea_n=$((linea_n + 1))
+      if (( linea_n == 1 )); then
+        [[ "$linea" == "---" ]] && en_bloque=1 && continue
+        break
+      fi
+      if [[ "$linea" == "---" ]]; then cerrado=1; break; fi
+      # dependencias en lista de bloque YAML ("- TASK-NNN")
+      if (( en_deps_bloque )); then
+        if [[ "$linea" =~ ^[[:space:]]*-[[:space:]]*\"?([^\"[:space:]]+)\"?[[:space:]]*$ ]]; then
+          deps+=" ${BASH_REMATCH[1]}"
+          continue
+        fi
+        en_deps_bloque=0
+      fi
+      if [[ "$linea" =~ ^id:[[:space:]]*\"?([^\"[:space:]]+)\"?[[:space:]]*$ ]]; then
+        id="${BASH_REMATCH[1]}"
+      elif [[ "$linea" =~ ^estado:[[:space:]]*\"?([^\"[:space:]]+)\"?[[:space:]]*$ ]]; then
+        estado="${BASH_REMATCH[1]}"
+      elif [[ "$linea" =~ ^dependencias:[[:space:]]*\[(.*)\][[:space:]]*$ ]]; then
+        lista="${BASH_REMATCH[1]//,/ }"
+        lista="${lista//\"/}"
+        deps+=" $lista"
+      elif [[ "$linea" =~ ^dependencias:[[:space:]]*$ ]]; then
+        en_deps_bloque=1
+      fi
+    done < "$fichero"
+
+    if (( ! en_bloque )); then
+      fallo "$fichero no abre con frontmatter (---)"
+      continue
+    fi
+    (( cerrado )) || fallo "$fichero: el frontmatter no se cierra con ---"
+
+    if [[ -z "$id" ]]; then
+      fallo "$fichero: falta el campo id"
+    elif [[ "$id" != "$nombre" ]]; then
+      fallo "$fichero: id $id no coincide con la carpeta $nombre"
+    fi
+    if [[ -n "$id" ]]; then
+      if [[ -n "${tarea_existe[$id]:-}" ]]; then
+        fallo "id $id repetido: ${tarea_existe[$id]} y $fichero"
+      else
+        tarea_existe[$id]="$fichero"
+      fi
+      deps_de[$id]="$deps"
+    fi
+
+    esperado="${estado_de_carpeta[$carpeta]}"
+    if [[ -z "$estado" ]]; then
+      fallo "$fichero: falta el campo estado"
+    elif [[ "$estado" != "$esperado" ]]; then
+      fallo "$fichero: estado $estado en la carpeta $carpeta (se espera $esperado)"
+    fi
+  done
+done
+
+# --- c) dependencias existentes --------------------------------------------
+
+for id in "${!deps_de[@]}"; do
+  for dep in ${deps_de[$id]}; do
+    [[ -n "${tarea_existe[$dep]:-}" ]] || fallo "$id depende de $dep, que no existe"
+    [[ "$dep" == "$id" ]] && fallo "$id depende de si misma"
+  done
+done
+
+# --- d) enlaces relativos de la documentacion ------------------------------
+
+nenlaces=0
+comprobar_enlaces() {
+  local md="$1" base linea resto destino ruta en_codigo=0
+  base="${md%/*}"
+  [[ "$base" == "$md" ]] && base="."
+  while IFS= read -r linea || [[ -n "$linea" ]]; do
+    linea="${linea%$'\r'}"
+    if [[ "$linea" =~ ^[[:space:]]*(\`\`\`|~~~) ]]; then
+      en_codigo=$((1 - en_codigo))
+      continue
+    fi
+    (( en_codigo )) && continue
+    resto="$linea"
+    while [[ "$resto" =~ \]\(([^\)[:space:]]+)(\ \"[^\"]*\")?\) ]]; do
+      destino="${BASH_REMATCH[1]}"
+      resto="${resto#*"${BASH_REMATCH[0]}"}"
+      case "$destino" in
+        http://*|https://*|mailto:*|\#*) continue ;;
+      esac
+      ruta="${destino%%#*}"
+      ruta="${ruta%%:[0-9]*}"   # file.md:42 (enlace a linea)
+      [[ -z "$ruta" ]] && continue
+      nenlaces=$((nenlaces + 1))
+      if [[ "$ruta" == /* ]]; then
+        [[ -e ".$ruta" ]] || fallo "$md: enlace roto $destino"
+      else
+        [[ -e "$base/$ruta" ]] || fallo "$md: enlace roto $destino"
+      fi
+    done
+  done < "$md"
+}
+
+docs_md=(README.md CLAUDE.md)
+while IFS= read -r -d '' f; do docs_md+=("${f#./}"); done < <(find ./docs -name '*.md' -print0 2>/dev/null)
+for md in "${docs_md[@]}"; do
+  [[ -f "$md" ]] && comprobar_enlaces "$md"
+done
+
+# --- resumen ---------------------------------------------------------------
+
+echo "Tareas: $ntareas; enlaces relativos: $nenlaces; fallos: $fallos"
+(( fallos == 0 ))
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
new file mode 100644
index 0000000..f3650e1
--- /dev/null
+++ b/.github/workflows/ci.yml
@@ -0,0 +1,29 @@
+# CI minima de TaskCodeTwo (TASK-001, F1-0): valida tareas/ y los enlaces de
+# la documentacion. La CI de codigo llega con F2-1 y la de verdad con F5-1.
+name: ci
+
+on:
+  # Cualquier rama: con la 0.16 las tareas se cierran con merge local (sin PR),
+  # asi que la rama de la tarea se verifica en su push.
+  push:
+  pull_request:
+    branches: [develop, main]
+
+permissions:
+  contents: read
+
+concurrency:
+  group: ci-${{ github.ref }}
+  cancel-in-progress: true
+
+jobs:
+  repo:
+    name: comprobar-repo
+    runs-on: ubuntu-latest
+    timeout-minutes: 5
+    steps:
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+      - name: Estructura de tareas/ y enlaces de la documentacion
+        run: bash .github/scripts/comprobar-repo.sh
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-001/planificacion/brainstorm/peticion-unificador-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-001/planificacion/brainstorm/peticion-unificador-2.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-001/planificacion/plan-final.md                       | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-001/tarea.md                                          | 7 ++++++-
 4 files changed, 6 insertions(+), 1 deletion(-)
````
