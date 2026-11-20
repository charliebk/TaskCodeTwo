# Peticion de revision — TASK-050 (ronda 1)

- Tarea: TASK-050 — F6-4 Busqueda de errores
- Rama revisada: feature/task-050-f6-4-busqueda-de-errores
- Rama base: develop
- Commit revisado (HEAD): 90640c05dc759042baa565655060c3224b753c6f
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-050 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-050/tarea.md
- tareas/03-en-revision/TASK-050/planificacion/
- .github/workflows/revision-ia.yml
- crates/tc2/esquemas/hallazgos.schema.json
- crates/tc2/tests/fixtures/hallazgos/diff.txt
- crates/tc2/tests/fixtures/hallazgos/respuesta-mixta.md
- crates/tc2/tests/fixtures/hallazgos/respuesta-prosa.md
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/revision-ia.md

## Commits a revisar (git log develop..HEAD)

````
90640c0 feat(TASK-050): revision-ia valida los hallazgos antes de publicar
96d665e feat(TASK-050): tc2 hallazgos valida la evidencia de cada hallazgo
936ffd3 chore(TASK-050): tarea en curso
````

## Diff de tu dominio (7 fichero(s) de 15; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index d5d7aee..3f23fa2 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -1,9 +1,12 @@
-# Workflow reutilizable del revisor IA en CI (TASK-049, F6-3; ADR F1-15
-# decisiones 1, 2 y 6, F1-8, F1-6 D5a y adenda F6-3 de F1-15).
+# Workflow reutilizable del revisor IA en CI (TASK-049, F6-3; TASK-050,
+# F6-4; ADR F1-15 decisiones 1, 2 y 6, F1-8, F1-6 D5a y adendas F6-3 de
+# F1-15 y F6-3 y F6-4 de F1-8).
 #
 # El enrutado no lo decide el modelo: `tc2 revisoras` elige las revisoras,
 # las reglas y el modelo con las tablas de F1-15 y el catalogo de la base.
-# Dos jobs, cada uno en su clase de F1-8:
+# Tampoco decide que hallazgos cuentan: `tc2 hallazgos validar` rechaza el
+# que no trae evidencia de su tipo anclada a una linea del diff (F6-4).
+# Cuatro jobs, cada uno en su clase de F1-8:
 #   - preparar (lee, sin secretos ni escritura: contents y actions): saca la
 #     base (nunca el head del PR), el plugin taskcode a un SHA fijado y el
 #     codigo de tc2 a la version de este workflow; trae el head como dato
@@ -12,16 +15,26 @@
 #     el perfil de la cache por arbol, del run que lo ejecuto: TASK-137)
 #     con tope de tamano; corre
 #     `tc2 revisoras` y deja un prompt por revisora (su SKILL.md, la seccion
-#     del informe y el diff de sus rutas, con tope) en el artefacto
-#     `revision-ia-prompts`. No ejecuta nada del PR: ni la suite ni su
-#     codigo.
-#   - publicar (privilegio por el entorno `revisor`, sin checkout): con
-#     ANTHROPIC_API_KEY llama a la API de Anthropic (Messages, curl) una vez
-#     por revisora con el modelo elegido, y con la App del revisor
-#     (TC2_APP_REVISOR_ID y TC2_APP_REVISOR_CLAVE) publica el check-run
-#     `revision-ia` en el head con conclusion `neutral` y el resumen
-#     (revisoras, reglas, modelo, avisos) (D4). El GITHUB_TOKEN no escribe
-#     nada. Sin la App, sale limpio con un aviso y no publica.
+#     del informe, el diff de sus rutas, con tope, y las instrucciones de
+#     `tc2 hallazgos instrucciones`) en el artefacto `revision-ia-prompts`,
+#     con el diff recortado que vio cada revisora (`diffs/<i>.txt`) y el
+#     informe. No ejecuta nada del PR: ni la suite ni su codigo.
+#   - consultar (privilegio por el entorno `revisor`, sin checkout, solo
+#     ANTHROPIC_API_KEY): llama a la API de Anthropic (Messages, curl) una
+#     vez por revisora con el modelo elegido y pasa cada respuesta, cortada
+#     a 64 KiB y en base64, por `outputs` (un hueco por revisora, hasta 6).
+#     Un job con privilegio no sube artefactos (lint).
+#   - validar (ejecuta: `contents` y `actions` de lectura, sin secretos):
+#     compila tc2 como preparar y pasa cada respuesta por `tc2 hallazgos
+#     validar` contra el diff que vio esa revisora; sube
+#     `revision-ia-hallazgos` (resumen y texto del check, JSON por revisora).
+#   - publicar (privilegio por el entorno `revisor`, sin checkout, solo la
+#     App del revisor: TC2_APP_REVISOR_ID y TC2_APP_REVISOR_CLAVE): lee
+#     `revision-ia-hallazgos` como dato y publica el check-run `revision-ia`
+#     en el head con conclusion `neutral` (D4; el rojo es de F6-5) y el
+#     resumen (seleccion, estado de cada revisora y la linea de `tc2
+#     hallazgos`). El GITHUB_TOKEN no escribe nada. Sin la App, sale limpio
+#     con un aviso y no publica.
 #
 # Uso desde el proyecto (fijado por SHA; workflow_run toma el llamador de la
 # rama por defecto, que un PR no puede cambiar):
@@ -36,7 +49,7 @@
 #       permissions: { contents: read, actions: read }
 #       uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
 #       # Unica excepcion a R9 de F1-8: sin heredar, los secretos del entorno
-#       # revisor no llegan a `publicar` (E2E de TASK-049).
+#       # revisor no llegan a `consultar` ni a `publicar` (E2E de TASK-049).
 #       secrets: inherit
 #       with:
 #         verificacion-run: ${{ github.event.workflow_run.id }}
@@ -110,6 +123,8 @@ jobs:
     permissions:
       contents: read
       actions: read
+    outputs:
+      tc2-sha: ${{ steps.version.outputs.sha }}
     steps:
       - name: Version de tc2
         id: version
