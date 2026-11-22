# Peticion de revision — TASK-052 (ronda 1)

- Tarea: TASK-052 — F6-6 Rondas incrementales
- Rama revisada: feature/task-052-f6-6-rondas-incrementales
- Rama base: develop
- Commit revisado (HEAD): 8822e8ee1e21657de2fd037cc0d334b27fe58006
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-052 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-052/tarea.md
- tareas/03-en-revision/TASK-052/planificacion/
- .github/workflows/revision-ia.yml
- crates/tc2/esquemas/hallazgos.schema.json
- crates/tc2/tests/fixtures/hallazgos/respuesta-menor-nota.md
- crates/tc2/tests/fixtures/hallazgos/respuesta-ronda2.md
- docs/adr/F1-10-carriles-y-modo-automatico.md
- docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
- docs/adr/F1-16-lotes-desatendidos.md
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/revision-ia.md

## Commits a revisar (git log develop..HEAD)

````
8822e8e feat(TASK-052): review guarda el informe del head anterior y adendas F6-6
2015305 feat(TASK-052): workflow revision-ia con ronda anterior e incremental
21709fa feat(TASK-052): tc2 revisoras --ronda para aplicar S13
5d8ad46 feat(TASK-052): ronda siguiente desde el check anterior y ordenes de rondas
b70d3b3 feat(TASK-052): contrato v3 de hallazgos con rondas y cierres
8084d7c chore(TASK-052): tarea en curso
````

## Diff de tu dominio (9 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index 273a05c..6e72898 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -1,14 +1,15 @@
 # Workflow reutilizable del revisor IA en CI (TASK-049, F6-3; TASK-050,
-# F6-4; TASK-051, F6-5; ADR F1-15 decisiones 1, 2 y 6, F1-8, F1-6 D5a y
-# adendas F6-3 y F6-5 de F1-15 y F6-3 y F6-4 de F1-8).
+# F6-4; TASK-051, F6-5; TASK-052, F6-6; ADR F1-15 decisiones 1, 2 y 6, F1-8,
+# F1-6 D5a y adendas F6-3, F6-5 y F6-6 de F1-15 y F6-3, F6-4 y F6-6 de F1-8).
 #
 # El enrutado no lo decide el modelo: `tc2 revisoras` elige las revisoras,
 # las reglas y el modelo con las tablas de F1-15 y el catalogo de la base.
 # Tampoco decide que hallazgos cuentan: `tc2 hallazgos validar` rechaza el
 # que no trae evidencia de su tipo anclada a una linea del diff (F6-4). Ni
-# el veredicto: `tc2 hallazgos veredicto` saca la conclusion (F6-5).
+# el veredicto: `tc2 hallazgos veredicto` saca la conclusion (F6-5). Ni la
+# ronda: `tc2 hallazgos ronda` la decide con el check anterior (F6-6).
 # Cuatro jobs, cada uno en su clase de F1-8:
-#   - preparar (lee, sin secretos ni escritura: contents y actions): saca la
+#   - preparar (lee, sin secretos ni escritura: contents, actions y checks): saca la
 #     base (nunca el head del PR), el plugin taskcode a un SHA fijado y el
 #     codigo de tc2 a la version de este workflow; trae el head como dato
 #     (`git fetch` de su SHA, sin checkout) para el diff; baja el informe de
@@ -19,7 +20,13 @@
 #     del informe, el diff de sus rutas, con tope, y las instrucciones de
 #     `tc2 hallazgos instrucciones`) en el artefacto `revision-ia-prompts`,
 #     con el diff recortado que vio cada revisora (`diffs/<i>.txt`) y el
-#     informe. No ejecuta nada del PR: ni la suite ni su codigo.
+#     informe. No ejecuta nada del PR: ni la suite ni su codigo. F6-6: antes
+#     busca la ronda anterior (el ultimo check `revision-ia` de la App del
+#     revisor en un commit anterior del PR, con `checks: read`) y `tc2
+#     hallazgos ronda` deja `ronda/ronda.json`: en modo incremental cada
+#     revisora ve el diff desde ese commit y sus abiertos
+#     (`abiertos/abiertos-<i>.json`); en `ligera` y `tope` no hay prompts y
+#     `consultar` no llama al modelo.
 #   - consultar (privilegio por el entorno `revisor`, sin checkout, solo
 #     ANTHROPIC_API_KEY): llama a la API de Anthropic (Messages, curl) una
 #     vez por revisora con el modelo elegido y pasa cada respuesta, cortada
@@ -53,7 +60,7 @@
 #   jobs:
 #     revision-ia:
 #       if: github.event.workflow_run.event == 'pull_request' && github.event.workflow_run.pull_requests[0] != null
-#       permissions: { contents: read, actions: read }
+#       permissions: { contents: read, actions: read, checks: read }
 #       uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
 #       # Unica excepcion a R9 de F1-8: sin heredar, los secretos del entorno
 #       # revisor no llegan a `consultar` ni a `publicar` (E2E de TASK-049).
@@ -130,6 +137,7 @@ jobs:
     permissions:
       contents: read
       actions: read
+      checks: read
     outputs:
       tc2-sha: ${{ steps.version.outputs.sha }}
     steps:
@@ -274,6 +282,63 @@ jobs:
           gh api "repos/$REPO/actions/artifacts/$id/zip" > "$RUNNER_TEMP/informe.zip"
           # tc2 vuelve a aplicar su tope (64 KiB) al leerlo.
           unzip -p "$RUNNER_TEMP/informe.zip" informe-verificacion.json | head -c 65537 > "$RUNNER_TEMP/informe.json" || rm -f "$RUNNER_TEMP/informe.json"
