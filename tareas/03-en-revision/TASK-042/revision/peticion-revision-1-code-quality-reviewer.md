# Peticion de revision — TASK-042 (ronda 1)

- Tarea: TASK-042 — F5-2 Cache por SHA del arbol compartida
- Rama revisada: feature/task-042-f5-2-cache-por-sha-del-arbol-compartida
- Rama base: develop
- Commit revisado (HEAD): aee1621fc055573ce506eeeb31dda3082f1027a2
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-042 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-042/tarea.md
- tareas/03-en-revision/TASK-042/planificacion/
- .github/workflows/verificacion.yml
- docs/ci/verificacion-reutilizable.md
- plantillas/gitlab/verificacion.gitlab-ci.yml

## Commits a revisar (git log develop..HEAD)

````
aee1621 docs(TASK-042): E2E de la cache por arbol, medicion y divergencias
132a9dd fix(TASK-042): buscar con filter=all
88e7a1c fix(TASK-042): ejecucion del check-run por su check suite
c3e803c fix(TASK-042): version del workflow leida de la propia ejecucion
1028bbe feat(TASK-042): cache por SHA del arbol en el workflow de verificacion
6f87439 chore(TASK-042): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 4; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index dbb6ee8..04055b4 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -1,20 +1,27 @@
-# Workflow reutilizable de verificacion (TASK-041, F5-1; ADR F1-7 y F1-8).
+# Workflow reutilizable de verificacion (TASK-041, F5-1; TASK-042, F5-2;
+# ADR F1-7 y F1-8).
 #
 # Solo llama al comando de verificacion del proyecto: no conoce sus pasos ni
 # sus grupos. Tres jobs, cada uno en su clase de F1-8:
-#   - buscar (ejecuta, Linux, contents: read): elige los perfiles del evento
-#     (F1-7, decision 4). La busqueda en la cache por arbol llega con F5-2.
+#   - buscar (Linux, solo lectura: contents, checks y actions): elige los
+#     perfiles del evento (F1-7, decision 4) y busca en los check-runs
+#     `verificacion` del commit verificado V, sus padres y sus abuelos las
+#     lineas `arbol=` candidatas de cada perfil (F1-7, decision 3; F5-2). No
+#     ejecuta nada del PR: solo lee el arbol con git.
 #   - ejecutar (ejecuta, matriz por perfil, runner del perfil, sin secretos):
-#     corre `comando argumentos-<perfil>` en el shell que declara el proyecto
-#     y sube el informe JSON de `TC2_INFORME` (F5-4) como artefacto
-#     `informe-verificacion-<perfil>`, si el comando lo escribio.
-#   - verificacion (privilegio, sin checkout): suma los resultados y publica el
-#     check-run `verificacion` con GITHUB_TOKEN (App github-actions, 15368).
+#     calcula la huella local, y si un candidato tiene la misma se salta el
+#     comando (paso `skipped`); si no, corre `comando argumentos-<perfil>` en
+#     el shell que declara el proyecto y sube el informe JSON de `TC2_INFORME`
+#     (F5-4) como artefacto `informe-verificacion-<perfil>`, si se escribio.
+#   - verificacion (privilegio, sin checkout): lee el arbol de V por la API,
+#     suma los resultados y publica el check-run `verificacion` con
+#     GITHUB_TOKEN (App github-actions, 15368), con una linea `arbol=` por
+#     perfil (F1-8, decision 7).
 #
-# Uso desde el proyecto (fijado por SHA, nunca por rama):
+# Uso desde el proyecto (fijado por SHA, nunca por rama; un solo llamador):
 #   jobs:
 #     verificacion:
-#       permissions: { contents: read, checks: write }
+#       permissions: { contents: read, checks: write, actions: read }
 #       uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>
 #       with:
 #         comando: scripts\verificar.bat
@@ -71,6 +78,14 @@ on:
         description: Perfiles separados por espacios; vacio, los del evento (F1-7, decision 4)
         type: string
         default: ""
+      huella-arbol:
+        description: Rutas versionadas que entran en la clave de la cache (lockfiles, toolchain), separadas por espacios (F1-7, decision 2)
+        type: string
+        default: ""
+      huella-local:
+        description: Orden que imprime lo que la suite lee fuera del arbol en el runner del perfil (F1-7, decision 2); vacia, sha256 de nada
+        type: string
+        default: ""
 
 permissions: {}
 
@@ -81,8 +96,14 @@ jobs:
     timeout-minutes: 5
     permissions:
       contents: read
+      checks: read
+      actions: read
     outputs:
       perfiles: ${{ steps.perfiles.outputs.perfiles }}
+      arbol: ${{ steps.candidatos.outputs.arbol }}
+      claves: ${{ steps.candidatos.outputs.claves }}
+      runners: ${{ steps.candidatos.outputs.runners }}
+      candidatos: ${{ steps.candidatos.outputs.candidatos }}
     steps:
       - name: Perfiles del evento
         id: perfiles
