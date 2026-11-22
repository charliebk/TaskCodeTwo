# Peticion de revision — TASK-051 (ronda 1)

- Tarea: TASK-051 — F6-5 Hallazgos estructurados
- Rama revisada: feature/task-051-f6-5-hallazgos-estructurados
- Rama base: develop
- Commit revisado (HEAD): 02000f5a7c78e8404c5a4257bdb7eb7ecc6bfa1b
- Fecha: 2026-11-22
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-051 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-051/tarea.md
- tareas/03-en-revision/TASK-051/planificacion/
- .github/workflows/revision-ia.yml
- crates/tc2/esquemas/hallazgos.schema.json
- crates/tc2/tests/fixtures/hallazgos/respuesta-importante.md
- crates/tc2/tests/fixtures/hallazgos/respuesta-menor.md
- crates/tc2/tests/fixtures/hallazgos/respuesta-mixta.md
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
- docs/ci/revision-ia.md

## Commits a revisar (git log develop..HEAD)

````
02000f5 test(TASK-051): E2E de revision-ia con hallazgos en el repo de pruebas
051c157 feat(TASK-051): review guarda el informe de revision-ia en la tarea
9d30c10 feat(TASK-051): leer-revision en el puerto y en GitHub
62fdc8c feat(TASK-051): revision-ia publica el veredicto de tc2 y el lint lo exige
53456c0 feat(TASK-051): tc2 hallazgos veredicto con check.json y lotes
6f24681 feat(TASK-051): clase, esquema v2, veredicto y anotaciones puros
57e103f chore(TASK-051): tarea en curso
````

## Diff de tu dominio (9 fichero(s) de 22; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index 90ffc1c..8e42b14 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -1,11 +1,12 @@
 # Workflow reutilizable del revisor IA en CI (TASK-049, F6-3; TASK-050,
-# F6-4; ADR F1-15 decisiones 1, 2 y 6, F1-8, F1-6 D5a y adendas F6-3 de
-# F1-15 y F6-3 y F6-4 de F1-8).
+# F6-4; TASK-051, F6-5; ADR F1-15 decisiones 1, 2 y 6, F1-8, F1-6 D5a y
+# adendas F6-3 y F6-5 de F1-15 y F6-3 y F6-4 de F1-8).
 #
 # El enrutado no lo decide el modelo: `tc2 revisoras` elige las revisoras,
 # las reglas y el modelo con las tablas de F1-15 y el catalogo de la base.
 # Tampoco decide que hallazgos cuentan: `tc2 hallazgos validar` rechaza el
-# que no trae evidencia de su tipo anclada a una linea del diff (F6-4).
+# que no trae evidencia de su tipo anclada a una linea del diff (F6-4). Ni
+# el veredicto: `tc2 hallazgos veredicto` saca la conclusion (F6-5).
 # Cuatro jobs, cada uno en su clase de F1-8:
 #   - preparar (lee, sin secretos ni escritura: contents y actions): saca la
 #     base (nunca el head del PR), el plugin taskcode a un SHA fijado y el
@@ -26,15 +27,21 @@
 #     Un job con privilegio no sube artefactos (lint).
 #   - validar (ejecuta: `contents` y `actions` de lectura, sin secretos):
 #     compila tc2 como preparar y pasa cada respuesta por `tc2 hallazgos
-#     validar` contra el diff que vio esa revisora; sube
-#     `revision-ia-hallazgos` (resumen y texto del check, JSON por revisora).
+#     validar` contra el diff que vio esa revisora y junta todas con `tc2
+#     hallazgos veredicto` (check.json con la conclusion, el resumen y las
+#     anotaciones por linea, y los lotes anotaciones-<k>.json); sube
+#     `revision-ia-hallazgos` (eso, el texto del check y el JSON por
+#     revisora).
 #   - publicar (privilegio por el entorno `revisor`, sin checkout, solo la
 #     App del revisor: TC2_APP_REVISOR_ID y TC2_APP_REVISOR_CLAVE): lee
-#     `revision-ia-hallazgos` como dato y publica el check-run `revision-ia`
-#     en el head con conclusion `neutral` (D4; el rojo es de F6-5) y el
-#     resumen (seleccion, estado de cada revisora y la linea de `tc2
-#     hallazgos`). El GITHUB_TOKEN no escribe nada. Sin la App, sale limpio
-#     con un aviso y no publica.
+#     `revision-ia-hallazgos` como dato y publica tal cual el check.json de
+#     validar como check-run `revision-ia` en el head (conclusion `success`
+#     o `failure`: un CRITICO o IMPORTANTE aceptado, o una revisora sin
+#     respuesta valida, la deja en rojo; F6-5), y despues cada lote de
+#     anotaciones. Sin el artefacto, o con una conclusion que no es una de
+#     esas dos, publica `failure` con veredicto `incompleta`: no calcula
+#     nada. El GITHUB_TOKEN no escribe nada. Sin la App, sale limpio con un
+#     aviso y no publica.
 #
 # Uso desde el proyecto (fijado por SHA; workflow_run toma el llamador de la
 # rama por defecto, que un PR no puede cambiar):