+      # F6-6: la ronda anterior es el ultimo check revision-ia de la App del
+      # revisor en un commit anterior del PR (no la rama de estado, que
+      # escribe el agente). Se vuelcan en crudo los check-runs de cada commit,
+      # del mas nuevo al mas viejo (GITHUB_TOKEN con checks: read), y decide
+      # tc2; del elegido, sus paginas de anotaciones. Pasado el tope de
+      # commits, falla (el check sale incompleta).
+      - name: Ronda anterior
+        env:
+          GH_TOKEN: ${{ github.token }}
+          REPO: ${{ github.repository }}
+          HEAD_SHA: ${{ inputs.head-sha }}
+          BASE: ${{ inputs.base }}
+          RAMA: ${{ inputs.rama }}
+        run: |
+          set -euo pipefail
+          dir="$RUNNER_TEMP/revision-ia/ronda"
+          mkdir -p "$dir/checks" "$dir/anotaciones"
+          git -C base rev-list --first-parent --max-count=102 "origin/$BASE..$HEAD_SHA" > "$RUNNER_TEMP/commits.txt"
+          if [ "$(wc -l < "$RUNNER_TEMP/commits.txt")" -gt 101 ]; then
+            echo "::error::revision-ia: el PR pasa de 100 commits sin el head: no se busca la ronda anterior (falla en cerrado)" >&2
+            exit 1
+          fi
+          checks=()
+          anterior=ninguno
+          sha=""
+          k=0
+          for c in $(tail -n +2 "$RUNNER_TEMP/commits.txt"); do
+            f="$dir/checks/$k.json"
+            gh api --method GET "repos/$REPO/commits/$c/check-runs" -f check_name=revision-ia -f per_page=100 > "$f"
+            checks+=(--check-runs "$f")
+            k=$((k + 1))
+            linea=$(cd base && tc2 hallazgos anterior "${checks[@]}")
+            echo "$c: $linea"
+            anterior=$(printf '%s\n' "$linea" | sed -nE 's/.* anterior=([0-9]+|ninguno).*/\1/p')
+            if [ "$anterior" != ninguno ]; then
+              sha=$(printf '%s\n' "$linea" | sed -nE 's/.* sha=([0-9a-f]{40}).*/\1/p')
+              break
+            fi
+          done
+          anotaciones=()
+          rango=("origin/$BASE...$HEAD_SHA")
+          if [ "$anterior" != ninguno ]; then
+            for p in $(seq 1 30); do
+              f="$dir/anotaciones/$p.json"
+              gh api --method GET "repos/$REPO/check-runs/$anterior/annotations" -f per_page=100 -f page="$p" > "$f"
+              anotaciones+=(--anotaciones "$f")
+              [ "$(jq 'length' "$f")" -eq 100 ] || break
+            done
+            # Tras un rebase el commit anterior no es ancestro: el diff vuelve
+            # a base...head y los abiertos se arrastran igual.
+            if git -C base merge-base --is-ancestor "$sha" "$HEAD_SHA"; then rango=("$sha" "$HEAD_SHA"); fi
+          fi
+          git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "${rango[@]}" > "$dir/rutas.txt"
+          tarea=()
+          id=$(printf '%s' "$RAMA" | sed -nE 's#^(feature|fix|hotfix)/task-([0-9]{3,})-.*#TASK-\2#p')
+          if [ -n "$id" ]; then tarea=(--tarea "$id"); fi
+          (cd base && tc2 hallazgos ronda "${checks[@]}" "${anotaciones[@]}" --rutas "$dir/rutas.txt" "${tarea[@]}" --salida "$dir") | tee "$dir/linea.txt"
       - name: Seleccion y prompts
         env:
           HEAD_SHA: ${{ inputs.head-sha }}
@@ -288,9 +353,21 @@ jobs:
           set -eu
           salida="$RUNNER_TEMP/revision-ia"
           mkdir -p "$salida/prompts" "$salida/diffs"
+          # F6-6: la ronda la decidio el paso anterior. En incremental el diff
+          # es el de lo cambiado desde el commit de la ronda anterior.
+          ronda="$salida/ronda/ronda.json"
+          modo=$(jq -r '.modo' "$ronda")
+          numero=$(jq -r '.ronda' "$ronda")
+          anterior=$(jq -r '.sha_anterior // ""' "$ronda")
+          rango=("origin/$BASE...$HEAD_SHA")
+          if [ "$modo" = incremental ] && git -C base merge-base --is-ancestor "$anterior" "$HEAD_SHA"; then rango=("$anterior" "$HEAD_SHA"); fi
           # Rutas literales separadas por NUL: sin -z git entrecomilla y escapa
           # lo no ASCII ("\303\251.rs") y la ruta no casaria con nada.
-          git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "origin/$BASE...$HEAD_SHA" > "$salida/rutas.txt"
+          git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "${rango[@]}" > "$salida/rutas.txt"
+          # Los ficheros de los abiertos graves tambien eligen revisora.
+          if [ "$modo" = incremental ]; then
+            jq -j '.abiertos[] | select(.hallazgo.severidad != "MENOR") | .hallazgo.fichero + "\u0000"' "$ronda" >> "$salida/rutas.txt"
+          fi
           tarea=()
           id=$(printf '%s' "$RAMA" | sed -nE 's#^(feature|fix|hotfix)/task-([0-9]{3,})-.*#TASK-\2#p')
           if [ -n "$id" ]; then tarea=(--tarea "$id"); fi
@@ -305,13 +382,21 @@ jobs:
             cp "$RUNNER_TEMP/informe.json" "$salida/informe.json"
           fi
           (cd base && tc2 revisoras --json --rutas "$salida/rutas.txt" --nul "${tarea[@]}" \
-            --plugin "$PLUGIN_NOMBRE=$PLUGIN" "${informe[@]}" --detalle "$salida/detalle.json") > "$salida/seleccion.json"
+            --plugin "$PLUGIN_NOMBRE=$PLUGIN" "${informe[@]}" --ronda "$numero" --detalle "$salida/detalle.json") > "$salida/seleccion.json"
           jq -r '.claves | "revisoras=\(.revisoras | join(",")) agentes=\(.agentes | join(",")) modelo=\(.modelo) modelo-api=\(.["modelo-api"]) reglas=\(.reglas | join(",")) avisos=\(.avisos | join(",")) carril=\(.carril) complejidad=\(.complejidad) informe=\(.informe // "sin-informe")"' "$salida/seleccion.json" > "$salida/resumen.txt"
           if [ -n "$de" ]; then sed -i "s/\$/ informe-de=run-$de/" "$salida/resumen.txt"; fi