@@ -118,6 +139,106 @@ jobs:
           done
           echo "perfiles=[$lista]" >> "$GITHUB_OUTPUT"
           echo "Perfiles de este evento ($EVENTO): [${lista}]"
+      # V, sus padres y sus abuelos con sus arboles: profundidad 3 basta.
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        if: steps.perfiles.outputs.perfiles != '[]'
+        with:
+          persist-credentials: false
+          fetch-depth: 3
+      # F1-7, decision 3, y F1-8, decision 7. Candidato = linea `arbol=` de un
+      # check-run `verificacion` en verde de github-actions (15368), publicado
+      # por una ejecucion que llama a este workflow en esta misma version, con
+      # el arbol de V y el perfil, el runner y la clave actuales.
+      - name: Candidatos de la cache por arbol
+        id: candidatos
+        if: steps.perfiles.outputs.perfiles != '[]'
+        env:
+          GH_TOKEN: ${{ github.token }}
+          REPO: ${{ github.repository }}
+          PERFILES: ${{ steps.perfiles.outputs.perfiles }}
+          RUN_ID: ${{ github.run_id }}
+          ENTRADAS: ${{ toJSON(inputs) }}
+        run: |
+          set -eu
+          # Version de este workflow: `github.job_workflow_sha` llega vacio en
+          # el reutilizable (E2E de F5-2); se lee de la propia ejecucion.
+          VERSION=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID" --jq '[.referenced_workflows[]? | select(.path | startswith("charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@")) | .sha] | unique | if length == 1 then .[0] else "" end') || VERSION=""
+          v=$(git rev-parse HEAD)
+          arbol=$(git rev-parse 'HEAD^{tree}')
+          rutas=$(printf '%s' "$ENTRADAS" | jq -r '.["huella-arbol"] // ""')
+          huella=""
+          if [ -n "$rutas" ]; then
+            # shellcheck disable=SC2086 # rutas separadas por espacios
+            huella=$(git ls-tree -r HEAD -- $rutas)
+          fi
+          huella=$(printf '%s' "$huella" | sha256sum | cut -d' ' -f1)
+          claves='{}'
+          runners='{}'
+          for p in $(printf '%s' "$PERFILES" | jq -r '.[]'); do
+            r=$(printf '%s' "$ENTRADAS" | jq -r --arg p "$p" '.["runner-" + $p] | fromjson | if type == "array" then join(",") else tostring end' | tr -d ' ')
+            k=$(printf '%s|%s|%s|%s' "$p" "$r" "$huella" "$VERSION" | sha256sum | cut -d' ' -f1)
+            claves=$(printf '%s' "$claves" | jq -c --arg p "$p" --arg k "$k" '. + {($p): $k}')
+            runners=$(printf '%s' "$runners" | jq -c --arg p "$p" --arg r "$r" '. + {($p): $r}')
+          done
+          {
+            echo "arbol=$arbol"
+            echo "claves=$claves"
+            echo "runners=$runners"
+          } >> "$GITHUB_OUTPUT"
+          echo "V=$v arbol=$arbol version=$VERSION claves=$claves runners=$runners"
+          if [ -z "$VERSION" ]; then
+            echo "::warning::version del workflow desconocida: sin busqueda, se verifica entero"
+            echo "candidatos={}" >> "$GITHUB_OUTPUT"
+            exit 0
+          fi
+          # F1-8, decision 7: un PR que toca los workflows o .taskcode no
+          # reutiliza nada, tampoco la linea de su head anterior.
+          if git rev-parse -q --verify 'HEAD^1' > /dev/null; then
+            tocadas=$(git diff --name-only 'HEAD^1' HEAD | grep -E '^(\.github/workflows/|\.taskcode/)' || true)
+            if [ -n "$tocadas" ]; then
+              echo "Regla de rutas: el diff frente a la base toca $(printf '%s' "$tocadas" | tr '\n' ' '); sin acierto"
+              echo "candidatos={}" >> "$GITHUB_OUTPUT"
+              exit 0
+            fi
+          fi
+          # V, sus padres y los padres de estos, en ese orden (como mucho 7).
+          padres=$(git rev-list --parents -n 1 "$v" | cut -d' ' -f2- -s)
+          commits="$v $padres"
+          for p in $padres; do
+            commits="$commits $(git rev-list --parents -n 1 "$p" | cut -d' ' -f2- -s)"
+          done
+          commits=$(printf '%s\n' $commits | awk 'NF && !visto[$0]++' | head -n 7)
+          filtro_run="[.referenced_workflows[]? | select((.path | startswith(\"charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@\")) and .sha == \"$VERSION\")] | length"
+          lineas="$RUNNER_TEMP/lineas-cache.txt"
+          : > "$lineas"
+          for c in $commits; do
+            # filter=all: "Re-run all jobs" saca de `latest` (y de la suite)
+            # el check-run del intento anterior (E2E de F5-2). El mas reciente
+            # primero.
+            checks=$(gh api --method GET "repos/$REPO/commits/$c/check-runs" -f check_name=verificacion -f app_id=15368 -f filter=all --jq '.check_runs | sort_by(.completed_at) | reverse | .[] | select(.app.id == 15368 and .name == "verificacion" and .conclusion == "success") | @base64') \
+              || { echo "commit $c: sin lectura de check-runs; no cuenta"; continue; }
+            for ch in $checks; do
+              # GitHub ignora el details_url de un check-run de GITHUB_TOKEN
+              # (E2E de F5-2): la ejecucion sale de su check suite.
+              suite=$(printf '%s' "$ch" | base64 -d | jq -r '.check_suite.id // ""')
+              case "$suite" in ''|*[!0-9]*) echo "commit $c: check-run sin check suite; no cuenta"; continue ;; esac
+              llama=$(gh api --method GET "repos/$REPO/actions/runs" -f check_suite_id="$suite" --jq "[.workflow_runs[] | $filtro_run] | add // 0") || llama=0
+              if [ "$llama" = 0 ]; then
+                echo "commit $c: la ejecucion de la suite $suite no llama a este workflow en $VERSION; no cuenta"
+                continue
+              fi
+              printf '%s' "$ch" | base64 -d | jq -r '.output.summary // ""' | tr -d '\r' \
+                | awk -v a="$arbol" -v de="$c" '/^arbol=/ { split("", f); for (i = 1; i <= NF; i++) { n = index($i, "="); if (n > 1) f[substr($i, 1, n - 1)] = substr($i, n + 1) } if (f["arbol"] == a) print f["perfil"], f["runner"], f["clave"], f["local"], f["origen"], de }' >> "$lineas"
+            done
+          done
+          candidatos=$(jq -R -s -c --argjson claves "$claves" --argjson runners "$runners" '
+            [split("\n")[] | select(length > 0) | split(" ")
+             | {perfil: .[0], runner: .[1], clave: .[2], local: .[3], origen: .[4], de: .[5]}
+             | select($claves[.perfil] == .clave and $runners[.perfil] == .runner)]
+            | reduce .[] as $x ({}; .[$x.perfil] += [{local: $x.local, origen: $x.origen, de: $x.de}])' "$lineas")
+          echo "candidatos=$candidatos" >> "$GITHUB_OUTPUT"
+          echo "Commits consultados: $(printf '%s' "$commits" | tr '\n' ' ')"
+          echo "Candidatos: $candidatos"
 
   ejecutar:
     name: ${{ matrix.perfil }}
@@ -137,6 +258,21 @@ jobs:
     concurrency:
       group: verificacion-${{ github.repository }}-${{ github.event.pull_request.number || github.ref }}-${{ matrix.perfil }}
       cancel-in-progress: false
+    # Una salida con nombre propio por perfil: cada pata de la matriz escribe
+    # solo las suyas (F5-2).
+    outputs:
+      local-rapida: ${{ steps.cache.outputs.local-rapida }}
+      local-impacto: ${{ steps.cache.outputs.local-impacto }}
+      local-completa: ${{ steps.cache.outputs.local-completa }}
+      local-pesada: ${{ steps.cache.outputs.local-pesada }}
+      de-rapida: ${{ steps.cache.outputs.de-rapida }}
+      de-impacto: ${{ steps.cache.outputs.de-impacto }}
+      de-completa: ${{ steps.cache.outputs.de-completa }}
+      de-pesada: ${{ steps.cache.outputs.de-pesada }}
+      origen-rapida: ${{ steps.cache.outputs.origen-rapida }}
+      origen-impacto: ${{ steps.cache.outputs.origen-impacto }}
+      origen-completa: ${{ steps.cache.outputs.origen-completa }}
+      origen-pesada: ${{ steps.cache.outputs.origen-pesada }}
     defaults:
       run:
         shell: ${{ inputs.shell }}
@@ -147,9 +283,39 @@ jobs:
       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
         with:
           persist-credentials: false
+      # F1-7, decision 2: la huella local solo se calcula en el runner del
+      # perfil, con el shell del proyecto.
+      - name: huella local, perfil ${{ matrix.perfil }}
+        if: inputs.huella-local != ''
+        run: ${{ inputs.huella-local }} > "${{ runner.temp }}/huella-local.txt"
+      - name: cache por arbol, perfil ${{ matrix.perfil }}
+        id: cache
+        shell: bash
+        env:
+          CANDIDATOS: ${{ needs.buscar.outputs.candidatos }}
+          HUELLA: ${{ runner.temp }}/huella-local.txt
+        run: |
+          set -eu
+          touch "$HUELLA"
+          huella=$(sha256sum "$HUELLA" | cut -d' ' -f1)
+          acierto=$(printf '%s' "$CANDIDATOS" | jq -r --arg p "$TC2_PERFIL" --arg l "$huella" '[.[$p][]? | select(.local == $l)][0] // empty | "\(.de) \(.origen)"')
+          de=${acierto%% *}
+          origen=${acierto#* }
+          {
+            echo "local-$TC2_PERFIL=$huella"
+            echo "de-$TC2_PERFIL=$de"
+            echo "origen-$TC2_PERFIL=$origen"
+            if [ -n "$de" ]; then echo "acierto=si"; else echo "acierto=no"; fi
+          } >> "$GITHUB_OUTPUT"
+          if [ -n "$de" ]; then
+            echo "Perfil $TC2_PERFIL: reutilizada de $de (origen $origen, local $huella)"
+          else
+            echo "Perfil $TC2_PERFIL: sin acierto (local $huella); se ejecuta el comando"
+          fi
       # TC2_INFORME (F5-4): el comando escribe ahi su informe JSON, si sabe;
       # la plataforma solo lo sube. `runner` no existe en el `env` del job.
       - name: comando del proyecto, perfil ${{ matrix.perfil }}
+        if: steps.cache.outputs.acierto != 'si'
         env:
           TC2_INFORME: ${{ runner.temp }}/informe-verificacion.json
         run: ${{ inputs.comando }} ${{ inputs[format('argumentos-{0}', matrix.perfil)] }}
@@ -169,14 +335,20 @@ jobs:
     timeout-minutes: 5
     permissions:
       checks: write
+      contents: read
     steps:
       - name: Publicar el check verificacion
         env:
           GH_TOKEN: ${{ github.token }}
           REPO: ${{ github.repository }}
+          V: ${{ github.sha }}
           HEAD_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
           RESULTADO: ${{ needs.ejecutar.result }}
           PERFILES: ${{ needs.buscar.outputs.perfiles }}
+          ARBOL_BUSCAR: ${{ needs.buscar.outputs.arbol }}
+          CLAVES: ${{ needs.buscar.outputs.claves }}
+          RUNNERS: ${{ needs.buscar.outputs.runners }}
+          SALIDAS: ${{ toJSON(needs.ejecutar.outputs) }}
           EJECUCION: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
         run: |
           set -eu
@@ -188,6 +360,29 @@ jobs:
             informes="${informes:+$informes,}informe-verificacion-$p"
           done
           resumen="perfiles=$PERFILES resultado=$RESULTADO informe=$informes ejecucion=$EJECUCION"
+          # F1-8, decision 7: el arbol lo lee el publicador por la API; si no
+          # es el de buscar, no se escribe ninguna linea y el check sale rojo.
+          arbol=$(gh api --method GET "repos/$REPO/commits/$V" --jq '.commit.tree.sha')
+          if [ "$arbol" != "$ARBOL_BUSCAR" ]; then
+            echo "verificacion: arbol de $V por la API ($arbol) distinto del de buscar ($ARBOL_BUSCAR)" >&2
+            conclusion=failure
+          elif [ "$conclusion" = success ]; then
+            for p in $(printf '%s' "$PERFILES" | tr -d '[]"' | tr ',' ' '); do
+              campo() { printf '%s' "$SALIDAS" | jq -r --arg k "$1-$p" '.[$k] // ""'; }
+              huella=$(campo local)
+              de=$(campo de)
+              origen=$(campo origen)
+              if [ -z "$huella" ]; then
+                echo "verificacion: el perfil $p no dejo su huella local; sin linea" >&2
+                continue
+              fi
+              if [ -n "$de" ]; then estado="reutilizada de $de"; else estado=ejecutada; origen=$HEAD_SHA; fi
+              r=$(printf '%s' "$RUNNERS" | jq -r --arg p "$p" '.[$p]')
+              k=$(printf '%s' "$CLAVES" | jq -r --arg p "$p" '.[$p]')
+              resumen="$resumen
+          arbol=$arbol perfil=$p runner=$r clave=$k local=$huella origen=$origen $estado"
+            done
+          fi
           gh api --method POST "repos/$REPO/check-runs" \
             -f name=verificacion -f head_sha="$HEAD_SHA" -f status=completed \
             -f conclusion="$conclusion" -f details_url="$EJECUCION" \
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index 27b181e..bf0bd41 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -9,10 +9,9 @@ decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.
 | GitHub Actions | [`.github/workflows/verificacion.yml`](../../.github/workflows/verificacion.yml) (`on: workflow_call`) | un job del proyecto con `uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>` |
 | GitLab CI | [`plantillas/gitlab/verificacion.gitlab-ci.yml`](../../plantillas/gitlab/verificacion.gitlab-ci.yml) (`spec:inputs`) | `include: remote` a la URL raw de la plantilla fijada por SHA |
 
-Fuera de F5-1: la caché por árbol y la línea `arbol=` (F5-2), el runner
-propio (F5-5), `carril` y `revision-ia` (F6), y la definición de CI en otro
-proyecto de GitLab (F2-9). El informe JSON para el revisor (F5-4) se describe
-más abajo.
+Fuera de F5-1: el runner propio (F5-5), `carril` y `revision-ia` (F6), y la
+definición de CI en otro proyecto de GitLab (F2-9). El informe JSON para el
+revisor (F5-4) y la caché por árbol (F5-2) se describen más abajo.
 
 ## Entradas
 
@@ -29,6 +28,8 @@ se elige el runner.
 | `argumentos-completa` | `--perfil completa` | Ídem; vacío (`""`) llama al comando sin argumentos |
 | `argumentos-pesada` | `--perfil pesada` | Ídem |
 | `perfiles` | `""` (los del evento) | Perfiles forzados, separados por espacios |
+| `huella-arbol` | `""` | Rutas versionadas que entran en la clave de la caché (lockfiles, toolchain), separadas por espacios (F5-2) |
+| `huella-local` | `""` | Orden que imprime lo que la suite lee fuera del árbol en el runner del perfil (F5-2); vacía, `local = sha256("")` |
 | GitHub: `runner-<perfil>` | `'"ubuntu-latest"'` | `runs-on` del perfil, en JSON: una cadena o una lista de etiquetas (`'["self-hosted","windows","toolsaweb"]'`) |
 | GitHub: `shell` | `bash` | Shell en el que corre el comando (`cmd` para un `.bat`) |
 | GitLab: `tags-<perfil>` | `[]` | Etiquetas del runner del perfil |
@@ -67,8 +68,9 @@ Resultado: código de salida (0 verde, otro rojo).
 - **GitLab**: un job `verificacion-<perfil>` por perfil del evento; el estado
   es el del pipeline.
 
-El job que llama tiene que conceder `checks: write` (un workflow reutilizable
-no puede tener más permisos que su llamador). En un PR desde un fork el
+El job que llama tiene que conceder `contents: read`, `checks: write` y
+`actions: read` (un workflow reutilizable no puede tener más permisos que su
+llamador; `actions: read` lo pide `buscar` desde F5-2). En un PR desde un fork el
 `GITHUB_TOKEN` es de solo lectura: el publicador falla y el PR queda en rojo
 (falla cerrado).
 
@@ -90,6 +92,7 @@ jobs:
     permissions:
       contents: read
       checks: write
+      actions: read
     uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>
     with:
       comando: scripts\verificar.bat
@@ -117,6 +120,7 @@ jobs:
     permissions:
       contents: read
       checks: write
+      actions: read
     uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>
     with:
       comando: scripts\verificar.bat
@@ -148,9 +152,10 @@ quiere evitar el pipeline de rama duplicado con el de MR, lo pone en su
 `cargo test` (y por tanto en `cargo xtask verificar --perfil rapida`). Sobre el
 workflow: eventos admitidos (`workflow_call`, `pull_request`, `push`,
 `merge_group`, `workflow_dispatch`), `permissions` arriba y en cada job (sin
-`write-all`), las únicas acciones admitidas son `actions/checkout` fijada por
-SHA y con `persist-credentials: false` (sin `ref:` ni `repository:` en un job
-que escribe) y `actions/upload-artifact` fijada por SHA y solo en un job sin
+`write-all`; de lectura solo `contents`, `checks` y `actions`, F5-2), las
+únicas acciones admitidas son `actions/checkout` fijada por
+SHA, con `persist-credentials: false` y nunca en un job que escribe (F5-2:
+antes solo se prohibían `ref:` y `repository:`) y `actions/upload-artifact` fijada por SHA y solo en un job sin
 escritura (F5-4), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
 ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`
 (también con corchetes, `github['event']`, `secrets['X']`, o con espacios);
@@ -190,6 +195,71 @@ La plataforma solo lo transporta (F1-7, decisión 1: no conoce los pasos):
   hay artefacto y el revisor solo tiene el resumen del check, que basta para no
   relanzar un perfil en verde. El publicador no abre el artefacto.
 
+## Caché por árbol (F5-2)
+
+TASK-042. Cero repeticiones sobre el mismo árbol (F1-7, decisiones 2 y 3;
+F1-8, decisión 7), sin componentes nuevos: los tres jobs de siempre.
+
+- **`buscar`** (Linux; `contents`, `checks` y `actions` de lectura; no
+  ejecuta nada del PR): checkout del commit verificado `V` (el merge ref en un
+  PR) con `fetch-depth: 3`; `V`, sus padres y sus abuelos (como mucho 7) salen
+  de `git`. Para cada uno pide
+  `GET commits/{c}/check-runs?check_name=verificacion&app_id=15368&filter=all`
+  (el más reciente primero) y solo cuenta un check-run en verde cuya
+  ejecución (su check suite →
+  `GET actions/runs?check_suite_id={id}`) tenga en `referenced_workflows` este
+  workflow (`charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@…`) con
+  el mismo `sha` que la ejecución actual (la versión del workflow, leída con
+  `GET actions/runs/{run_id}`). De su resumen toma las líneas `arbol=`
+  con el árbol de `V` y el perfil, el runner y la clave actuales: son los
+  candidatos de cada perfil, en el orden de los commits. Si el diff de `V`
+  frente a su primer padre (la base) toca `.github/workflows/**` o
+  `.taskcode/**`, no hay candidatos.
+- **`ejecutar`** (runner del perfil): calcula la huella local (la orden de
+  `huella-local` en el shell del proyecto, y `sha256sum` en `bash`); el primer
+  candidato con el mismo `local` es el **acierto** y el paso del comando sale
+  `skipped`. Corre siempre en el runner del perfil: un runner propio caído deja
+  el check pendiente aunque haya candidato (falla cerrado).
+- **`verificacion`** (publicador): lee el árbol de `V` con `GET commits/{V}`;
+  si no es el de `buscar`, el check sale rojo y sin líneas. Con el resultado en
+  verde añade al resumen una línea por perfil.
+
+**La línea** (contrato para F8-1), una por perfil, debajo de la de siempre
+(`perfiles= resultado= informe= ejecucion=`):
+
+```
+arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> ejecutada
+arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> reutilizada de <commit>
+```
+
+- `runner`: el `runner-<perfil>` declarado, sin comillas ni espacios (una
+  lista de etiquetas, unida por comas).
+- `clave = sha256(perfil|runner|huella-arbol|versión del workflow)` (el `sha`
+  de `referenced_workflows` de la ejecución), con
+  `huella-arbol = sha256(git ls-tree -r HEAD -- <rutas>)` (o de nada sin rutas).
+- `local = sha256` de lo que imprime `huella-local` en el runner del perfil.
+- `origen`: el commit (head del PR) donde se ejecutó de verdad; una línea
+  reutilizada conserva el del candidato. `reutilizada de <commit>`: el commit
+  en cuyo check estaba la línea que dio el acierto.
+
+**Cómo leer un acierto**: en el job `<llamador> / <perfil>`, el paso «cache
+por arbol» dice `reutilizada de <commit>` y el paso del comando sale
+`skipped`; en el check `verificacion`, la línea del perfil termina en
+`reutilizada de <commit>`. Solo cuenta un check en verde: una línea de un
+check rojo no es candidata.
+
+**Coste**: hasta 7 lecturas de check-runs más una de la ejecución por cada
+check en verde encontrado, y una de `commits/{V}` en el publicador. Un acierto
+despierta igualmente el runner del perfil (huella local).
+
+**GitLab** (D2): sin búsqueda; cada pipeline verifica entero (degradación
+declarada, RR-15). Con el comando en verde, `after_script` escribe la línea
+(`origen=$CI_COMMIT_SHA ejecutada`, `runner` = `CI_RUNNER_TAGS`, versión vacía en
+la clave) en `.tc2/cache-verificacion-<perfil>.txt`, dentro del artefacto
+`informe-verificacion-<perfil>` (un job de GitLab tiene un solo archivo de
+artefactos). `after_script` no cambia el estado del job: en un runner sin `sh`
+no hay línea.
+
 ## Divergencias con los ADR
 
 1. **Argumentos por perfil** (D1): F1-7, decisión 1, fija `--perfil
@@ -209,8 +279,9 @@ La plataforma solo lo transporta (F1-7, decisión 1: no conoce los pasos):
    mismo repo llaman al reutilizable en el mismo commit, salen dos check-runs
    `verificacion` (uno por llamador) y GitHub puede listar el creado con
    `GITHUB_TOKEN` en la suite de otra ejecución (el rojo de un llamador
-   apareció en la lista de jobs del otro). Con `filter=latest` solo cuenta el
-   último: el proyecto declara un único llamador.
+   apareció en la lista de jobs del otro). El proyecto declara un único
+   llamador; desde F5-2 además `buscar` enlaza cada check-run con su ejecución
+   por la check suite, y con dos llamadores ese enlace dejaría de ser fiable.
 6. **Concurrencia por PR (o ref) y perfil**: el grupo es
    `verificacion-<repo>-<github.event.pull_request.number || github.ref>-<perfil>`
    (revisión de la ronda 1: por repo y perfil, un tercer PR cancelaba la
@@ -238,6 +309,43 @@ La plataforma solo lo transporta (F1-7, decisión 1: no conoce los pasos):
 10. **ToolsaWeb y OpenDockTrading** no producen el informe hasta migrar (F9):
     su `.estado/verificar-ultima.json` tiene otra forma. Hasta entonces solo
     existe el resumen del check.
+11. **Versión del workflow en la clave** (F1-7, decisión 2; plan de F5-2):
+    `github.job_workflow_sha` llega vacío dentro del reutilizable (E2E). La
+    versión es el `sha` de `referenced_workflows` de la propia ejecución
+    (`GET actions/runs/{run_id}`, `actions: read`); si no hay exactamente uno,
+    no se busca y se verifica entero.
+12. **`filter=all` en vez de `filter=latest`** (F1-7, decisión 3): «Re-run all
+    jobs» saca de `latest` (y de la lista de la suite) el check-run del intento
+    anterior, así que con `latest` una re-ejecución no encontraba nada (E2E,
+    run 44607389843 intento 2). Se leen todos los del commit, el más reciente
+    primero. Una línea en verde de un intento anterior sigue siendo válida: el
+    resultado es función del árbol, la clave y la huella local.
+13. **Origen del check-run por su check suite, no por `details_url`** (F1-8,
+    decisión 7; D3 del plan): GitHub ignora el `details_url` de un check-run
+    creado con `GITHUB_TOKEN` (queda en `…/runs/<id del check-run>`). `buscar`
+    enlaza el check-run con su ejecución por `check_suite.id`
+    (`GET actions/runs?check_suite_id=`) y exige ahí este workflow en la misma
+    versión.
+14. **Huella local volátil en los runners de GitHub**: con `huella-local: uname
+    -sr` dos patas de la misma ejecución, y la misma pata entre intentos,
+    sacaron `local` distintos (imágenes con distinto kernel; runs 44607057098
+    y 44607389843): sin acierto. La huella local debe nombrar solo lo que de
+    verdad cambia el resultado (versiones de herramientas fijadas), no el
+    sistema del runner. Se documenta para F1-11.
+15. **PR del E2E a una rama base con el llamador, no a `main`**: la regla de
+    rutas rechaza toda línea si el PR toca `.github/workflows/**`, y el
+    llamador del E2E vive ahí; contra `main` (sin llamador) nunca habría
+    acierto. La base `cache-f5-2-base` lleva el mismo llamador que el PR.
+16. **Lint más estricto** (F1-8, decisión 1): un job que escribe ya no puede
+    hacer checkout (antes solo se prohibían `ref:` y `repository:`), y solo se
+    admiten lecturas de `contents`, `checks` y `actions`. El lector YAML
+    admite además `- |` (bloque literal como elemento de lista: el
+    `after_script` de GitLab).
+17. **GitLab: línea en un fichero del artefacto del informe** (D2): un job de
+    GitLab tiene un solo archivo de artefactos; la línea va en
+    `.tc2/cache-verificacion-<perfil>.txt` dentro de
+    `informe-verificacion-<perfil>`, sin la versión del workflow en la clave.
+    Sin pipeline real (D2): solo el lint de la plantilla.
 
 ## Prueba E2E (2026-11-19, workflow y plantilla en `790b72d`)
 
@@ -280,3 +388,46 @@ Suposiciones del plan:
   (requisito de sus runners compartidos).
 - **`charliebk/TaskCodeTwo` público**: sí; el reutilizable se resolvió desde
   otro repo por SHA de una rama que no es `develop` ni `main`.
+
+## Prueba E2E de la caché por árbol (2026-11-19, workflow en `132a9dd`)
+
+TASK-042, en `charliebk/taskcodetwo-e2e-github`: llamador único con un stub
+Linux (`sh scripts/verificar.sh`), `perfiles: completa pesada` (dos patas de
+la matriz) y `huella-local: echo e2e-f5-2`. PR #13 de `cache-f5-2` a
+`cache-f5-2-base` (rama base con el mismo llamador: ver divergencia 15),
+cerrado sin mergear y ramas borradas al terminar.
+
+| Caso | Ejecución | Paso del comando (completa / pesada) | Línea del perfil |
+|---|---|---|---|
+| (a) PR nuevo | [44607911390, intento 1](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44607911390/attempts/1) | `success` / `success` | `ejecutada` |
+| (a) «Re-run all jobs» (`gh run rerun`) | [44607911390, intento 2](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44607911390/attempts/2) | `skipped` / `skipped` | `reutilizada de 10d5e76` (el head) |
+| (b) commit vacío | [44608246992](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608246992) | `skipped` / `skipped` | `reutilizada de 10d5e76` (el head anterior, abuelo de V) |
+| (c) cambiar `datos.txt` | [44608436324](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608436324) | `success` / `success` | `ejecutada` |
+| (c) tocar `.github/workflows/verificacion.yml` | [44608660673](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608660673) | `success` / `success` | `ejecutada` («Regla de rutas») |
+| (c) commit vacío tras tocar el workflow | [44608801152](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44608801152) | `success` / `success` | `ejecutada`: mismo árbol, pero la regla de rutas rechaza toda línea |
+
+Medición (criterio 1) con
+`medicion/medir.sh` de la carpeta de TASK-042 en `tareas/` (solo `gh api --method GET`;
+salida en `salida.txt` junto a él): por árbol y perfil, el paso del comando no
+salió `skipped` **una vez** en los árboles `5b30777` y `91a0f2f`; dos veces en
+`555e6b4`, el árbol cuyo PR toca `.github/workflows/**`, donde la regla de
+rutas obliga a verificar entero.
+
+Suposiciones del plan:
+
+- **`github.job_workflow_sha` en el reutilizable**: **falsa**, llega vacío
+  (run 44606759639, `version=` vacío). La versión se lee de
+  `referenced_workflows` de la propia ejecución (divergencia 11).
+- **Salidas con nombre fijo por perfil en la matriz**: confirmado; con dos
+  patas, el publicador recibió `local-completa` y `local-pesada` sin pisarse.
+- **Merge ref con `fetch-depth: 3`**: confirmado; da V, sus dos padres (la
+  base primero) y los abuelos, y el diff frente a `HEAD^1` es el del PR.
+- **Una re-ejecución deja el check anterior como `latest` en el head**:
+  **falsa**: «Re-run all jobs» saca el check-run del intento anterior de la
+  suite y de `filter=latest` (sigue existiendo por id y con `filter=all`)
+  (divergencia 12).
+- **`bash` en el runner del perfil**: solo se probó Linux; en Windows el paso
+  «cache por arbol» usa el `bash` de Git for Windows (no verificado aquí).
+- **`GET commits/{V}` sobre el merge ref desde el publicador**: funciona con
+  `contents: read`; el árbol coincidió con el de `buscar` en todas las
+  ejecuciones.
diff --git a/plantillas/gitlab/verificacion.gitlab-ci.yml b/plantillas/gitlab/verificacion.gitlab-ci.yml
index e553b53..af0b200 100644
--- a/plantillas/gitlab/verificacion.gitlab-ci.yml
+++ b/plantillas/gitlab/verificacion.gitlab-ci.yml
@@ -12,9 +12,13 @@
 #       inputs:
 #         comando: scripts/verificar.sh
 # Fuera de F5-1 (D4): carril, estado de revision-ia y ci_config_path en otro
-# proyecto. La linea de la cache por arbol ira en un artefacto del job (D3, F5-2).
+# proyecto.
 # Informe JSON (F5-4): el comando lo escribe en TC2_INFORME y cada job lo sube
 # como artefacto informe-verificacion-<perfil> (when: always).
+# Cache por arbol (F5-2, D2): solo se escribe la linea `arbol=` del perfil en
+# verde, en el fichero .tc2/cache-verificacion-<perfil>.txt del mismo
+# artefacto (un job tiene un solo archivo de artefactos). Sin busqueda: en
+# GitLab cada pipeline verifica entero (degradacion declarada, RR-15).
 # Guia: docs/ci/verificacion-reutilizable.md.
 spec:
   inputs:
@@ -51,6 +55,12 @@ spec:
     perfiles:
       description: Perfiles forzados separados por espacios (tambien en push de rama)
       default: ""
+    huella-arbol:
+      description: Rutas versionadas que entran en la clave de la cache, separadas por espacios
+      default: ""
+    huella-local:
+      description: Orden que imprime lo que la suite lee fuera del arbol en el runner del perfil
+      default: ""
     stage:
       description: Etapa de los jobs de verificacion
       default: test
@@ -67,11 +77,32 @@ variables:
     RAMA_BASE: $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
     # F5-4: el comando escribe ahi su informe JSON, si sabe; el job lo sube.
     TC2_INFORME: $CI_PROJECT_DIR/.tc2/informe-verificacion.json
+    TC2_HUELLA_ARBOL: "$[[ inputs.huella-arbol ]]"
+    TC2_HUELLA_LOCAL: "$[[ inputs.huella-local ]]"
+  # F5-2: la linea `arbol=` solo si el comando salio en verde. Va en
+  # after_script (sh), que no cambia el estado del job: en un runner sin sh
+  # no hay linea, pero el resultado del perfil es el mismo. `runner` son las
+  # etiquetas del runner; la version del workflow va vacia (GitLab no la da).
+  after_script:
+    - |
+      if [ "$CI_JOB_STATUS" = success ]; then
+        mkdir -p .tc2
+        arbol=$(git rev-parse 'HEAD^{tree}')
+        huella=""
+        if [ -n "$TC2_HUELLA_ARBOL" ]; then huella=$(git ls-tree -r HEAD -- $TC2_HUELLA_ARBOL); fi
+        huella=$(printf '%s' "$huella" | sha256sum | cut -d' ' -f1)
+        runner=$(printf '%s' "$CI_RUNNER_TAGS" | tr -d '[]" ')
+        clave=$(printf '%s|%s|%s|%s' "$TC2_PERFIL" "$runner" "$huella" "" | sha256sum | cut -d' ' -f1)
+        if [ -n "$TC2_HUELLA_LOCAL" ]; then sh -c "$TC2_HUELLA_LOCAL" > .tc2/huella-local.txt; else : > .tc2/huella-local.txt; fi
+        local_h=$(sha256sum .tc2/huella-local.txt | cut -d' ' -f1)
+        echo "arbol=$arbol perfil=$TC2_PERFIL runner=$runner clave=$clave local=$local_h origen=$CI_COMMIT_SHA ejecutada" | tee ".tc2/cache-verificacion-$TC2_PERFIL.txt"
+      fi
   artifacts:
     name: informe-verificacion-$TC2_PERFIL
     when: always
     paths:
       - .tc2/informe-verificacion.json
+      - .tc2/cache-verificacion-*.txt
 
 verificacion-rapida:
   extends: .tc2-verificacion
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-042/medicion/medir.sh                                             | 49 +++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-042/medicion/salida.txt                                           | 21 +++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-042/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-042/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-042/tarea.md                                    |  3 ++-
 5 files changed, 72 insertions(+), 1 deletion(-)
````