@@ -534,6 +541,8 @@ jobs:
             exit 1
           fi
           cp "$dir/resumen.txt" "$out/resumen.txt"
+          printf 'preparar=success consultar=%s\n' "$CONSULTAR" >> "$out/resumen.txt"
+          veredicto=()
           : > "$out/texto.md"
           prosa="$RUNNER_TEMP/prosa.md"
           : > "$prosa"
@@ -556,6 +565,8 @@ jobs:
               linea="hallazgos error"
             fi
             printf '%s: %s; %s\n' "$rid" "$estado" "$linea" >> "$out/resumen.txt"
+            # Si validar no dejo su JSON, veredicto la cuenta como incompleta.
+            veredicto+=(--hallazgos "$out/hallazgos-$i.json")
             {
               echo "## $rid ($modelo)"
               echo
@@ -580,6 +591,8 @@ jobs:
               cat "$prosa"
             } >> "$out/texto.md"
           fi
+          # F6-5: la conclusion la saca tc2; publicar solo la transporta.
+          tc2 hallazgos veredicto --salida "$out" --extra "$out/resumen.txt" "${veredicto[@]}" | tee "$out/veredicto.txt"
           cat "$out/resumen.txt"
       - name: Hallazgos para publicar
         uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
@@ -619,21 +632,27 @@ jobs:
           fi
           dir="$RUNNER_TEMP/revision-ia-hallazgos"
           mkdir -p "$dir"
-          resumen="$RUNNER_TEMP/resumen.txt"
           texto="$RUNNER_TEMP/texto.md"
-          printf 'preparar=%s consultar=%s validar=%s\n' "$PREPARAR" "$CONSULTAR" "$VALIDAR" > "$resumen"
+          veredicto="$RUNNER_TEMP/veredicto.json"
           echo "Sin hallazgos validados." > "$texto"
           # Lo escribe validar (tc2 de este workflow sobre la respuesta del
-          # modelo): se lee como dato, con tope, y no se ejecuta nada.
+          # modelo): se lee como dato y no se ejecuta nada. La conclusion es
+          # la de check.json, si es una de las dos admitidas.
+          conclusion=""
           art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-hallazgos --jq '.artifacts[0].id // ""') || art=""
           if [ -n "$art" ] && gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/hallazgos.zip" \
-            && unzip -q -o "$RUNNER_TEMP/hallazgos.zip" -d "$dir" && [ -s "$dir/resumen.txt" ]; then
-            cat "$dir/resumen.txt" >> "$resumen"
+            && unzip -q -o "$RUNNER_TEMP/hallazgos.zip" -d "$dir" && [ -s "$dir/check.json" ]; then
+            conclusion=$(jq -r '.conclusion // ""' "$dir/check.json") || conclusion=""
             if [ -s "$dir/texto.md" ]; then cp "$dir/texto.md" "$texto"; fi
-          else
-            echo "::warning::revision-ia: no se encuentra o no se lee el artefacto revision-ia-hallazgos de validar; el check sale neutral con sin-hallazgos"
-            echo "sin-hallazgos" >> "$resumen"
           fi
+          case "$conclusion" in
+            success|failure) cp "$dir/check.json" "$veredicto" ;;
+            *)
+              echo "::warning::revision-ia: sin check.json legible de validar (preparar=$PREPARAR consultar=$CONSULTAR validar=$VALIDAR); se publica failure con veredicto incompleta"
+              printf 'preparar=%s consultar=%s validar=%s\nsin veredicto de validar: la revision no se completo\n' "$PREPARAR" "$CONSULTAR" "$VALIDAR" > "$RUNNER_TEMP/resumen.txt"
+              jq -n --rawfile s "$RUNNER_TEMP/resumen.txt" \
+                '{conclusion: "failure", output: {title: "revision-ia: incompleta (sin veredicto de validar)", summary: $s, annotations: []}}' > "$veredicto" ;;
+          esac
           # Token de instalacion de la App del revisor (JWT RS256 con openssl;
           # la clave solo vive en un fichero temporal de este paso).
           b64url() { openssl base64 -A | tr '+/' '-_' | tr -d '='; }
@@ -658,11 +677,24 @@ jobs:
             exit 1
           fi
           echo "::add-mask::$token"