+          sed -i "s/\$/ ronda=$numero modo=$modo/" "$salida/resumen.txt"
           cat "$salida/resumen.txt"
           modelo=$(jq -r '.claves["modelo-api"]' "$salida/seleccion.json")
           : > "$salida/prompts/lista.tsv"
-          n=$(jq '.revisoras | length' "$salida/detalle.json")
+          case "$modo" in
+            completa) n=$(jq '.revisoras | length' "$salida/detalle.json") ;;
+            incremental)
+              n=$(jq '.revisoras | length' "$salida/detalle.json")
+              tc2 hallazgos abiertos --ronda "$ronda" --detalle "$salida/detalle.json" --salida "$salida/abiertos" ;;
+            # ligera y tope: sin modelo; validar saca el veredicto de ronda.json.
+            *) n=0; echo "revision-ia: ronda $numero en modo $modo: no se consulta al modelo" ;;
+          esac
           for i in $(seq 0 $((n - 1))); do
             rid=$(jq -r ".revisoras[$i].id" "$salida/detalle.json")
             skill=$(jq -r ".revisoras[$i].skill // \"\"" "$salida/detalle.json")
@@ -323,7 +408,7 @@ jobs:
             bytes=0
             : > "$RUNNER_TEMP/diff-$i.txt"
             if [ "${#rutas[@]}" -gt 0 ]; then
-              git -c core.quotepath=false --literal-pathspecs -C base diff --no-renames "origin/$BASE...$HEAD_SHA" -- "${rutas[@]}" > "$RUNNER_TEMP/diff-$i.txt"
+              git -c core.quotepath=false --literal-pathspecs -C base diff --no-renames "${rango[@]}" -- "${rutas[@]}" > "$RUNNER_TEMP/diff-$i.txt"
               bytes=$(wc -c < "$RUNNER_TEMP/diff-$i.txt")
             fi
             head -c "$TOPE" "$RUNNER_TEMP/diff-$i.txt" > "$RUNNER_TEMP/diff-$i-cortado.txt"
@@ -345,6 +430,14 @@ jobs:
               echo
               if [ -s "$salida/seccion.md" ]; then cat "$salida/seccion.md"; else echo "## Informe de verificacion (F5-4)"; echo; echo "No hay informe legible: no des por verificado nada que no se vea en el diff."; fi
               echo
+              if [ "$modo" = incremental ]; then
+                echo "## Hallazgos abiertos de la ronda anterior"
+                echo
+                echo "Ronda $numero (F6-6): el diff de abajo es lo cambiado desde el commit $anterior de la ronda anterior. Estos hallazgos siguen abiertos; cierra solo los que el diff arregla (ver «Lo que devuelves»)."
+                echo
+                cat "$salida/abiertos/abiertos-$i.md"
+                echo
+              fi
               echo "## Diff de las rutas de esta revisora"
               echo
               echo "${cerca}diff"
@@ -548,6 +641,9 @@ jobs:
           : > "$prosa"
           informe=()
           if [ -s "$dir/informe.json" ]; then informe=(--informe "$dir/informe.json"); fi
+          # F6-6: la ronda de preparar (nuevos, persisten y cerrados).
+          ronda=()
+          if [ -s "$dir/ronda/ronda.json" ]; then ronda=(--ronda "$dir/ronda/ronda.json"); fi
           while IFS="$(printf '\t')" read -r i rid modelo; do
             [ -n "$i" ] || continue
             r="R$i"
@@ -560,7 +656,9 @@ jobs:
               : > "$respuesta"
             fi
             : > "$out/resumen-$i.md"
-            if ! linea=$(tc2 hallazgos validar --respuesta "$respuesta" --diff "$dir/diffs/$i.txt" --revisora "$rid" "${informe[@]}" \
+            abiertos=()
+            if [ -f "$dir/abiertos/abiertos-$i.json" ]; then abiertos=(--abiertos "$dir/abiertos/abiertos-$i.json"); fi
+            if ! linea=$(tc2 hallazgos validar --respuesta "$respuesta" --diff "$dir/diffs/$i.txt" --revisora "$rid" "${informe[@]}" "${abiertos[@]}" \
               --salida "$out/hallazgos-$i.json" --resumen "$out/resumen-$i.md"); then
               linea="hallazgos error"
             fi
@@ -592,7 +690,7 @@ jobs:
             } >> "$out/texto.md"
           fi
           # F6-5: la conclusion la saca tc2; publicar solo la transporta.
-          tc2 hallazgos veredicto --salida "$out" --extra "$out/resumen.txt" "${veredicto[@]}" | tee "$out/veredicto.txt"
+          tc2 hallazgos veredicto --salida "$out" --extra "$out/resumen.txt" "${ronda[@]}" "${veredicto[@]}" | tee "$out/veredicto.txt"
           cat "$out/resumen.txt"
       - name: Hallazgos para publicar
         uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