@@ -265,7 +280,7 @@ jobs:
         run: |
           set -eu
           salida="$RUNNER_TEMP/revision-ia"
-          mkdir -p "$salida/prompts"
+          mkdir -p "$salida/prompts" "$salida/diffs"
           # Rutas literales separadas por NUL: sin -z git entrecomilla y escapa
           # lo no ASCII ("\303\251.rs") y la ruta no casaria con nada.
           git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "origin/$BASE...$HEAD_SHA" > "$salida/rutas.txt"
@@ -279,6 +294,8 @@ jobs:
             nombre="informe-verificacion-$PERFIL del run $de"
             if [ -n "${origen:-}" ]; then nombre="$nombre (verificacion reutilizada por la cache por arbol; origen $origen)"; fi
             informe=(--informe "$RUNNER_TEMP/informe.json" --seccion "$salida/seccion.md" --artefacto "$nombre")
+            # F6-4: validar contrasta `paso_rojo` con el mismo informe.
+            cp "$RUNNER_TEMP/informe.json" "$salida/informe.json"
           fi
           (cd base && tc2 revisoras --json --rutas "$salida/rutas.txt" --nul "${tarea[@]}" \
             --plugin "$PLUGIN_NOMBRE=$PLUGIN" "${informe[@]}" --detalle "$salida/detalle.json") > "$salida/seleccion.json"
@@ -303,6 +320,8 @@ jobs:
               bytes=$(wc -c < "$RUNNER_TEMP/diff-$i.txt")
             fi
             head -c "$TOPE" "$RUNNER_TEMP/diff-$i.txt" > "$RUNNER_TEMP/diff-$i-cortado.txt"
+            # F6-4: el diff tal como lo ve el modelo, para anclar sus hallazgos.
+            cp "$RUNNER_TEMP/diff-$i-cortado.txt" "$salida/diffs/$i.txt"
             largo=$(awk '{ while (match($0, /`+/)) { if (RLENGTH > m) m = RLENGTH; $0 = substr($0, RSTART + RLENGTH) } } END { print m + 0 }' "$RUNNER_TEMP/diff-$i-cortado.txt")
             if [ "$largo" -lt 3 ]; then largo=3; else largo=$((largo + 1)); fi
             cerca=$(printf '%*s' "$largo" '' | tr ' ' '`')
@@ -332,7 +351,8 @@ jobs:
               echo
               echo "## Lo que devuelves"
               echo
-              echo "Un informe en Markdown en espanol: hallazgos clasificados CRITICO, IMPORTANTE o MENOR, cada uno con fichero y linea del diff, el caso que lo rompe y el arreglo propuesto; si no hay hallazgos, dilo. Termina con una linea \`veredicto: aprobado\` o \`veredicto: cambios\`."
+              # F6-4: el mismo texto con el que se validan los hallazgos.
+              tc2 hallazgos instrucciones
             } > "$p"
             printf '%s\t%s\t%s\n' "$i" "$rid" "$modelo" >> "$salida/prompts/lista.tsv"
           done
@@ -345,57 +365,64 @@ jobs:
           if-no-files-found: error
           retention-days: 7
 
-  publicar:
-    name: revision-ia
+  consultar:
+    name: consultar
     needs: preparar
-    if: always()
+    if: needs.preparar.result == 'success'
     runs-on: ubuntu-latest
-    timeout-minutes: 30
+    timeout-minutes: 60
     environment: revisor
     permissions:
       actions: read
+    # Un hueco por revisora (r<i>: la respuesta en base64; e<i>: su estado).
+    # Un job con privilegio no sube artefactos: las respuestas van a validar
+    # por outputs (1 MiB por job): 6 huecos de 64 KiB de texto, en base64
+    # unos 87 KB cada uno, por debajo tambien de los 128 KiB que admite una
+    # variable de entorno en validar.
+    outputs:
+      r0: ${{ steps.modelo.outputs.r0 }}
+      r1: ${{ steps.modelo.outputs.r1 }}
+      r2: ${{ steps.modelo.outputs.r2 }}
+      r3: ${{ steps.modelo.outputs.r3 }}
+      r4: ${{ steps.modelo.outputs.r4 }}
+      r5: ${{ steps.modelo.outputs.r5 }}
+      e0: ${{ steps.modelo.outputs.e0 }}
+      e1: ${{ steps.modelo.outputs.e1 }}
+      e2: ${{ steps.modelo.outputs.e2 }}
+      e3: ${{ steps.modelo.outputs.e3 }}
+      e4: ${{ steps.modelo.outputs.e4 }}
+      e5: ${{ steps.modelo.outputs.e5 }}
     steps:
-      - name: Revisar con el modelo y publicar revision-ia
+      - name: Consultar al modelo
+        id: modelo
         env:
           GH_TOKEN: ${{ github.token }}
           REPO: ${{ github.repository }}
           RUN_ID: ${{ github.run_id }}
-          HEAD_SHA: ${{ inputs.head-sha }}
-          PREPARAR: ${{ needs.preparar.result }}
-          APP_ID: ${{ secrets.TC2_APP_REVISOR_ID }}
-          APP_CLAVE: ${{ secrets.TC2_APP_REVISOR_CLAVE }}
           ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
-          EJECUCION: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
         run: |
           set -euo pipefail
-          if [ -z "$APP_ID" ] || [ -z "$APP_CLAVE" ]; then
-            echo "::warning::revision-ia: el entorno revisor no tiene TC2_APP_REVISOR_ID y TC2_APP_REVISOR_CLAVE; no se publica el check"
-            exit 0
-          fi
           dir="$RUNNER_TEMP/revision-ia"
           mkdir -p "$dir"