-          # D4 de F6-3: neutral con el resumen; no bloquea hasta F6-5. Con
-          # --rawfile: el texto no pasa por argv (tope de 128 KiB por argumento).
-          jq -n --arg sha "$HEAD_SHA" --rawfile s "$resumen" --rawfile t "$texto" --arg url "$EJECUCION" \
-            '{name: "revision-ia", head_sha: $sha, status: "completed", conclusion: "neutral", details_url: $url,
-              output: {title: "revision-ia: neutral (F6-4)", summary: ($s | .[0:65000]), text: ($t | .[0:65000])}}' > "$RUNNER_TEMP/check.json"
-          api -X POST -H "Authorization: Bearer $token" "https://api.github.com/repos/$REPO/check-runs" --data-binary "@$RUNNER_TEMP/check.json" \
-            | jq -r '"check-run \(.id) app_id=\(.app.id) \(.name) \(.conclusion)"'
-          cat "$resumen"
+          # El check.json tal cual, con lo que solo sabe este job (nombre, SHA,
+          # enlace) y el texto; con --rawfile, nada pasa por argv (tope de
+          # 128 KiB por argumento).
+          jq --arg sha "$HEAD_SHA" --rawfile t "$texto" --arg url "$EJECUCION" \
+            '{name: "revision-ia", head_sha: $sha, status: "completed", details_url: $url, conclusion: .conclusion,
+              output: (.output | .summary |= .[0:65000] | .text = ($t | .[0:65000]))}' "$veredicto" > "$RUNNER_TEMP/check.json"
+          id=$(api -X POST -H "Authorization: Bearer $token" "https://api.github.com/repos/$REPO/check-runs" --data-binary "@$RUNNER_TEMP/check.json" \
+            | jq -r '"\(.id) app_id=\(.app.id) \(.name) \(.conclusion)"' | tee /dev/stderr | cut -d' ' -f1)
+          case "$id" in
+            ''|*[!0-9]*) echo "::error::revision-ia: la API no devolvio el id del check-run" >&2; exit 1 ;;
+          esac
+          # Las anotaciones que pasan de 50 van en PATCH, un lote por peticion
+          # (la API las anade a las que ya hay).
+          for k in $(seq 1 20); do
+            lote="$dir/anotaciones-$k.json"
+            [ -s "$lote" ] || break
+            jq --rawfile t "$texto" '.output |= (.summary |= .[0:65000] | .text = ($t | .[0:65000]))' "$lote" > "$RUNNER_TEMP/lote.json"
+            api -X PATCH -H "Authorization: Bearer $token" "https://api.github.com/repos/$REPO/check-runs/$id" --data-binary "@$RUNNER_TEMP/lote.json" \
+              | jq -r '"anotaciones=\(.output.annotations_count)"' | sed "s/^/lote $k: /"
+          done
+          jq -r '.output.title' "$veredicto"
diff --git a/crates/tc2/esquemas/hallazgos.schema.json b/crates/tc2/esquemas/hallazgos.schema.json
index a3b9e5d..1a6f8a6 100644
--- a/crates/tc2/esquemas/hallazgos.schema.json
+++ b/crates/tc2/esquemas/hallazgos.schema.json
@@ -1,14 +1,14 @@
 {
   "$schema": "https://json-schema.org/draft/2020-12/schema",
-  "title": "Hallazgos del revisor IA, version 1 (F6-4)",
-  "description": "Salida de `tc2 hallazgos validar --salida`: los hallazgos aceptados y los rechazados con su motivo, para una revisora. $defs/respuesta es lo que la revisora devuelve en su unico bloque json (lo valida hallazgos::validar). Reglas que el esquema no expresa y comprueba tc2: el id no se repite; fichero y linea estan en el lado nuevo del diff que vio la revisora; linea_original de un mutante es el texto de esa linea y la mutacion es distinta; esperado y obtenido de un borde son distintos; paso_rojo es un paso en rojo del informe de verificacion (F5-4). La evidencia se declara y se valida, no se ejecuta (F6-6). Una clave desconocida falla cerrado.",
+  "title": "Hallazgos del revisor IA, version 2 (F6-4 y F6-5)",
+  "description": "Salida de `tc2 hallazgos validar --salida`: los hallazgos aceptados y los rechazados con su motivo, para una revisora. $defs/respuesta es lo que la revisora devuelve en su unico bloque json (lo valida hallazgos::validar). Reglas que el esquema no expresa y comprueba tc2: el id no se repite; fichero y linea estan en el lado nuevo del diff que vio la revisora; linea_original de un mutante es el texto de esa linea y la mutacion es distinta; esperado y obtenido de un borde son distintos; paso_rojo es un paso en rojo del informe de verificacion (F5-4). La evidencia se declara y se valida, no se ejecuta (F6-6). Una clave desconocida falla cerrado. F6-5: cada hallazgo lleva su clase (lista cerrada de F1-15); $defs/veredicto es el check.json de `tc2 hallazgos veredicto` (conclusion success o failure, nunca neutral), $defs/lote cada anotaciones-<k>.json y $defs/informe-tarea el informe que `tc2 review` guarda en TASK-NNN/revision/.",
   "type": "object",
   "additionalProperties": false,
   "required": ["version", "revisora", "aceptados", "rechazados"],
   "properties": {
     "version": {
-      "description": "version del contrato; otra distinta de 1 es error",
-      "const": 1
+      "description": "version del contrato; otra distinta de 2 es error",
+      "const": 2
     },
     "revisora": {
       "description": "id de la revisora (tc2 revisoras)",
@@ -34,7 +34,7 @@
     "hallazgo": {
       "type": "object",
       "additionalProperties": false,
-      "required": ["id", "severidad", "fichero", "linea", "descripcion", "evidencia"],
+      "required": ["id", "severidad", "clase", "fichero", "linea", "descripcion", "evidencia"],
       "properties": {
         "id": {
           "description": "unico en la respuesta",
@@ -44,6 +44,10 @@
         "severidad": {
           "enum": ["CRITICO", "IMPORTANTE", "MENOR"]
         },
+        "clase": {
+          "description": "lista cerrada de F1-15 (decision 12); si encaja en varias, la primera",
+          "enum": ["puerta-seguridad", "contrato", "correccion", "fidelidad-evidencia", "reproducibilidad"]
+        },
         "fichero": {
           "description": "ruta de la cabecera +++ b/<ruta> del diff, sin b/",
           "type": "string",
@@ -113,6 +117,86 @@
         "obtenido": { "type": "string", "minLength": 1 }
       }
     },
+    "aceptado-de": {
+      "description": "un aceptado con su revisora: el raw_details de su anotacion",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["revisora", "hallazgo"],
+      "properties": {
+        "revisora": { "type": "string", "minLength": 1 },
+        "hallazgo": { "$ref": "#/$defs/hallazgo" }
+      }
+    },
+    "anotacion": {
+      "description": "anotacion del check-run: failure para CRITICO e IMPORTANTE, warning para MENOR; raw_details es un aceptado-de en JSON",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["path", "start_line", "end_line", "annotation_level", "title", "message", "raw_details"],
+      "properties": {
+        "path": { "type": "string", "minLength": 1 },
+        "start_line": { "type": "integer", "minimum": 1 },
+        "end_line": { "type": "integer", "minimum": 1 },
+        "annotation_level": { "enum": ["failure", "warning"] },
+        "title": { "type": "string", "minLength": 1, "maxLength": 255 },
+        "message": { "type": "string", "minLength": 1 },
+        "raw_details": { "type": "string", "minLength": 2 }
+      }
+    },
+    "salida-check": {
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["title", "summary", "annotations"],
+      "properties": {
+        "title": { "type": "string", "minLength": 1 },
+        "summary": { "type": "string", "minLength": 1 },
+        "annotations": {
+          "type": "array",
+          "maxItems": 50,
+          "items": { "$ref": "#/$defs/anotacion" }
+        }
+      }
+    },
+    "veredicto": {
+      "description": "check.json: la conclusion y la salida del check-run con las primeras 50 anotaciones; name y head_sha los pone el job publicar",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["conclusion", "output"],
+      "properties": {
+        "conclusion": { "enum": ["success", "failure"] },
+        "output": { "$ref": "#/$defs/salida-check" }
+      }
+    },
+    "lote": {
+      "description": "anotaciones-<k>.json: el cuerpo del PATCH con las anotaciones que siguen",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["output"],
+      "properties": {
+        "output": { "$ref": "#/$defs/salida-check" }
+      }
+    },
+    "informe-tarea": {
+      "description": "TASK-NNN/revision/revision-ia-<sha12>.json: lo leido del check publicado, sin la prosa del modelo",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["version", "tarea", "pr", "sha", "conclusion", "veredicto", "criticos", "importantes", "menores", "resumen", "aceptados"],
+      "properties": {
+        "version": { "const": 2 },
+        "tarea": { "type": "string", "minLength": 1 },
+        "pr": { "type": "integer", "minimum": 1 },
+        "sha": { "type": "string", "minLength": 40 },
+        "conclusion": { "enum": ["success", "failure"] },
+        "veredicto": { "enum": ["rojo", "verde", "incompleta"] },
+        "criticos": { "type": "integer", "minimum": 0 },
+        "importantes": { "type": "integer", "minimum": 0 },
+        "menores": { "type": "integer", "minimum": 0 },
+        "resumen": { "type": "string" },
+        "aceptados": {
+          "type": "array",
+          "items": { "$ref": "#/$defs/aceptado-de" }
+        }
+      }
+    },
     "rechazo": {
       "description": "un hallazgo rechazado (indice en la lista) o toda la respuesta (indice null)",
       "type": "object",
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index 2f693d6..45da892 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -568,3 +568,17 @@ no se ha configurado nada en ninguna plataforma):
 - **Las escrituras de la decisión 4** (`abrir-pr`, `merge-pr`,
   `aplicar-puertas`): solo por la documentación de la API.
 - **El límite de ficheros del diff de un MR en GitLab**.
+
+## Adenda (F6-5, TASK-051) — operación `leer-revision`
+
+La lista cerrada del puerto gana `leer-revision` (agente o persona; el
+revisor no), como el precedente `entorno-push` de F6-2: lee el check
+`revision-ia` del head del PR que publicó su origen (`app.id` =
+`plataforma.revisor_id`, el mismo filtro que `estado-puertas`), solo si ya
+terminó (si no, `esperando motivo=sin-check`), con su conclusión, título y
+resumen, y pagina `check-runs/{id}/annotations` para leer los aceptados de
+cada `raw_details`; uno que falta o no es un aceptado de `tc2 hallazgos
+veredicto` da `error motivo=respuesta-invalida`. Permiso: `checks:read` (y
+`pull_requests:read` para el head), que el agente ya tiene (F1-6). Por
+defecto, `rechazada motivo=no-soportada`: GitLab no tiene revisión en CI.
+Lo usa `tc2 review` para guardar el informe en la carpeta de la tarea.
diff --git a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
index 532c7e3..6bde44c 100644
--- a/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
+++ b/docs/adr/F1-15-seleccion-de-skills-agentes-y-modelo.md
@@ -850,3 +850,21 @@ fixtures que copian el bloque yaml de las 5 revisoras de la 0.16.2
 **No verificado (E2E de quien orquesta)**: el workflow en un PR real (App
 del revisor, clave del modelo, check `revision-ia` en `neutral` publicado por
 la App, el prompt con la sección del informe y sin ejecutar la suite).
+
+## Adenda (F6-5, TASK-051) — columna «Clase» y conclusión del check
+
+- **Clase**: la consecuencia «F6-5 añade la columna Clase» se cumple en el
+  contrato de hallazgos v2 (`crates/tc2/esquemas/hallazgos.schema.json`):
+  `clase` es obligatoria en cada hallazgo, con la lista cerrada de la
+  decisión 12 (`puerta-seguridad`, `contrato`, `correccion`,
+  `fidelidad-evidencia`, `reproducibilidad`; precedencia en ese orden). Si
+  falta o no es de la lista, el hallazgo se rechaza con `hallazgo-invalido`.
+  Las instrucciones del prompt salen de las mismas constantes
+  (`hallazgos::Clase`).
+- **Conclusión de `revision-ia`**: sustituye la D4 de la adenda F6-3
+  (`neutral`). `tc2 hallazgos veredicto` da `failure` si queda abierto un
+  CRITICO o IMPORTANTE aceptado (veredicto `rojo`) o si alguna revisora no
+  dejó respuesta válida o no hubo ninguna (`incompleta`), y `success` en el
+  resto (`verde`). Nunca `neutral`: GitHub lo da por cumplido en un check
+  obligatorio y una revisión que no ocurrió pasaría la puerta (RR-04). Guía:
+  `docs/ci/revision-ia.md`, «Veredicto del check».
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index cc93a3d..0d02c43 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -577,3 +577,8 @@ con su evidencia en los tests de `crates/tc2/tests/estado_*.rs`:
    rechaza una vez pasa a simularlo con un `receive-pack` que muere la
    primera vez.
 - Adenda (2026-11-18, TASK-127): `commit-estado` pasa las rutas de los blobs y las entradas del índice por stdin, no por argumentos: con 1000 ficheros la línea de órdenes pasaba del tope de Windows y `hash-object` fallaba. El puerto `Git` admite entrada por stdin solo en tres subcomandos: `cat-file` (F2-4b, solo lectura, con sus argumentos comprobados como los de cualquier llamada) y, en su forma exacta y ninguna otra, `hash-object -w --no-filters --stdin-paths` (`Git::ejecutar_con_entrada`) y `update-index --add -z --index-info` solo con índice temporal de ruta absoluta (`Git::ejecutar_con_indice_y_entrada`). `update-ref --stdin` sigue prohibido. Un commit de estado lanza así un número fijo de procesos sea cual sea el número de ficheros (test `mil_ficheros_en_una_transaccion_con_procesos_acotados` de `estado_casos.rs`); el registro de la capa deja de mostrar esas rutas.
+- Adenda (2026-11-21, TASK-051, F6-5): el puerto `Plataforma` gana la
+  operación `leer-revision` (detalle en la adenda F6-5 de F1-13) y la rama
+  de estado la operación `revision-guardar`, que publica
+  `TASK-NNN/revision/revision-ia-<sha12>.json` y `.md` con la tarea
+  `en-revision`; exige red y, con el mismo contenido, sale `sin-cambios`.
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index 98d53e0..2dc7157 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -1,6 +1,6 @@
-# Revisor IA en CI (F6-3, F6-4)
+# Revisor IA en CI (F6-3, F6-4, F6-5)
 
-TASK-049 y TASK-050. El check `revision-ia` lo publica la **App del revisor** desde un
+TASK-049, TASK-050 y TASK-051. El check `revision-ia` lo publica la **App del revisor** desde un
 workflow reutilizable que corre cuando termina `verificacion` en un PR
 ([`.github/workflows/revision-ia.yml`](../../.github/workflows/revision-ia.yml)).
 Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
@@ -8,7 +8,8 @@ Qué revisoras se lanzan y con qué modelo no lo decide el modelo: lo decide
 [F1-15](../adr/F1-15-seleccion-de-skills-agentes-y-modelo.md) (adenda F6-3)
 y el catálogo de la base. Qué hallazgos cuentan tampoco: `tc2 hallazgos
 validar` rechaza todo hallazgo sin evidencia de su tipo anclada a una línea
-del diff (F6-4, «Hallazgos con evidencia»). Clases de jobs y secretos: ADR
+del diff (F6-4, «Hallazgos con evidencia»). Ni el color del check: lo saca
+`tc2 hallazgos veredicto` (F6-5, «Veredicto del check»). Clases de jobs y secretos: ADR
 [F1-8](../adr/F1-8-seguridad-modelo-de-amenazas.md) (adendas F6-3 y F6-4).
 
 ## Qué hace
@@ -17,8 +18,8 @@ del diff (F6-4, «Hallazgos con evidencia»). Clases de jobs y secretos: ADR
 |---|---|---|---|
 | `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion`, o del run que ejecutó el perfil si ese run lo reutilizó de la caché (ver «Informe de una verificación reutilizada»), con tope de 1 MiB del zip y 64 KiB del JSON; saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` (prompts, el diff recortado que vio cada revisora en `diffs/<i>.txt` y el informe) |
 | `consultar` | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo `ANTHROPIC_API_KEY` | llama a la API de Anthropic una vez por revisora y pasa cada respuesta a `validar` por `outputs` (ver «Hallazgos con evidencia») |
-| `validar` | ejecuta | `contents: read`, `actions: read`; sin secretos | compila `tc2` de la misma versión que `preparar` (no se pasan binarios entre jobs), pasa cada respuesta por `tc2 hallazgos validar` contra el diff de su revisora y sube `revision-ia-hallazgos` |
-| `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo la App del revisor | lee `revision-ia-hallazgos` como dato y publica el check-run `revision-ia` con la App del revisor, conclusión `neutral` |
+| `validar` | ejecuta | `contents: read`, `actions: read`; sin secretos | compila `tc2` de la misma versión que `preparar` (no se pasan binarios entre jobs), pasa cada respuesta por `tc2 hallazgos validar` contra el diff de su revisora, las junta con `tc2 hallazgos veredicto` (`check.json` y lotes `anotaciones-<k>.json`) y sube `revision-ia-hallazgos` |
+| `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout; solo la App del revisor | lee `revision-ia-hallazgos` como dato y publica tal cual su `check.json` como check-run `revision-ia` con la App del revisor (conclusión `success` o `failure`), y después cada lote de anotaciones con un `PATCH`; sin `check.json` legible, `failure` con veredicto `incompleta` |
 
 El prompt de cada revisora lleva su `SKILL.md`, la línea de la selección,
 la sección del informe de verificación (`seccion_revisor` de F5-4, con la
@@ -32,8 +33,9 @@ modelo no tiene herramientas. La sección «Lo que devuelves» del prompt es la
 salida de `tc2 hallazgos instrucciones`, hecha con las mismas constantes que
 el validador.
 
-El resumen del check empieza por `preparar=<r> consultar=<r> validar=<r>`
-(el resultado de cada job), sigue con `revisoras= agentes= modelo=
+El resumen del check (F6-5) empieza por el veredicto, los recuentos y una
+fila por revisora (ver «Veredicto del check»); después, como líneas de
+texto plano, `preparar=success consultar=<r>`, `revisoras= agentes= modelo=
 modelo-api= reglas= avisos= carril= complejidad= informe=` (más
 `informe-de=run-<id>`, el run del que sale el informe, cuando hay informe) y
 da una línea por revisora: `<id>: <estado>; hallazgos ok revisora=<id>
@@ -42,8 +44,9 @@ llamada (`ok(<stop_reason>) tokens=...`, `negativa`, `error-http-<n> ...`,
 `sin-clave-del-modelo` o `sin-respuesta`; `cortada=<bytes>` si pasó de 64
 KiB). El texto lleva, por revisora, las tablas de aceptados y rechazados y
 después el informe del modelo. Si `publicar` no encuentra o no puede leer el
-artefacto `revision-ia-hallazgos`, lo avisa en el log y publica igual el
-check `neutral` con `sin-hallazgos`. Un fallo de la App (no instalada, id o
+artefacto `revision-ia-hallazgos` (o su `check.json`), lo avisa en el log y
+publica el check en `failure` con el título `revision-ia: incompleta (sin
+veredicto de validar)` y `preparar= consultar= validar=` en el resumen. Un fallo de la App (no instalada, id o
 clave que no valen, sin token de instalación) se dice con un `::error::`
 explícito y no publica.
 
@@ -137,7 +140,7 @@ commit (o del de la entrada `tc2-sha`).
 **Sin secretos sale limpio**: sin `TC2_APP_REVISOR_ID` o
 `TC2_APP_REVISOR_CLAVE`, `publicar` termina en verde con un aviso y no
 publica nada. Con la App y sin `ANTHROPIC_API_KEY`, publica el check
-`neutral` con la selección y, por revisora, `sin-clave-del-modelo` y su
+en `failure` (veredicto `incompleta`: no hubo revisión) con la selección y, por revisora, `sin-clave-del-modelo` y su
 respuesta vacía rechazada (`respuesta:sin-bloque`). Este repo
 no activa aún el llamador: hasta que existan la App y el entorno, solo
 daría ejecuciones sin check.
@@ -172,7 +175,8 @@ hace en el repo E2E (rama por defecto `main`), no en TaskCodeTwo.
 4. Esperar a `verificacion` y después a `revision-ia` (pestaña Actions).
    Comprobar:
    - el check-run `revision-ia` del head lo publica la App del revisor
-     (`app.id` = su id), conclusión `neutral`;
+     (`app.id` = su id), conclusión `success` o `failure` (sin saldo en la
+     cuenta del modelo, `failure` con título `revision-ia: incompleta ...`);
    - el resumen dice `revisoras=taskcode-plugin:angular-vue-reviewer,taskcode-plugin:rust-wasm-reviewer`,
      `reglas=` con `S08` y `S12`, el modelo y `informe=verde` (o `rojo`);
    - el artefacto `revision-ia-prompts` tiene un prompt por revisora con la
@@ -214,12 +218,13 @@ cita como dato: lo más que consigue es citar otro informe del mismo workflow.
 Cada revisora termina su respuesta con **un único** bloque ```` ```json ````
 con la lista de sus hallazgos (`[]` si no hay). Contrato:
 [`crates/tc2/esquemas/hallazgos.schema.json`](../../crates/tc2/esquemas/hallazgos.schema.json),
-versión 1 (`$defs/respuesta` es lo que va en el bloque). Cada hallazgo:
+versión 2 (`$defs/respuesta` es lo que va en el bloque). Cada hallazgo:
 
 | Clave | Qué es |
 |---|---|
 | `id` | único en la lista; letras, dígitos, `.`, `_`, `-`, hasta 64 |
 | `severidad` | `CRITICO`, `IMPORTANTE` o `MENOR` |
+| `clase` | F6-5: `puerta-seguridad`, `contrato`, `correccion`, `fidelidad-evidencia` o `reproducibilidad` (lista cerrada de F1-15; si encaja en varias, la primera); si falta o no es de la lista, `hallazgo-invalido` |
 | `fichero` | ruta de la cabecera `+++ b/<ruta>` del diff, sin `b/` |
 | `linea` | línea del **lado nuevo** del diff que vio esa revisora (contexto o añadida) |
 | `descripcion` | qué falla y por qué |
@@ -240,7 +245,7 @@ no es una lista), `fuera-de-tope` (más de 100 hallazgos),
 `linea-fuera-del-diff`, `mutante-no-coincide`, `mutante-sin-cambio`,
 `borde-sin-diferencia` y `paso-rojo-no-esta`. Que haya rechazos no es un
 fallo de la orden: sale `ok`, y qué hallazgos ponen el check en rojo lo
-decide F6-5.
+decide F6-5 (abajo).
 
 ```
 tc2 hallazgos validar --respuesta <md> --diff <txt> --revisora <id>
@@ -263,9 +268,77 @@ revisora saldría `sin-respuesta` con un aviso (hoy la selección da como
 mucho tres de dominio o la genérica). En `validar` las respuestas entran por
 `env:`, nunca interpoladas en `run:` (lo comprueba el test del workflow).
 
+## Veredicto del check (F6-5)
+
+`tc2 hallazgos veredicto --salida <dir> [--hallazgos <salida-de-validar.json>]... [--extra <lineas.txt>]`
+junta las salidas de `validar` de todas las revisoras (una que falta, no se
+lee o no es de la versión 2 cuenta como revisora sin respuesta válida) y
+decide, sin GitHub:
+
+| Veredicto | Cuándo | Conclusión |
+|---|---|---|
+| `incompleta` | no hay revisoras, o alguna no dejó JSON o su respuesta entera se rechazó (`sin-bloque`, `json-invalido`...) | `failure` |
+| `rojo` | algún CRITICO o IMPORTANTE **aceptado** (en F6-5 todo aceptado está abierto; el cierre llega con F6-6) | `failure` |
+| `verde` | el resto (solo MENOR, o nada) | `success` |
+
+Un CRITICO rechazado (sin evidencia, fuera del diff...) no pone el check en
+rojo. Nunca sale `neutral`: GitHub da por cumplido un check obligatorio en
+`neutral`, y una revisión que no ocurrió (sin saldo, o una respuesta sin
+bloque por una inyección) pasaría la puerta (decisión 1 de TASK-051, adenda
+F6-5 de F1-15).
+
+Escribe en `<dir>`:
+
+- `check.json` (`$defs/veredicto`): `conclusion`, `output.title`
+  (`revision-ia: <veredicto> (criticos=N importantes=M menores=K)`),
+  `output.summary` (veredicto, recuentos, tabla por revisora y las líneas de
+  `--extra` por `plano`) y las primeras 50 anotaciones;
+- `anotaciones-<k>.json` (`$defs/lote`), el cuerpo de cada `PATCH` con las
+  50 siguientes (la API admite 50 por petición).
+
+Cada aceptado es una **anotación** del check (se ve en «Files changed»; no
+hacen falta comentarios de PR ni permisos nuevos): `path` y `start_line` del
+hallazgo, nivel `failure` para CRITICO e IMPORTANTE y `warning` para MENOR,
+título `<id> <SEVERIDAD> <clase> (<revisora>)` y mensaje por `plano`, y en
+`raw_details` el aceptado con su revisora en JSON (`$defs/aceptado-de`).
+Línea: `hallazgos ok veredicto=<v> conclusion=<c> criticos=N importantes=M
+menores=K revisoras=R incompletas=I lotes=L`; un rojo no es un fallo de la
+orden.
+
+`publicar` no calcula nada: añade `name`, `head_sha`, `status`,
+`details_url` y `output.text`, y acepta la conclusión de `check.json` solo si
+es `success` o `failure`. El lint del workflow exige que `validar` corra
+`tc2 hallazgos veredicto`, que `publicar` lea la conclusión de
+`"$dir/check.json"`, que solo nombre `success` y `failure` y que `success`
+no aparezca como literal.
+
+### Informe en la carpeta de la tarea
+
+`tc2 review TASK-NNN`, cuando `revision-ia` ya terminó (`revision-ia=ok` o
+`fallo`), lee el check con la identidad del agente (`checks:read`; operación
+`leer-revision` del puerto, adenda F6-5 de F1-13) y guarda por el punto único
+de escritura de la rama de estado:
+
+- `TASK-NNN/revision/revision-ia-<sha12>.json` (`$defs/informe-tarea`):
+  tarea, PR, SHA, conclusión, veredicto, recuentos, el resumen del check y
+  los aceptados leídos de `raw_details`;
+- `TASK-NNN/revision/revision-ia-<sha12>.md`: el veredicto y la tabla de
+  aceptados, generados por `tc2` con `plano`, **sin** la prosa del modelo.
+
+La línea de `review` gana `hallazgos=<rojo|verde|incompleta|sin-leer>` e
+`informe=<ruta>`. Repetirla con el mismo check sale `sin-cambios`. Con el
+check pendiente no se lee ni se escribe nada: el informe se guarda al
+relanzar `review` cuando el check ya terminó. La CI no escribe en la rama de
+estado (necesitaría `contents: write`, que solo tiene el agente). Los
+rechazados quedan solo como recuento y motivos del resumen.
+
 ## Límites conocidos
 
-- `neutral` no bloquea hasta F6-5; lo contiene la aprobación de la persona.
+- Sin saldo en la API de Anthropic, `revision-ia` sale siempre en rojo
+  `incompleta` (es lo honesto: no hubo revisión); el título lo distingue de
+  un rojo por hallazgos.
+- Si un `PATCH` de un lote falla a medias, el check ya está publicado con su
+  conclusión y le faltan anotaciones; el job `publicar` sale en rojo.
 - La evidencia de cada hallazgo se valida pero no se ejecuta hasta F6-6.
 - Que GitHub deje pasar las respuestas por `outputs` sin enmascarar ni
   truncar no se ha comprobado aún en un run real (E2E de TASK-050).
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-051/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-051/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-051/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-importante.md" ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-menor.md" ":(literal)crates/tc2/tests/fixtures/hallazgos/respuesta-mixta.md"
````

````
crates/tc2/tests/fixtures/hallazgos/respuesta-importante.md | 31 +++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/hallazgos/respuesta-menor.md      | 21 +++++++++++++++++++++
 crates/tc2/tests/fixtures/hallazgos/respuesta-mixta.md      |  5 +++++
 3 files changed, 57 insertions(+)
````