diff --git a/crates/tc2/esquemas/hallazgos.schema.json b/crates/tc2/esquemas/hallazgos.schema.json
index bce971a..bdc8f28 100644
--- a/crates/tc2/esquemas/hallazgos.schema.json
+++ b/crates/tc2/esquemas/hallazgos.schema.json
@@ -1,14 +1,14 @@
 {
   "$schema": "https://json-schema.org/draft/2020-12/schema",
-  "title": "Hallazgos del revisor IA, version 2 (F6-4 y F6-5)",
-  "description": "Salida de `tc2 hallazgos validar --salida`: los hallazgos aceptados y los rechazados con su motivo, para una revisora. $defs/respuesta es lo que la revisora devuelve en su unico bloque json (lo valida hallazgos::validar). Reglas que el esquema no expresa y comprueba tc2: el id no se repite; fichero y linea estan en el lado nuevo del diff que vio la revisora; linea_original de un mutante es el texto de esa linea y la mutacion es distinta; esperado y obtenido de un borde son distintos; paso_rojo es un paso en rojo del informe de verificacion (F5-4). La evidencia se declara y se valida, no se ejecuta (F6-6). Una clave desconocida falla cerrado. F6-5: cada hallazgo lleva su clase (lista cerrada de F1-15); $defs/veredicto es el check.json de `tc2 hallazgos veredicto` (conclusion success o failure, nunca neutral), $defs/lote cada anotaciones-<k>.json y $defs/informe-tarea el informe que `tc2 review` guarda en TASK-NNN/revision/.",
+  "title": "Hallazgos del revisor IA, version 3 (F6-4, F6-5 y F6-6)",
+  "description": "Salida de `tc2 hallazgos validar --salida`: los hallazgos aceptados y los rechazados con su motivo, para una revisora. $defs/respuesta es lo que la revisora devuelve en su unico bloque json (lo valida hallazgos::validar). Reglas que el esquema no expresa y comprueba tc2: el id no se repite; fichero y linea estan en el lado nuevo del diff que vio la revisora; linea_original de un mutante es el texto de esa linea y la mutacion es distinta; esperado y obtenido de un borde son distintos; paso_rojo es un paso en rojo del informe de verificacion (F5-4). La evidencia se declara y se valida, no se ejecuta (F6-6). Una clave desconocida falla cerrado. F6-5: cada hallazgo lleva su clase (lista cerrada de F1-15); $defs/veredicto es el check.json de `tc2 hallazgos veredicto` (conclusion success o failure, nunca neutral), $defs/lote cada anotaciones-<k>.json y $defs/informe-tarea el informe que `tc2 review` guarda en TASK-NNN/revision/. F6-6 (v3): rondas incrementales. La respuesta puede ser la lista (ronda 1) o un objeto {hallazgos, cierres}; un cierre ($defs/cierre) cierra un abierto CRITICO o IMPORTANTE de la revisora si su linea esta en el diff incremental y, si el abierto traia paso_rojo, ese paso ya no esta en rojo. aceptado-de gana ronda y estado (un raw_details v2 sin ellos se lee como ronda 1 y abierto); los recuentos son de los abiertos; los cerrados en la ronda van como anotacion notice; $defs/ronda es el ronda.json de `tc2 hallazgos ronda`. El titulo del check v3 lleva ronda= y tc2 sigue leyendo el v2.",
   "type": "object",
   "additionalProperties": false,
-  "required": ["version", "revisora", "aceptados", "rechazados"],
+  "required": ["version", "revisora", "aceptados", "rechazados", "cierres"],
   "properties": {
     "version": {
-      "description": "version del contrato; otra distinta de 2 es error",
-      "const": 2
+      "description": "version del contrato; otra distinta de 3 es error",
+      "const": 3
     },
     "revisora": {
       "description": "id de la revisora (tc2 revisoras)",
@@ -22,11 +22,34 @@
     "rechazados": {
       "type": "array",
       "items": { "$ref": "#/$defs/rechazo" }
+    },
+    "cierres": {
+      "description": "cierres aceptados (F6-6; vacia en la ronda 1)",
+      "type": "array",
+      "items": { "$ref": "#/$defs/cierre-aceptado" }
     }
   },
   "$defs": {
     "respuesta": {
-      "description": "contenido del unico bloque json de la respuesta de la revisora; vacia si no hay hallazgos",
+      "description": "contenido del unico bloque json de la respuesta de la revisora: la lista de hallazgos (vacia si no hay ninguno) o, en una ronda incremental (F6-6), un objeto con los hallazgos nuevos y los cierres de abiertos",
+      "oneOf": [
+        { "$ref": "#/$defs/lista-hallazgos" },
+        {
+          "type": "object",
+          "additionalProperties": false,
+          "required": ["hallazgos"],
+          "properties": {
+            "hallazgos": { "$ref": "#/$defs/lista-hallazgos" },
+            "cierres": {
+              "type": "array",
+              "maxItems": 100,
+              "items": { "$ref": "#/$defs/cierre" }
+            }
+          }
+        }
+      ]
+    },
+    "lista-hallazgos": {
       "type": "array",
       "maxItems": 100,
       "items": { "$ref": "#/$defs/hallazgo" }
@@ -117,18 +140,46 @@
         "obtenido": { "type": "string", "minLength": 1 }
       }
     },
+    "cierre": {
+      "description": "cierre de un abierto CRITICO o IMPORTANTE (F6-6): su id, la linea del lado nuevo del diff incremental que lo arregla y por que",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["id", "fichero", "linea", "motivo"],
+      "properties": {
+        "id": { "type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$" },
+        "fichero": { "type": "string", "minLength": 1 },
+        "linea": { "type": "integer", "minimum": 1 },
+        "motivo": { "type": "string", "minLength": 1 }
+      }
+    },
+    "cierre-aceptado": {
+      "description": "un cierre aceptado, con la revisora y la ronda del abierto que cierra",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["revisora", "ronda", "id", "fichero", "linea", "motivo"],
+      "properties": {
+        "revisora": { "type": "string", "minLength": 1 },
+        "ronda": { "type": "integer", "minimum": 1 },
+        "id": { "type": "string", "minLength": 1 },
+        "fichero": { "type": "string", "minLength": 1 },
+        "linea": { "type": "integer", "minimum": 1 },
+        "motivo": { "type": "string", "minLength": 1 }
+      }
+    },
     "aceptado-de": {
-      "description": "un aceptado con su revisora: el raw_details de su anotacion",
+      "description": "un aceptado con su revisora, la ronda en que se encontro y si sigue abierto: el raw_details de su anotacion",
       "type": "object",
       "additionalProperties": false,
-      "required": ["revisora", "hallazgo"],
+      "required": ["revisora", "hallazgo", "ronda", "estado"],
       "properties": {
         "revisora": { "type": "string", "minLength": 1 },
-        "hallazgo": { "$ref": "#/$defs/hallazgo" }
+        "hallazgo": { "$ref": "#/$defs/hallazgo" },
+        "ronda": { "type": "integer", "minimum": 1 },
+        "estado": { "enum": ["abierto", "cerrado"] }
       }
     },
     "anotacion": {
-      "description": "anotacion del check-run: failure para CRITICO e IMPORTANTE, warning para MENOR; raw_details es un aceptado-de en JSON",
+      "description": "anotacion del check-run: failure para CRITICO e IMPORTANTE abiertos, warning para MENOR abiertos y notice para los cerrados en esta ronda (F6-6); raw_details es un aceptado-de en JSON",
       "type": "object",
       "additionalProperties": false,
       "required": ["path", "start_line", "end_line", "annotation_level", "title", "message", "raw_details"],
@@ -136,7 +187,7 @@
         "path": { "type": "string", "minLength": 1 },
         "start_line": { "type": "integer", "minimum": 1 },
         "end_line": { "type": "integer", "minimum": 1 },
-        "annotation_level": { "enum": ["failure", "warning"] },
+        "annotation_level": { "enum": ["failure", "warning", "notice"] },
         "title": { "type": "string", "minLength": 1, "maxLength": 255 },
         "message": { "type": "string", "minLength": 1 },
         "raw_details": { "type": "string", "minLength": 2, "maxLength": 65536 }
@@ -176,20 +227,24 @@
       }
     },
     "informe-tarea": {
-      "description": "TASK-NNN/revision/revision-ia-<sha12>.json: lo leido del check publicado, sin la prosa del modelo",
+      "description": "TASK-NNN/revision/revision-ia-<sha12>.json: lo leido del check publicado, sin la prosa del modelo; los recuentos por severidad son de los abiertos",
       "type": "object",
       "additionalProperties": false,
-      "required": ["version", "tarea", "pr", "sha", "conclusion", "veredicto", "criticos", "importantes", "menores", "resumen", "aceptados"],
+      "required": ["version", "tarea", "pr", "sha", "conclusion", "veredicto", "ronda", "criticos", "importantes", "menores", "nuevos", "persisten", "cerrados", "resumen", "aceptados"],
       "properties": {
-        "version": { "const": 2 },
+        "version": { "const": 3 },
         "tarea": { "type": "string", "minLength": 1 },
         "pr": { "type": "integer", "minimum": 1 },
         "sha": { "type": "string", "minLength": 40 },
         "conclusion": { "enum": ["success", "failure"] },
-        "veredicto": { "enum": ["rojo", "verde", "incompleta"] },
+        "veredicto": { "enum": ["rojo", "verde", "incompleta", "tope"] },
+        "ronda": { "type": "integer", "minimum": 1 },
         "criticos": { "type": "integer", "minimum": 0 },
         "importantes": { "type": "integer", "minimum": 0 },
         "menores": { "type": "integer", "minimum": 0 },
+        "nuevos": { "type": "integer", "minimum": 0 },
+        "persisten": { "type": "integer", "minimum": 0 },
+        "cerrados": { "type": "integer", "minimum": 0 },
         "resumen": { "type": "string" },
         "aceptados": {
           "type": "array",
@@ -197,6 +252,27 @@
         }
       }
     },
+    "ronda": {
+      "description": "ronda.json de `tc2 hallazgos ronda` (F6-6): la ronda, el modo, el commit del check anterior (base del diff incremental), el tope efectivo, los abiertos heredados y los MENOR heredados que se cierran porque su fichero cambio",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["version", "ronda", "modo", "sha_anterior", "tope", "abiertos", "cerrados"],
+      "properties": {
+        "version": { "const": 3 },
+        "ronda": { "type": "integer", "minimum": 1 },
+        "modo": { "enum": ["completa", "incremental", "ligera", "tope"] },
+        "sha_anterior": { "type": ["string", "null"] },
+        "tope": { "type": "integer", "minimum": 1 },
+        "abiertos": {
+          "type": "array",
+          "items": { "$ref": "#/$defs/aceptado-de" }
+        },
+        "cerrados": {
+          "type": "array",
+          "items": { "$ref": "#/$defs/aceptado-de" }
+        }
+      }
+    },
     "rechazo": {
       "description": "un hallazgo rechazado (indice en la lista) o toda la respuesta (indice null)",
       "type": "object",
@@ -219,7 +295,10 @@
             "mutante-no-coincide",
             "mutante-sin-cambio",
             "borde-sin-diferencia",
-            "paso-rojo-no-esta"
+            "paso-rojo-no-esta",
+            "cierre-desconocido",
+            "cierre-fuera-del-diff",
+            "paso-rojo-sigue-rojo"
           ]
         },
         "detalle": { "type": "string" }