-          resumen="preparar=$PREPARAR"
-          texto=""
-          prompts=no
-          if [ "$PREPARAR" = success ]; then
-            art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-prompts --jq '.artifacts[0].id // ""') || art=""
-            if [ -n "$art" ] && gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/prompts.zip" \
-              && unzip -q -o "$RUNNER_TEMP/prompts.zip" -d "$dir" && [ -s "$dir/resumen.txt" ] && [ -f "$dir/prompts/lista.tsv" ]; then
-              prompts=si
-            else
-              echo "::warning::revision-ia: no se encuentra o no se lee el artefacto revision-ia-prompts de preparar; el check sale neutral con sin-prompts"
-              resumen="preparar=$PREPARAR sin-prompts"
-            fi
+          art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-prompts --jq '.artifacts[0].id // ""') || art=""
+          if [ -z "$art" ] || ! gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/prompts.zip" \
+            || ! unzip -q -o "$RUNNER_TEMP/prompts.zip" -d "$dir" || [ ! -f "$dir/prompts/lista.tsv" ]; then
+            echo "::warning::revision-ia: no se lee el artefacto revision-ia-prompts; validar lo dira como sin-respuesta"
+            exit 0
           fi
-          if [ "$prompts" = si ]; then
-            resumen="$(cat "$dir/resumen.txt")"
-            while IFS="$(printf '\t')" read -r i rid modelo; do
-              [ -n "$i" ] || continue
-              if [ -z "$ANTHROPIC_API_KEY" ]; then
-                resumen="$resumen
-          $rid: sin-clave-del-modelo"
-                continue
-              fi
+          huecos=6
+          tope=65536
+          while IFS="$(printf '\t')" read -r i rid modelo; do
+            [ -n "$i" ] || continue
+            if [ "$i" -ge "$huecos" ]; then
+              echo "::warning::revision-ia: la revisora $rid pasa de los $huecos huecos de outputs; sale sin-respuesta"
+              continue
+            fi
+            texto="$RUNNER_TEMP/texto-$i.md"
+            : > "$texto"
+            if [ -z "$ANTHROPIC_API_KEY" ]; then
+              estado="sin-clave-del-modelo"
+            else
               # Messages API (anthropic-version 2026-10-21). Con sonnet y opus,
               # fallbacks "default" ante una negativa del clasificador; haiku
               # no tiene fallback del servidor.
@@ -421,15 +448,163 @@ jobs:
                 estado="negativa"
               else
                 estado="ok($parada) tokens=$(jq -r '"\(.usage.input_tokens)+\(.usage.output_tokens)"' "$respuesta")"
-                texto="$texto
-          ## $rid ($modelo)
+                jq -r '[.content[] | select(.type == "text") | .text] | join("\n")' "$respuesta" > "$RUNNER_TEMP/completo-$i.md"
+                bytes=$(wc -c < "$RUNNER_TEMP/completo-$i.md")
+                head -c "$tope" "$RUNNER_TEMP/completo-$i.md" > "$texto"
+                if [ "$bytes" -gt "$tope" ]; then
+                  echo "::warning::revision-ia: la respuesta de $rid ocupa $bytes bytes y se corta en $tope; lo que pase del corte no se valida"
+                  estado="$estado cortada=$bytes"
+                fi
+              fi
+            fi
+            printf 'r%s=%s\n' "$i" "$(base64 -w0 < "$texto")" >> "$GITHUB_OUTPUT"
+            printf 'e%s=%s\n' "$i" "$(printf '%s' "$estado" | tr -d '\r\n')" >> "$GITHUB_OUTPUT"
+            echo "$rid: $estado"
+          done < "$dir/prompts/lista.tsv"
 
-          $(jq -r '[.content[] | select(.type == "text") | .text] | join("\n")' "$respuesta")
-          "
+  validar:
+    name: validar
+    needs: [preparar, consultar]
+    if: always() && needs.preparar.result == 'success'
+    runs-on: ubuntu-latest
+    timeout-minutes: 30
+    permissions:
+      contents: read
+      actions: read
+    steps:
+      # El codigo de tc2 a la misma version que preparar (sin pasar binarios
+      # entre jobs: decision 4 de TASK-050).
+      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
+        with:
+          persist-credentials: false
+          repository: charliebk/TaskCodeTwo
+          ref: ${{ needs.preparar.outputs.tc2-sha }}
+          path: tc2-src
+      - name: Compilar tc2
+        working-directory: tc2-src
+        run: |
+          set -eu
+          rustup toolchain install
+          cargo build --locked -p tc2
+          echo "$PWD/target/debug" >> "$GITHUB_PATH"
+      # Las respuestas del modelo son dato: llegan por env (nunca en run:) y
+      # solo las lee tc2. Cada una se valida contra el diff que vio su
+      # revisora; la que falta o no se lee se valida vacia (sin-bloque).
+      - name: Validar hallazgos
+        env:
+          GH_TOKEN: ${{ github.token }}
+          REPO: ${{ github.repository }}
+          RUN_ID: ${{ github.run_id }}
+          CONSULTAR: ${{ needs.consultar.result }}
+          R0: ${{ needs.consultar.outputs.r0 }}
+          R1: ${{ needs.consultar.outputs.r1 }}
+          R2: ${{ needs.consultar.outputs.r2 }}
+          R3: ${{ needs.consultar.outputs.r3 }}
+          R4: ${{ needs.consultar.outputs.r4 }}
+          R5: ${{ needs.consultar.outputs.r5 }}
+          E0: ${{ needs.consultar.outputs.e0 }}
+          E1: ${{ needs.consultar.outputs.e1 }}
+          E2: ${{ needs.consultar.outputs.e2 }}
+          E3: ${{ needs.consultar.outputs.e3 }}
+          E4: ${{ needs.consultar.outputs.e4 }}
+          E5: ${{ needs.consultar.outputs.e5 }}
+        run: |
+          set -euo pipefail
+          dir="$RUNNER_TEMP/revision-ia"
+          out="$RUNNER_TEMP/revision-ia-hallazgos"
+          mkdir -p "$dir" "$out"
+          art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-prompts --jq '.artifacts[0].id // ""') || art=""
+          if [ -z "$art" ] || ! gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/prompts.zip" \
+            || ! unzip -q -o "$RUNNER_TEMP/prompts.zip" -d "$dir" || [ ! -s "$dir/resumen.txt" ] || [ ! -f "$dir/prompts/lista.tsv" ]; then
+            echo "::error::revision-ia: validar no lee el artefacto revision-ia-prompts de preparar" >&2
+            exit 1
+          fi
+          cp "$dir/resumen.txt" "$out/resumen.txt"
+          : > "$out/texto.md"
+          informe=()
+          if [ -s "$dir/informe.json" ]; then informe=(--informe "$dir/informe.json"); fi
+          while IFS="$(printf '\t')" read -r i rid modelo; do
+            [ -n "$i" ] || continue
+            r="R$i"
+            e="E$i"
+            estado="${!e:-sin-respuesta consultar=$CONSULTAR}"
+            respuesta="$RUNNER_TEMP/respuesta-$i.md"
+            : > "$respuesta"
+            if [ -n "${!r:-}" ] && ! printf '%s' "${!r}" | base64 -d > "$respuesta"; then
+              estado="$estado respuesta-ilegible"
+              : > "$respuesta"
+            fi
+            : > "$out/resumen-$i.md"
+            if ! linea=$(tc2 hallazgos validar --respuesta "$respuesta" --diff "$dir/diffs/$i.txt" --revisora "$rid" "${informe[@]}" \
+              --salida "$out/hallazgos-$i.json" --resumen "$out/resumen-$i.md"); then
+              linea="hallazgos error"
+            fi
+            printf '%s: %s; %s\n' "$rid" "$estado" "$linea" >> "$out/resumen.txt"
+            {
+              echo "## $rid ($modelo)"
+              echo
+              cat "$out/resumen-$i.md"
+              if [ -s "$respuesta" ]; then
+                echo "#### Informe del modelo"
+                echo
+                cat "$respuesta"
+                echo
               fi
-              resumen="$resumen
-          $rid: $estado"
-            done < "$dir/prompts/lista.tsv"
+            } >> "$out/texto.md"
+          done < "$dir/prompts/lista.tsv"
+          cat "$out/resumen.txt"
+      - name: Hallazgos para publicar
+        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
+        with:
+          name: revision-ia-hallazgos
+          path: ${{ runner.temp }}/revision-ia-hallazgos
+          if-no-files-found: error
+          retention-days: 7
+
+  publicar:
+    name: revision-ia
+    needs: [preparar, consultar, validar]
+    if: always()
+    runs-on: ubuntu-latest
+    timeout-minutes: 15
+    environment: revisor
+    permissions:
+      actions: read
+    steps:
+      - name: Publicar revision-ia
+        env:
+          GH_TOKEN: ${{ github.token }}
+          REPO: ${{ github.repository }}
+          RUN_ID: ${{ github.run_id }}
+          HEAD_SHA: ${{ inputs.head-sha }}
+          PREPARAR: ${{ needs.preparar.result }}
+          CONSULTAR: ${{ needs.consultar.result }}
+          VALIDAR: ${{ needs.validar.result }}
+          APP_ID: ${{ secrets.TC2_APP_REVISOR_ID }}
+          APP_CLAVE: ${{ secrets.TC2_APP_REVISOR_CLAVE }}
+          EJECUCION: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
+        run: |
+          set -euo pipefail
+          if [ -z "$APP_ID" ] || [ -z "$APP_CLAVE" ]; then
+            echo "::warning::revision-ia: el entorno revisor no tiene TC2_APP_REVISOR_ID y TC2_APP_REVISOR_CLAVE; no se publica el check"
+            exit 0
+          fi
+          dir="$RUNNER_TEMP/revision-ia-hallazgos"
+          mkdir -p "$dir"
+          resumen="$RUNNER_TEMP/resumen.txt"
+          texto="$RUNNER_TEMP/texto.md"
+          printf 'preparar=%s consultar=%s validar=%s\n' "$PREPARAR" "$CONSULTAR" "$VALIDAR" > "$resumen"
+          echo "Sin hallazgos validados." > "$texto"
+          # Lo escribe validar (tc2 de este workflow sobre la respuesta del
+          # modelo): se lee como dato, con tope, y no se ejecuta nada.
+          art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-hallazgos --jq '.artifacts[0].id // ""') || art=""
+          if [ -n "$art" ] && gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/hallazgos.zip" \
+            && unzip -q -o "$RUNNER_TEMP/hallazgos.zip" -d "$dir" && [ -s "$dir/resumen.txt" ]; then
+            cat "$dir/resumen.txt" >> "$resumen"
+            if [ -s "$dir/texto.md" ]; then cp "$dir/texto.md" "$texto"; fi
+          else
+            echo "::warning::revision-ia: no se encuentra o no se lee el artefacto revision-ia-hallazgos de validar; el check sale neutral con sin-hallazgos"
+            echo "sin-hallazgos" >> "$resumen"
           fi
           # Token de instalacion de la App del revisor (JWT RS256 con openssl;
           # la clave solo vive en un fichero temporal de este paso).
@@ -455,10 +630,11 @@ jobs:
             exit 1
           fi
           echo "::add-mask::$token"
-          # D4 de F6-3: neutral con el resumen; no bloquea hasta F6-5.
-          jq -n --arg sha "$HEAD_SHA" --arg s "$resumen" --arg t "${texto:-Sin informes del modelo.}" --arg url "$EJECUCION" \
+          # D4 de F6-3: neutral con el resumen; no bloquea hasta F6-5. Con
+          # --rawfile: el texto no pasa por argv (tope de 128 KiB por argumento).
+          jq -n --arg sha "$HEAD_SHA" --rawfile s "$resumen" --rawfile t "$texto" --arg url "$EJECUCION" \
             '{name: "revision-ia", head_sha: $sha, status: "completed", conclusion: "neutral", details_url: $url,