diff --git a/docs/adr/F1-10-carriles-y-modo-automatico.md b/docs/adr/F1-10-carriles-y-modo-automatico.md
index 201484a..270e09e 100644
--- a/docs/adr/F1-10-carriles-y-modo-automatico.md
+++ b/docs/adr/F1-10-carriles-y-modo-automatico.md
@@ -747,3 +747,15 @@ Evidencia: `crates/tc2/tests/plan.rs` y los unitarios de `carril.rs`
 Evidencia: `crates/tc2/tests/aprobar.rs` y los unitarios de
 `crates/tc2/src/plan/aprobar.rs` y `operacion.rs`
 (`cambios_pedidos_cierran_la_ronda`).
+
+## Adenda (F6-6, TASK-052) — `tope` del revisor IA en CI
+
+- `tope_rondas` ya lo usa la CI: `tc2 hallazgos ronda` toma `tope_rondas` de
+  la configuración de la base más un elemento por cada línea `- ...` de
+  primer nivel en `## Rondas extra` de la tarea. Pasado ese tope con un
+  CRITICO o IMPORTANTE abierto, el check `revision-ia` sale `tope`
+  (`failure`) sin consultar al modelo y con los abiertos arrastrados. Es la
+  señal en CI de la parada `preguntar` de esta ADR: la traducción a esa
+  parada en el modo automático es de F4-4. Con el anterior en verde y cambios
+  solo en ficheros con MENOR abiertos, la ronda es `ligera` (sin modelo,
+  verde, la ronda no sube): los MENORES no gastan ronda.
diff --git a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
index 6bde44c..98a06d1 100644
--- a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
+++ b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
@@ -868,3 +868,22 @@ la App, el prompt con la sección del informe y sin ejecutar la suite).
   resto (`verde`). Nunca `neutral`: GitHub lo da por cumplido en un check
   obligatorio y una revisión que no ocurrió pasaría la puerta (RR-04). Guía:
   `docs/ci/revision-ia.md`, «Veredicto del check».