-              output: {title: "revision-ia: neutral (F6-3)", summary: ($s | .[0:65000]), text: ($t | .[0:65000])}}' > "$RUNNER_TEMP/check.json"
+              output: {title: "revision-ia: neutral (F6-4)", summary: ($s | .[0:65000]), text: ($t | .[0:65000])}}' > "$RUNNER_TEMP/check.json"
           api -X POST -H "Authorization: Bearer $token" "https://api.github.com/repos/$REPO/check-runs" --data-binary "@$RUNNER_TEMP/check.json" \
             | jq -r '"check-run \(.id) app_id=\(.app.id) \(.name) \(.conclusion)"'
-          printf '%s\n' "$resumen"
+          cat "$resumen"
diff --git a/crates/tc2/esquemas/hallazgos.schema.json b/crates/tc2/esquemas/hallazgos.schema.json
new file mode 100644
index 0000000..a3b9e5d
--- /dev/null
+++ b/crates/tc2/esquemas/hallazgos.schema.json
@@ -0,0 +1,145 @@
+{
+  "$schema": "https://json-schema.org/draft/2020-12/schema",
+  "title": "Hallazgos del revisor IA, version 1 (F6-4)",
+  "description": "Salida de `tc2 hallazgos validar --salida`: los hallazgos aceptados y los rechazados con su motivo, para una revisora. $defs/respuesta es lo que la revisora devuelve en su unico bloque json (lo valida hallazgos::validar). Reglas que el esquema no expresa y comprueba tc2: el id no se repite; fichero y linea estan en el lado nuevo del diff que vio la revisora; linea_original de un mutante es el texto de esa linea y la mutacion es distinta; esperado y obtenido de un borde son distintos; paso_rojo es un paso en rojo del informe de verificacion (F5-4). La evidencia se declara y se valida, no se ejecuta (F6-6). Una clave desconocida falla cerrado.",
+  "type": "object",
+  "additionalProperties": false,
+  "required": ["version", "revisora", "aceptados", "rechazados"],
+  "properties": {
+    "version": {
+      "description": "version del contrato; otra distinta de 1 es error",
+      "const": 1
+    },
+    "revisora": {
+      "description": "id de la revisora (tc2 revisoras)",
+      "type": "string",
+      "minLength": 1
+    },
+    "aceptados": {
+      "type": "array",
+      "items": { "$ref": "#/$defs/hallazgo" }
+    },
+    "rechazados": {
+      "type": "array",
+      "items": { "$ref": "#/$defs/rechazo" }
+    }
+  },
+  "$defs": {
+    "respuesta": {
+      "description": "contenido del unico bloque json de la respuesta de la revisora; vacia si no hay hallazgos",
+      "type": "array",
+      "maxItems": 100,
+      "items": { "$ref": "#/$defs/hallazgo" }
+    },
+    "hallazgo": {
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["id", "severidad", "fichero", "linea", "descripcion", "evidencia"],
+      "properties": {
+        "id": {
+          "description": "unico en la respuesta",
+          "type": "string",
+          "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$"
+        },
+        "severidad": {
+          "enum": ["CRITICO", "IMPORTANTE", "MENOR"]
+        },
+        "fichero": {
+          "description": "ruta de la cabecera +++ b/<ruta> del diff, sin b/",
+          "type": "string",
+          "minLength": 1
+        },
+        "linea": {
+          "description": "linea del lado nuevo del diff",
+          "type": "integer",
+          "minimum": 1
+        },
+        "descripcion": {
+          "type": "string",
+          "minLength": 1
+        },
+        "evidencia": { "$ref": "#/$defs/evidencia" }
+      }
+    },
+    "evidencia": {
+      "description": "obligatoria en todo hallazgo, tambien MENOR (decision de la persona, TASK-050)",
+      "oneOf": [
+        { "$ref": "#/$defs/reproduccion" },
+        { "$ref": "#/$defs/mutante" },
+        { "$ref": "#/$defs/borde" }
+      ]
+    },
+    "reproduccion": {
+      "description": "pasos y fallo esperado, o un paso en rojo del informe de verificacion",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["tipo"],
+      "anyOf": [
+        { "required": ["pasos", "fallo_esperado"] },
+        { "required": ["paso_rojo"] }
+      ],
+      "properties": {
+        "tipo": { "const": "reproduccion" },
+        "pasos": {
+          "type": "array",
+          "minItems": 1,
+          "items": { "type": "string", "minLength": 1 }
+        },
+        "fallo_esperado": { "type": "string", "minLength": 1 },
+        "paso_rojo": { "type": "string", "minLength": 1 }
+      }
+    },
+    "mutante": {
+      "description": "linea_original igual al texto de la linea citada, la mutacion y el test que deberia detectarla",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["tipo", "linea_original", "mutacion", "test"],
+      "properties": {
+        "tipo": { "const": "mutante" },
+        "linea_original": { "type": "string", "minLength": 1 },
+        "mutacion": { "type": "string", "minLength": 1 },
+        "test": { "type": "string", "minLength": 1 }
+      }
+    },
+    "borde": {
+      "description": "entrada, resultado esperado y resultado segun el codigo, distintos entre si",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["tipo", "entrada", "esperado", "obtenido"],
+      "properties": {
+        "tipo": { "const": "borde" },
+        "entrada": { "type": "string", "minLength": 1 },
+        "esperado": { "type": "string", "minLength": 1 },
+        "obtenido": { "type": "string", "minLength": 1 }
+      }
+    },
+    "rechazo": {
+      "description": "un hallazgo rechazado (indice en la lista) o toda la respuesta (indice null)",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["indice", "id", "motivo", "detalle"],
+      "properties": {
+        "indice": { "type": ["integer", "null"], "minimum": 0 },
+        "id": { "type": ["string", "null"] },
+        "motivo": {
+          "enum": [
+            "sin-bloque",
+            "bloque-repetido",
+            "json-invalido",
+            "fuera-de-tope",
+            "hallazgo-invalido",
+            "id-repetido",
+            "sin-evidencia",
+            "evidencia-incompleta",
+            "linea-fuera-del-diff",
+            "mutante-no-coincide",
+            "mutante-sin-cambio",
+            "borde-sin-diferencia",
+            "paso-rojo-no-esta"
+          ]
+        },
+        "detalle": { "type": "string" }
+      }
+    }
+  }
+}
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index b5ff11c..437f3d5 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -735,3 +735,45 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   `angular-vue-reviewer` y `rust-wasm-reviewer` (reglas S08, S12); el
   check `revision-ia` lo publica la App del revisor (5846307) con
   conclusión `neutral`.
+
+## Adenda (F6-4, TASK-050) — hallazgos con evidencia
+
+- **Decisión 3, revisada por la persona el 2026-11-20**: «F6-4 deja la
+  reproducción como artefacto» pasa a ser **«F6-4 exige y valida la
+  evidencia; F6-6 la ejecuta»**. El job `reproducir` (clase ejecuta, en
+  `pull_request`) que corre reproducciones, mutantes y bordes no se crea en
+  F6-4: sin la ronda siguiente (F6-6) no habría a quién darle su salida. Lo
+  que el modelo declara se valida como dato con reglas deterministas en
+  `tc2 hallazgos validar` (esquema `crates/tc2/esquemas/hallazgos.schema.json`,
+  versión 1): todo hallazgo, también MENOR, trae evidencia de su tipo
+  (`reproduccion`: pasos y fallo esperado, o un paso en rojo del informe
+  F5-4; `mutante`: `linea_original` igual al texto de esa línea en el diff,
+  la mutación y el test; `borde`: entrada, esperado y obtenido distintos)
+  anclada a una línea del lado nuevo del diff que vio esa revisora. Lo que
+  no cumple se rechaza con un motivo estable y el check lo lista (fila del
+  revisor IA de la decisión 4: «hallazgo sin evidencia → rechazado»).
+- **Cuatro jobs en `revision-ia.yml`**, cada uno en su clase y cada secreto
+  en un solo job: `preparar` (ejecuta, sin secretos) deja en el artefacto el
+  diff recortado de cada revisora; `consultar` (privilegio por el entorno
+  `revisor`, solo `ANTHROPIC_API_KEY`, sin checkout) llama al modelo;
+  `validar` (ejecuta, `contents` y `actions` de lectura, sin secretos)
+  compila `tc2` del mismo SHA que `preparar` (no se pasan binarios entre
+  jobs) y valida; `publicar` (privilegio, solo la App del revisor, sin
+  checkout) lee el artefacto `revision-ia-hallazgos` como dato y publica.
+  Ningún job con privilegio ejecuta un artefacto: la validación corre en
+  clase ejecuta, sobre código de TaskCodeTwo fijado por SHA. A3 (la clave
+  del modelo) ya no comparte job con la clave de la App (A2).
+- **Transporte de `consultar` a `validar`**: un job con privilegio no sube
+  artefactos (lint), así que las respuestas van por `outputs`, en base64, un
+  hueco por revisora (hasta 6) con 64 KiB de texto cada una. En `validar`
+  llegan por `env:` y solo las lee `tc2`; el test del workflow comprueba
+  que ningún `run:` interpola `needs.*`.
+- **Lo que queda**: la evidencia declarada no se ejecuta hasta F6-6 (lo
+  contienen el anclaje al diff, la coincidencia de `linea_original`, la
+  aprobación de la persona y F6-6); el check sigue `neutral` hasta F6-5; el
+  informe en prosa del modelo va al check tal cual (las tablas de hallazgos
+  sí escapan su Markdown) y la inyección de instrucciones es de F6-7.
+- **No verificado**: que GitHub deje pasar las respuestas por `outputs` de
+  un reutilizable sin enmascararlas ni truncarlas, y que la clave del modelo
+  y la App funcionen en dos jobs distintos del mismo entorno `revisor` con la
+  herencia actual (adenda F6-3): se comprueba en el E2E de TASK-050.
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index ced12f8..98d53e0 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -1,20 +1,24 @@
-# Revisor IA en CI (F6-3)
+# Revisor IA en CI (F6-3, F6-4)
 
-TASK-049. El check `revision-ia` lo publica la **App del revisor** desde un
+TASK-049 y TASK-050. El check `revision-ia` lo publica la **App del revisor** desde un
 workflow reutilizable que corre cuando termina `verificacion` en un PR
 ([`.github/workflows/revision-ia.yml`](../../.github/workflows/revision-ia.yml)).
 Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
 `tc2 revisoras` con las tablas del ADR
 [F1-15](../adr/F1-15-seleccion-de-skills-agentes-y-modelo.md) (adenda F6-3)
-y el catálogo de la base. Clases de jobs y secretos: ADR
-[F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md) (adenda F6-3).
+y el catálogo de la base. Qué hallazgos cuentan tampoco: `tc2 hallazgos
+validar` rechaza todo hallazgo sin evidencia de su tipo anclada a una línea
+del diff (F6-4, «Hallazgos con evidencia»). Clases de jobs y secretos: ADR
+[F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md) (adendas F6-3 y F6-4).
 
 ## Qué hace
 
 | Job | Clase (F1-8) | Permisos | Qué hace |
 |---|---|---|---|
-| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion`, o del run que ejecutó el perfil si ese run lo reutilizó de la caché (ver «Informe de una verificación reutilizada»), con tope de 1 MiB del zip y 64 KiB del JSON; saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` |
-| `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout | llama a la API de Anthropic una vez por revisora y publica el check-run `revision-ia` con la App del revisor, conclusión `neutral` |
+| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion`, o del run que ejecutó el perfil si ese run lo reutilizó de la caché (ver «Informe de una verificación reutilizada»), con tope de 1 MiB del zip y 64 KiB del JSON; saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` (prompts, el diff recortado que vio cada revisora en `diffs/<i>.txt` y el informe) |
+| `consultar` | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo `ANTHROPIC_API_KEY` | llama a la API de Anthropic una vez por revisora y pasa cada respuesta a `validar` por `outputs` (ver «Hallazgos con evidencia») |
+| `validar` | ejecuta | `contents: read`, `actions: read`; sin secretos | compila `tc2` de la misma versión que `preparar` (no se pasan binarios entre jobs), pasa cada respuesta por `tc2 hallazgos validar` contra el diff de su revisora y sube `revision-ia-hallazgos` |
+| `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo la App del revisor | lee `revision-ia-hallazgos` como dato y publica el check-run `revision-ia` con la App del revisor, conclusión `neutral` |
 
 El prompt de cada revisora lleva su `SKILL.md`, la línea de la selección,
 la sección del informe de verificación (`seccion_revisor` de F5-4, con la
@@ -24,16 +28,23 @@ El diff va entre cercas de acentos graves más largas que la secuencia más
 larga del propio diff (mínimo tres): ninguna línea del diff puede cerrar el
 bloque. La revisora de dominio recibe las rutas que casan con sus patrones; la
 genérica, las sueltas (S08) o todas (S09). Nadie ejecuta la suite: el
-modelo no tiene herramientas.
-
-El resumen del check dice `revisoras= agentes= modelo= modelo-api= reglas=
-avisos= carril= complejidad= informe=` (más `informe-de=run-<id>`, el run
-del que sale el informe, cuando hay informe) y una línea por revisora (`ok`,
-`negativa`, `error-http-<n>` o `sin-clave-del-modelo`); el texto, los
-informes del modelo. Si `publicar` no encuentra o no puede leer el artefacto
-`revision-ia-prompts`, lo avisa en el log y publica igual el check `neutral`
-con `preparar=<resultado> sin-prompts`. Un fallo de la App (no instalada,
-id o clave que no valen, sin token de instalación) se dice con un `::error::`
+modelo no tiene herramientas. La sección «Lo que devuelves» del prompt es la
+salida de `tc2 hallazgos instrucciones`, hecha con las mismas constantes que
+el validador.
+
+El resumen del check empieza por `preparar=<r> consultar=<r> validar=<r>`
+(el resultado de cada job), sigue con `revisoras= agentes= modelo=
+modelo-api= reglas= avisos= carril= complejidad= informe=` (más
+`informe-de=run-<id>`, el run del que sale el informe, cuando hay informe) y
+da una línea por revisora: `<id>: <estado>; hallazgos ok revisora=<id>
+aceptados=N rechazados=M motivos=<quien:motivo,...>`, con el estado de la
+llamada (`ok(<stop_reason>) tokens=...`, `negativa`, `error-http-<n> ...`,
+`sin-clave-del-modelo` o `sin-respuesta`; `cortada=<bytes>` si pasó de 64
+KiB). El texto lleva, por revisora, las tablas de aceptados y rechazados y
+después el informe del modelo. Si `publicar` no encuentra o no puede leer el
+artefacto `revision-ia-hallazgos`, lo avisa en el log y publica igual el
+check `neutral` con `sin-hallazgos`. Un fallo de la App (no instalada, id o
+clave que no valen, sin token de instalación) se dice con un `::error::`
 explícito y no publica.
 
 ### Credenciales del fetch
@@ -51,7 +62,7 @@ en ese paso. Así no aparece en la línea de órdenes, no se escribe en
 ## Cómo se invoca el modelo
 
 `curl` a la Messages API (`POST https://api.anthropic.com/v1/messages`) en
-el job `publicar`, con `x-api-key: $ANTHROPIC_API_KEY` y
+el job `consultar`, con `x-api-key: $ANTHROPIC_API_KEY` y
 `anthropic-version: 2026-10-21` (decisión D1 de TASK-049: clave de API como
 secreto del entorno, no Claude Code en el runner). Cuerpo:
 `{model, max_tokens: 16000, output_config: {effort: "high"}, messages}`,
@@ -82,12 +93,14 @@ leída en `taskcodetwo/tareas`; sin tarea, complejidad `media` con aviso.
    Las claves no pasan por el chat ni por ningún fichero del repo.
 3. **El llamador** en la rama por defecto, con `secrets: inherit`. Es la
    única excepción a la regla R9 de F1-8 (adenda F6-3): GitHub solo entrega
-   los secretos del entorno `revisor` al job `publicar` del reutilizable si
-   el llamador hereda. Declararlos en `on.workflow_call.secrets` no basta:
+   los secretos del entorno `revisor` a los jobs `consultar` y `publicar` del
+   reutilizable si el llamador hereda. Declararlos en `on.workflow_call.secrets` no basta:
    llegan vacíos (las dos cosas comprobadas en el E2E de TASK-049). El lint
    (perfil `llamador-revision`) solo admite `inherit` hacia
    `charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml` fijado por SHA,
-   y el del reutilizable impide usar secretos fuera de ese job. Ojo:
+   y el del reutilizable impide usar secretos fuera de esos jobs (y el test
+   del workflow, cada secreto fuera del suyo: la clave del modelo solo en
+   `consultar`, la App solo en `publicar`). Ojo:
    `inherit` entrega al reutilizable todos los secretos de repositorio y de
    organización, no solo los del entorno, y ese lint corre en los tests de
    TaskCodeTwo, no en tu repo: revisa el llamador a mano y fija el SHA:
@@ -124,7 +137,8 @@ commit (o del de la entrada `tc2-sha`).
 **Sin secretos sale limpio**: sin `TC2_APP_REVISOR_ID` o
 `TC2_APP_REVISOR_CLAVE`, `publicar` termina en verde con un aviso y no
 publica nada. Con la App y sin `ANTHROPIC_API_KEY`, publica el check
-`neutral` con la selección y `sin-clave-del-modelo` por revisora. Este repo
+`neutral` con la selección y, por revisora, `sin-clave-del-modelo` y su
+respuesta vacía rechazada (`respuesta:sin-bloque`). Este repo
 no activa aún el llamador: hasta que existan la App y el entorno, solo
 daría ejecuciones sin check.
 
@@ -162,7 +176,14 @@ hace en el repo E2E (rama por defecto `main`), no en TaskCodeTwo.
    - el resumen dice `revisoras=taskcode-plugin:angular-vue-reviewer,taskcode-plugin:rust-wasm-reviewer`,
      `reglas=` con `S08` y `S12`, el modelo y `informe=verde` (o `rojo`);
    - el artefacto `revision-ia-prompts` tiene un prompt por revisora con la
-     sección `## Informe de verificacion (F5-4)`;
+     sección `## Informe de verificacion (F5-4)` y las instrucciones de
+     `tc2 hallazgos instrucciones`, y `diffs/<i>.txt` por revisora;
+   - el resumen tiene, por revisora, la línea `hallazgos ok ... aceptados=N
+     rechazados=M motivos=...`; sin saldo en la cuenta del modelo, el estado
+     es `error-http-400 ...` y la respuesta vacía sale `respuesta:sin-bloque`
+     (F6-4: también así se publica el check que lo dice);
+   - el log de `validar` compila `tc2` y no ejecuta nada del PR, y el de
+     `consultar` avisa si una respuesta se cortó;
    - el log de `preparar` no ejecuta la suite ni nada del PR (solo `cargo
      build -p tc2` sobre el código de TaskCodeTwo).
 5. Repetir sin secretos de la App (o en un repo sin el entorno): `publicar`
@@ -188,9 +209,68 @@ El código del PR podría imitar esa línea en el log (la huella local corre
 antes en el mismo job), pero el informe ya lo escribe código del PR y solo se
 cita como dato: lo más que consigue es citar otro informe del mismo workflow.
 
+## Hallazgos con evidencia (F6-4)
+
+Cada revisora termina su respuesta con **un único** bloque ```` ```json ````
+con la lista de sus hallazgos (`[]` si no hay). Contrato:
+[`crates/tc2/esquemas/hallazgos.schema.json`](../../crates/tc2/esquemas/hallazgos.schema.json),
+versión 1 (`$defs/respuesta` es lo que va en el bloque). Cada hallazgo:
+
+| Clave | Qué es |
+|---|---|
+| `id` | único en la lista; letras, dígitos, `.`, `_`, `-`, hasta 64 |
+| `severidad` | `CRITICO`, `IMPORTANTE` o `MENOR` |
+| `fichero` | ruta de la cabecera `+++ b/<ruta>` del diff, sin `b/` |
+| `linea` | línea del **lado nuevo** del diff que vio esa revisora (contexto o añadida) |
+| `descripcion` | qué falla y por qué |
+| `evidencia` | obligatoria en **todos**, también MENOR (decisión de la persona) |
+
+Evidencia mínima por tipo (`tipo` y sus campos, llenos, ninguno de otro tipo):
+
+| `tipo` | Campos |
+|---|---|
+| `reproduccion` | `pasos` (lista no vacía) y `fallo_esperado`; o `paso_rojo`, un paso en rojo del informe F5-4 |
+| `mutante` | `linea_original` (igual al texto de esa línea en el diff, sin espacios a los lados), `mutacion` (distinta) y `test` (el que debería detectarla) |
+| `borde` | `entrada`, `esperado` y `obtenido`, con `esperado` distinto de `obtenido` |
+
+Motivos de rechazo (ids estables, en el orden en que se comprueban):
+`sin-bloque`, `bloque-repetido`, `json-invalido` (no se cierra, no es JSON o
+no es una lista), `fuera-de-tope` (más de 100 hallazgos),
+`hallazgo-invalido`, `id-repetido`, `sin-evidencia`, `evidencia-incompleta`,
+`linea-fuera-del-diff`, `mutante-no-coincide`, `mutante-sin-cambio`,
+`borde-sin-diferencia` y `paso-rojo-no-esta`. Que haya rechazos no es un
+fallo de la orden: sale `ok`, y qué hallazgos ponen el check en rojo lo
+decide F6-5.
+
+```
+tc2 hallazgos validar --respuesta <md> --diff <txt> --revisora <id>
+    [--informe <informe.json>] [--salida <hallazgos.json>] [--resumen <seccion.md>]
+tc2 hallazgos instrucciones
+```
+
+La evidencia **se declara y se valida, no se ejecuta**: la ejecución
+(decisión 3 de F1-8) pasa a F6-6 (adenda F6-4). Lo contienen el anclaje al
+diff que vio la revisora (lo que el tope dejó fuera se rechaza), que
+`linea_original` coincida con el texto de esa línea, la aprobación de la
+persona y F6-6.
+
+Transporte: `consultar` (privilegio) no puede subir artefactos (lint), así
+que cada respuesta va a `validar` por `outputs`, en base64 y en un hueco por
+revisora (`r0`..`r5`, con su estado en `e0`..`e5`). Con 64 KiB de texto por
+respuesta, cada hueco ocupa unos 87 KB: seis caben en el MiB de `outputs`
+de un job y cada uno en los 128 KiB de una variable de entorno. Una séptima
+revisora saldría `sin-respuesta` con un aviso (hoy la selección da como
+mucho tres de dominio o la genérica). En `validar` las respuestas entran por
+`env:`, nunca interpoladas en `run:` (lo comprueba el test del workflow).
+
 ## Límites conocidos
 
 - `neutral` no bloquea hasta F6-5; lo contiene la aprobación de la persona.
+- La evidencia de cada hallazgo se valida pero no se ejecuta hasta F6-6.
+- Que GitHub deje pasar las respuestas por `outputs` sin enmascarar ni
+  truncar no se ha comprobado aún en un run real (E2E de TASK-050).
+- Las tablas de hallazgos escapan el Markdown del modelo; su informe en
+  prosa, debajo, va tal cual (como en F6-3): la defensa es de F6-7.
 - Sin el detector de dominios (F4-6) ni rondas (F6-6): S10, S11 y S13 no se
   aplican.
 - El diff y el informe son datos del autor del PR que llegan al modelo; la
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-050/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-050/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-050/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/hallazgos/diff.txt" ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-mixta.md" ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-prosa.md"
````

````
crates/tc2/tests/fixtures/hallazgos/diff.txt           | 40 ++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/hallazgos/respuesta-mixta.md | 68 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/hallazgos/respuesta-prosa.md |  8 ++++++++
 3 files changed, 116 insertions(+)
````