+
+## Adenda (F6-6, TASK-052) — rondas: S13 se aplica y el título lleva la ronda
+
+- **S13 activa**: `tc2 revisoras --ronda <n>` deja de fijar la ronda a 1. En
+  CI la ronda sale de `ronda.json` de `tc2 hallazgos ronda` (el check
+  `revision-ia` anterior de la App del revisor), así que desde la ronda 2 la
+  fila S13 baja el modelo un escalón fuera de `riesgo-alto` y de
+  `dominio=puertas-seguridad` (con el detector de F4-6 pendiente, este
+  último no se cumple nunca todavía). Las rutas de la selección en modo
+  incremental son las del diff desde el commit anterior más los ficheros de
+  los abiertos CRITICO e IMPORTANTE. La selección anotada en local
+  (`roles.revision` en `tc2 review`) sigue con ronda 1: la fuente de la ronda
+  es el check de CI.
+- **Título del check**: `revision-ia: <veredicto> (ronda=R criticos=N
+  importantes=M menores=K)`, con los recuentos de los **abiertos**; los
+  cerrados en la ronda van como anotación `notice`. `tc2` sigue leyendo el
+  título v2 de F6-5 (sin `ronda=`) como ronda 1. Veredicto nuevo `tope`
+  (`failure`): pasado el tope de rondas con un CRITICO o IMPORTANTE abierto.
+  Contrato de hallazgos v3; guía: `docs/ci/revision-ia.md`, «Rondas (F6-6)».
diff --git a/docs/adr/F1-16-lotes-desatendidos.md b/docs/adr/F1-16-lotes-desatendidos.md
index cd09ddb..ed53e5f 100644
--- a/docs/adr/F1-16-lotes-desatendidos.md
+++ b/docs/adr/F1-16-lotes-desatendidos.md
@@ -932,3 +932,13 @@ taskcodetwo/tareas: lotes/noche-173-210.yml (diario), tareas/TASK-NNN/tarea.md (
   del check `carril` (F6) y no se implementa en F4-3.
 - No cambia F1-10: que una tarea de riesgo alto no entre en el lote afecta a
   lanzarlo, no a aprobar su plan.
+
+## Adenda (F6-6, TASK-052) — `## Rondas extra` sube el tope de la CI
+
+- El formato que lee `tc2 hallazgos ronda` es un elemento de lista de primer
+  nivel (`- ...` o `* ...`) por ronda extra en `## Rondas extra` de la tarea
+  (rama de estado); los subelementos sangrados no cuentan. Cada uno suma uno
+  a `tope_rondas` de la base. Con D6a nadie escribe esa sección de noche: la
+  ronda extra la decide la persona por la mañana. Si el agente la escribiera,
+  solo ganaría rondas (tokens), no saltaría ninguna puerta: la aprobación de
+  la persona sigue siendo obligatoria.
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 0c243d3..43e16e2 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -824,3 +824,32 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   de la release en el runner propio con el usuario de servicio y los datos de
   solo lectura; `github.event.repository.private` llega con valor al
   reutilizable.
+
+## Adenda (F6-6, TASK-052) — rondas incrementales
+
+- **`checks: read` en `preparar` y en el llamador**: `preparar` lee el check
+  `revision-ia` de la ronda anterior con el `GITHUB_TOKEN` (`checks: read`,
+  lectura ya admitida por la lista blanca de la decisión 1 para `buscar` de
+  F5-2). El llamador tiene que darlo (un reutilizable no gana permisos que
+  el llamador no da); sin él, el paso «Ronda anterior» falla y el check sale
+  `incompleta` (falla en cerrado). El lint del llamador (`llamador-revision`)
+  admite `checks: read` y rechaza `checks: write`. Ningún job nuevo.
+- **La ronda anterior sale del check de la App del revisor**, filtrado por
+  su `app.id` (`plataforma.revisor_id` de la base), no del informe
+  `revision-ia-<sha12>.json` de la rama de estado, que escribe el agente: si
+  el agente borrara un IMPORTANTE de su informe, la ronda siguiente lo
+  seguiría viendo. Un check que no casa con sus anotaciones (título,
+  conclusión y abiertos) para el cálculo de la ronda (falla en cerrado).
+- **Cierre explícito**: un abierto CRITICO o IMPORTANTE solo se cierra con un
+  cierre anclado al diff incremental (y, si traía `paso_rojo`, con ese paso
+  ya en verde en el informe F5-4); el silencio del modelo no cierra nada. Los
+  abiertos reinyectados en el prompt pasan por `plano`; la defensa contra la
+  inyección sigue siendo de F6-7.
+- **Decisión 3 (ejecución de la evidencia), revisada por la persona el
+  2026-11-22**: la adenda F6-4 decía «F6-6 la ejecuta»; **no entra en
+  F6-6**. Ejecutarla exigiría un comando de test por proyecto (clave nueva de
+  F1-11), un job de clase ejecuta que corra código del PR con la mutación del
+  modelo y saldo en la API para probarlo, y pasa de la fila F6-6 del plan.
+  Queda como acción de Carlos crear una fila F6-x. Mientras, RR-03 se cumple
+  en parte: se exige y valida la evidencia y en el cierre solo se contrasta
+  `paso_rojo` con el informe F5-4.
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index 2d5346a..1ef537b 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -1,6 +1,6 @@
-# Revisor IA en CI (F6-3, F6-4, F6-5)
+# Revisor IA en CI (F6-3, F6-4, F6-5, F6-6)
 
-TASK-049, TASK-050 y TASK-051. El check `revision-ia` lo publica la **App del revisor** desde un
+TASK-049, TASK-050, TASK-051 y TASK-052. El check `revision-ia` lo publica la **App del revisor** desde un
 workflow reutilizable que corre cuando termina `verificacion` en un PR
 ([`.github/workflows/revision-ia.yml`](../../.github/workflows/revision-ia.yml)).
 Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
@@ -9,7 +9,8 @@ Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
 y el catálogo de la base. Qué hallazgos cuentan tampoco: `tc2 hallazgos
 validar` rechaza todo hallazgo sin evidencia de su tipo anclada a una línea
 del diff (F6-4, «Hallazgos con evidencia»). Ni el color del check: lo saca
-`tc2 hallazgos veredicto` (F6-5, «Veredicto del check»). Clases de jobs y secretos: ADR
+`tc2 hallazgos veredicto` (F6-5, «Veredicto del check»). Ni qué se mira en
+cada ronda: `tc2 hallazgos ronda` (F6-6, «Rondas»). Clases de jobs y secretos: ADR
 [F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md) (adendas F6-3 y F6-4).
 
 ## Qué hace
@@ -131,6 +132,8 @@ jobs:
     permissions:
       contents: read
       actions: read
+      # F6-6: preparar lee el check revision-ia de la ronda anterior.
+      checks: read
     uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
     secrets: inherit
     with:
@@ -141,6 +144,10 @@ jobs:
       base: ${{ github.event.workflow_run.pull_requests[0].base.ref }}
 ```
 
+Sin `checks: read` en el llamador, `preparar` no puede pedirlo (un
+reutilizable no gana permisos que el llamador no da): el paso «Ronda
+anterior» falla y el check sale `incompleta` (falla en cerrado).
+
 `workflows: [verificacion]` es el `name:` del workflow del proyecto que
 llama a `verificacion.yml`. `<sha>` es el commit de TaskCodeTwo con este
 workflow (40 hexadecimales, nunca una rama); `tc2` se compila de ese mismo
@@ -287,7 +294,8 @@ decide, sin GitHub:
 | Veredicto | Cuándo | Conclusión |
 |---|---|---|
 | `incompleta` | no hay revisoras, o alguna no dejó JSON o su respuesta entera se rechazó (`sin-bloque`, `json-invalido`...) | `failure` |
-| `rojo` | algún CRITICO o IMPORTANTE **aceptado** (en F6-5 todo aceptado está abierto; el cierre llega con F6-6) | `failure` |
+| `rojo` | algún CRITICO o IMPORTANTE **abierto** (nuevo o de una ronda anterior sin cierre válido) | `failure` |
+| `tope` | pasado el tope de rondas con un CRITICO o IMPORTANTE abierto (F6-6) | `failure` |
 | `verde` | el resto (solo MENOR, o nada) | `success` |
 
 Un CRITICO rechazado (sin evidencia, fuera del diff...) no pone el check en
@@ -299,7 +307,8 @@ F6-5 de F1-15).
 Escribe en `<dir>`:
 
 - `check.json` (`$defs/veredicto`): `conclusion`, `output.title`
-  (`revision-ia: <veredicto> (criticos=N importantes=M menores=K)`),
+  (`revision-ia: <veredicto> (ronda=R criticos=N importantes=M menores=K)`,
+  con los abiertos; `tc2` sigue leyendo el título v2 de F6-5, sin `ronda=`),
   `output.summary` (veredicto, recuentos, tabla por revisora y las líneas de
   `--extra` por `plano`) y las primeras 50 anotaciones;
 - `anotaciones-<k>.json` (`$defs/lote`), el cuerpo de cada `PATCH` con las
@@ -307,12 +316,14 @@ Escribe en `<dir>`:
 
 Cada aceptado es una **anotación** del check (se ve en «Files changed»; no
 hacen falta comentarios de PR ni permisos nuevos): `path` y `start_line` del
-hallazgo, nivel `failure` para CRITICO e IMPORTANTE y `warning` para MENOR,
-título `<id> <SEVERIDAD> <clase> (<revisora>)` y mensaje por `plano`, y en
-`raw_details` el aceptado con su revisora en JSON (`$defs/aceptado-de`).
-Línea: `hallazgos ok veredicto=<v> conclusion=<c> criticos=N importantes=M
-menores=K revisoras=R incompletas=I lotes=L`; un rojo no es un fallo de la
-orden.
+hallazgo, nivel `failure` para CRITICO e IMPORTANTE, `warning` para MENOR y
+`notice` para los cerrados en esa ronda (F6-6), título `<id> <SEVERIDAD>
+<clase> (<revisora>)` y mensaje por `plano`, y en `raw_details` el aceptado
+con su revisora, su ronda y su estado en JSON (`$defs/aceptado-de`; uno de
+F6-5 sin ronda ni estado se lee como ronda 1 y abierto). Línea: `hallazgos
+ok veredicto=<v> conclusion=<c> criticos=N importantes=M menores=K
+revisoras=R incompletas=I lotes=L ronda=R modo=<m> nuevos=N persisten=P
+cerrados=C`; un rojo no es un fallo de la orden.
 
 `publicar` no calcula nada: añade `name`, `head_sha`, `status`,
 `details_url` y `output.text`, y acepta la conclusión de `check.json` solo si
@@ -340,13 +351,71 @@ de escritura de la rama de estado:
 - `TASK-NNN/revision/revision-ia-<sha12>.md`: el veredicto y la tabla de
   aceptados, generados por `tc2` con `plano`, **sin** la prosa del modelo.
 
-La línea de `review` gana `hallazgos=<rojo|verde|incompleta|sin-leer>` e
-`informe=<ruta>`. Repetirla con el mismo check sale `sin-cambios`. Con el
+La línea de `review` gana `hallazgos=<rojo|verde|incompleta|tope|sin-leer>`,
+`informe=<ruta>` y (F6-6) `ronda=R nuevos=N cerrados=C`. Antes de empujar,
+si el PR de la rama ya existe y su check `revision-ia` terminó, `review`
+guarda el informe de ese head (el de la ronda anterior): después del push el
+PR apunta al commit nuevo y el check anterior ya no se lee por el PR. Repetirla con el mismo check sale `sin-cambios`. Con el
 check pendiente no se lee ni se escribe nada: el informe se guarda al
 relanzar `review` cuando el check ya terminó. La CI no escribe en la rama de
 estado (necesitaría `contents: write`, que solo tiene el agente). Los
 rechazados quedan solo como recuento y motivos del resumen.
 
+## Rondas (F6-6)
+
+TASK-052, RF-33. Cada check `revision-ia` describe su ronda (título con
+`ronda=`) y lleva en sus anotaciones **todos** los hallazgos que siguen
+abiertos. La ronda siguiente solo necesita ese check, publicado por la App
+del revisor; no la rama de estado, que escribe el agente (F1-8): si el
+agente borrara un IMPORTANTE de su informe, la ronda siguiente lo seguiría
+viendo.
+
+En `preparar`, el paso «Ronda anterior» recorre los commits del PR (`git
+rev-list --first-parent origin/<base>..<head>`, sin el head, como mucho 100;
+si hay más, falla y el check sale `incompleta`) del más nuevo al más viejo y
+vuelca en crudo sus check-runs `revision-ia` (`GITHUB_TOKEN` con `checks:
+read`). Tras cada volcado, `tc2 hallazgos anterior` dice si ya hay ronda
+anterior: el `revision-ia` más nuevo y terminado de la App del revisor
+(`plataforma.revisor_id` de la configuración de la base; sin él, no hay
+ronda anterior) que dice `rojo` o `verde`. Una ronda `incompleta` no cuenta
+ni sirve de base; una `tope` tampoco (no revisó nada). Del elegido se bajan
+sus páginas de anotaciones y `tc2 hallazgos ronda` exige que título,
+conclusión y abiertos casen (la regla de `leer-revision`); si no, falla
+(falla en cerrado) y el check sale `incompleta`. Con los ficheros cambiados
+desde ese commit (`git diff --name-only <sha>..<head>`; tras un rebase que lo
+deja fuera de la historia, `base...head`) decide `ronda.json`:
+
+| Modo | Cuándo | Qué pasa |
+|---|---|---|
+| `completa` | sin ronda anterior | ronda 1: todo el diff contra la base |
+| `incremental` | el resto | la ronda sube; cada revisora ve el diff desde el commit anterior y sus abiertos CRITICO e IMPORTANTE |
+| `ligera` | anterior `verde` y los cambios solo tocan ficheros con MENOR abiertos | sin modelo; verde; la ronda no sube |
+| `tope` | anterior `rojo` y la ronda siguiente pasa del tope | sin modelo; dictamen `tope` (`failure`) con los abiertos arrastrados |
+
+El tope efectivo es `tope_rondas` de la configuración de la base más un
+elemento por cada línea `- ...` de primer nivel en `## Rondas extra` de la
+tarea (rama de estado). La ronda extra la decide la persona (F1-16, D6a):
+F4-4 traducirá `tope` a la parada `preguntar` de F1-10.
+
+Cierre (falla en cerrado): en modo incremental la respuesta es un objeto
+`{"hallazgos": [...], "cierres": [...]}`. Un abierto CRITICO o IMPORTANTE solo
+se cierra con un cierre `{id, fichero, linea, motivo}` cuya línea esté en el
+lado nuevo del diff incremental que vio la revisora; si el abierto traía
+`paso_rojo`, ese paso tiene que estar ya en verde en el informe F5-4 del head
+nuevo. Motivos nuevos: `cierre-desconocido`, `cierre-fuera-del-diff`,
+`paso-rojo-sigue-rojo`. Un hallazgo nuevo no puede reutilizar el id de un
+abierto. Los MENOR no pasan por el cierre del modelo: se cierran cuando su
+fichero cambia. `tc2 hallazgos abiertos` reparte los abiertos graves: cada
+uno a su revisora si sigue elegida y, si no, a la que tiene su fichero entre
+sus rutas (los ficheros de los abiertos también entran en las rutas de `tc2
+revisoras --ronda <n>`, con lo que S13 se aplica desde la ronda 2).
+
+`veredicto --ronda` cuenta `nuevos` (aceptados en esta ronda), `persisten`
+(abiertos heredados sin cierre) y `cerrados` (cerrados en esta ronda) y los
+deja en el título, el resumen, las anotaciones y el informe de la carpeta de
+la tarea. La ejecución de la evidencia (adenda F6-4 de F1-8) no entra en
+F6-6 (decisión 3 de TASK-052).
+
 ## Límites conocidos
 
 - Sin saldo en la API de Anthropic, `revision-ia` sale siempre en rojo
@@ -354,13 +423,22 @@ rechazados quedan solo como recuento y motivos del resumen.
   un rojo por hallazgos.
 - Si un `PATCH` de un lote falla a medias, el check ya está publicado con su
   conclusión y le faltan anotaciones; el job `publicar` sale en rojo.
-- La evidencia de cada hallazgo se valida pero no se ejecuta hasta F6-6.
+- La evidencia de cada hallazgo se valida pero no se ejecuta (fuera de
+  F6-6; decisión 3 de TASK-052). En el cierre solo se contrasta `paso_rojo`
+  con el informe F5-4.
+- Rondas (F6-6): sin saldo en la API la cadena real no pasa de la ronda 1
+  (`incompleta` no cuenta). Una revisora que reporta un abierto con otro id
+  infla `nuevos`; la anotación de un abierto que persiste conserva la línea
+  de la ronda en que se encontró, aunque el código se haya desplazado. Un
+  force-push que reescribe los commits del PR deja fuera el check anterior:
+  la ronda vuelve a ser la 1, completa. El coste de buscar la ronda anterior
+  es una llamada a la API por commit del PR hasta encontrarla (tope 100).
 - Que GitHub deje pasar las respuestas por `outputs` sin enmascarar ni
   truncar no se ha comprobado aún en un run real (E2E de TASK-050).
 - Las tablas de hallazgos escapan el Markdown del modelo; su informe en
   prosa, debajo, va tal cual (como en F6-3): la defensa es de F6-7.
-- Sin el detector de dominios (F4-6) ni rondas (F6-6): S10, S11 y S13 no se
-  aplican.
+- Sin el detector de dominios (F4-6): S10 y S11 no se aplican (S13 sí, con
+  la ronda de F6-6).
 - El diff y el informe son datos del autor del PR que llegan al modelo; la
   defensa contra inyección de instrucciones es de F6-7.
 - Coste: una llamada por revisora y ronda (2 o 3 en un PR Rust + Vue).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-052/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-052/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-052/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-menor-nota.md" ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-ronda2.md"
````

````
crates/tc2/tests/fixtures/hallazgos/respuesta-menor-nota.md | 22 ++++++++++++++++++++++
 crates/tc2/tests/fixtures/hallazgos/respuesta-ronda2.md     | 32 ++++++++++++++++++++++++++++++++
 2 files changed, 54 insertions(+)
````
