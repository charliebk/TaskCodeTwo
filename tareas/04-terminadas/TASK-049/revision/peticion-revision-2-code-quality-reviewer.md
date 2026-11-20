# Peticion de revision — TASK-049 (ronda 2)

- Tarea: TASK-049 — F6-3 Revisor en CI con las skills de dominio de la 0.16
- Rama revisada: feature/task-049-f6-3-revisor-en-ci-con-las-skills-de-dom
- Rama base: develop
- Commit revisado (HEAD): fc65d9e024c04921628c2087ca6622c7efaaade0
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-049 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 17ad62c057ca6f147c5df63ee7124444224f7693 (el commit revisado en la ronda anterior)
- Alcance: completa (motivo: un informe de la ronda anterior no tiene tabla de hallazgos legible)

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
informe de esta ronda (informe-revision-2-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-049/tarea.md
- tareas/03-en-revision/TASK-049/planificacion/
- .github/workflows/revision-ia.yml
- docs/adr/F1-8-seguridad-modelo-de-amenazas.md
- docs/ci/revision-ia.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.


Tabla de hallazgos ausente o ilegible en: informe-revision-1-code-quality-reviewer.md, informe-revision-1-rust-wasm-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD)

````
fc65d9e docs(TASK-049): limites del E2E real (informe con cache, errores de la API)
8e6eff3 fix(TASK-049): secrets inherit solo en el llamador de revision-ia
c294bc9 fix(TASK-049): secretos declarados en workflow_call y motivo del error de la API
f22cc95 chore(TASK-049): informes de la ronda 1 con los hallazgos corregidos
e399b23 fix(TASK-049): hallazgos de la ronda 1 de revision
609cca1 chore(TASK-049): peticion de revision ronda 1
````

## Diff de tu dominio (3 fichero(s) de 8; git diff 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index baff97e..7b95f22 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -33,6 +33,9 @@
 #       if: github.event.workflow_run.event == 'pull_request' && github.event.workflow_run.pull_requests[0] != null
 #       permissions: { contents: read, actions: read }
 #       uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
+#       # Unica excepcion a R9 de F1-8: sin heredar, los secretos del entorno
+#       # revisor no llegan a `publicar` (E2E de TASK-049).
+#       secrets: inherit
 #       with:
 #         verificacion-run: ${{ github.event.workflow_run.id }}
 #         head-sha: ${{ github.event.workflow_run.head_sha }}
@@ -153,15 +156,26 @@ jobs:
           rustup toolchain install
           cargo build --locked -p tc2
           echo "$PWD/target/debug" >> "$GITHUB_PATH"
+      # Los checkouts no persisten credenciales; en un repo privado el fetch
+      # las necesita. El GITHUB_TOKEN del job (solo lectura) va en la cabecera
+      # por variables de entorno de git (GIT_CONFIG_COUNT/KEY/VALUE): ni en
+      # argv, ni en .git/config, ni en el log (enmascarado).
       - name: Head del PR como dato
         env:
           HEAD_SHA: ${{ inputs.head-sha }}
           BASE: ${{ inputs.base }}
+          TOKEN: ${{ github.token }}
+          SERVIDOR: ${{ github.server_url }}
         run: |
           set -eu
           case "$HEAD_SHA" in
             *[!0-9a-f]*|'') echo "revision-ia: head-sha no valido" >&2; exit 1 ;;
           esac
+          auth=$(printf 'x-access-token:%s' "$TOKEN" | base64 -w0)
+          echo "::add-mask::$auth"
+          export GIT_CONFIG_COUNT=1
+          export GIT_CONFIG_KEY_0="http.$SERVIDOR/.extraheader"
+          export GIT_CONFIG_VALUE_0="AUTHORIZATION: basic $auth"
           git -C base fetch --no-tags -q origin "$HEAD_SHA"
           git -C base fetch --no-tags -q origin '+refs/heads/taskcodetwo/tareas:refs/remotes/origin/taskcodetwo/tareas' || echo "sin rama de estado: la seleccion va sin tarea"
           git -C base rev-parse -q --verify "origin/$BASE" > /dev/null
@@ -204,7 +218,9 @@ jobs:
           set -eu
           salida="$RUNNER_TEMP/revision-ia"
           mkdir -p "$salida/prompts"
-          git --literal-pathspecs -C base diff --name-only --no-renames "origin/$BASE...$HEAD_SHA" > "$salida/rutas.txt"
+          # Rutas literales separadas por NUL: sin -z git entrecomilla y escapa
+          # lo no ASCII ("\303\251.rs") y la ruta no casaria con nada.
+          git -c core.quotepath=false --literal-pathspecs -C base diff --name-only -z --no-renames "origin/$BASE...$HEAD_SHA" > "$salida/rutas.txt"
           tarea=()
           id=$(printf '%s' "$RAMA" | sed -nE 's#^(feature|fix|hotfix)/task-([0-9]{3,})-.*#TASK-\2#p')
           if [ -n "$id" ]; then tarea=(--tarea "$id"); fi
@@ -212,7 +228,7 @@ jobs:
           if [ -s "$RUNNER_TEMP/informe.json" ]; then
             informe=(--informe "$RUNNER_TEMP/informe.json" --seccion "$salida/seccion.md" --artefacto "informe-verificacion-$PERFIL")
           fi
-          (cd base && tc2 revisoras --json --rutas "$salida/rutas.txt" "${tarea[@]}" \
+          (cd base && tc2 revisoras --json --rutas "$salida/rutas.txt" --nul "${tarea[@]}" \
             --plugin "$PLUGIN_NOMBRE=$PLUGIN" "${informe[@]}" --detalle "$salida/detalle.json") > "$salida/seleccion.json"
           jq -r '.claves | "revisoras=\(.revisoras | join(",")) agentes=\(.agentes | join(",")) modelo=\(.modelo) modelo-api=\(.["modelo-api"]) reglas=\(.reglas | join(",")) avisos=\(.avisos | join(",")) carril=\(.carril) complejidad=\(.complejidad) informe=\(.informe // "sin-informe")"' "$salida/seleccion.json" > "$salida/resumen.txt"
           cat "$salida/resumen.txt"
@@ -222,7 +238,21 @@ jobs:
           for i in $(seq 0 $((n - 1))); do
             rid=$(jq -r ".revisoras[$i].id" "$salida/detalle.json")
             skill=$(jq -r ".revisoras[$i].skill // \"\"" "$salida/detalle.json")
-            mapfile -t rutas < <(jq -r ".revisoras[$i].rutas[]" "$salida/detalle.json")
+            # Tambien separadas por NUL, para que ninguna ruta se parta.
+            mapfile -d '' -t rutas < <(jq -j ".revisoras[$i].rutas[] | . + \"\\u0000\"" "$salida/detalle.json")
+            # El diff de sus rutas, cortado al tope, antes del prompt: la cerca
+            # que lo delimita tiene mas acentos graves que la secuencia mas
+            # larga del diff, asi que el diff no puede cerrarla.
+            bytes=0
+            : > "$RUNNER_TEMP/diff-$i.txt"
+            if [ "${#rutas[@]}" -gt 0 ]; then
+              git -c core.quotepath=false --literal-pathspecs -C base diff --no-renames "origin/$BASE...$HEAD_SHA" -- "${rutas[@]}" > "$RUNNER_TEMP/diff-$i.txt"
+              bytes=$(wc -c < "$RUNNER_TEMP/diff-$i.txt")
+            fi
+            head -c "$TOPE" "$RUNNER_TEMP/diff-$i.txt" > "$RUNNER_TEMP/diff-$i-cortado.txt"
+            largo=$(awk '{ while (match($0, /`+/)) { if (RLENGTH > m) m = RLENGTH; $0 = substr($0, RSTART + RLENGTH) } } END { print m + 0 }' "$RUNNER_TEMP/diff-$i-cortado.txt")
+            if [ "$largo" -lt 3 ]; then largo=3; else largo=$((largo + 1)); fi
+            cerca=$(printf '%*s' "$largo" '' | tr ' ' '`')
             p="$salida/prompts/$i.md"
             {
               echo "Eres la revisora \`$rid\` de TaskCode: revision por pares en CI (F6-3) del PR #$PR, commit $HEAD_SHA, contra $BASE."
@@ -238,15 +268,10 @@ jobs:
               echo
               echo "## Diff de las rutas de esta revisora"
               echo
-              echo '```diff'
-              bytes=0
-              if [ "${#rutas[@]}" -gt 0 ]; then
-                git --literal-pathspecs -C base diff --no-renames "origin/$BASE...$HEAD_SHA" -- "${rutas[@]}" > "$RUNNER_TEMP/diff-$i.txt"
-                bytes=$(wc -c < "$RUNNER_TEMP/diff-$i.txt")
-                head -c "$TOPE" "$RUNNER_TEMP/diff-$i.txt"
-              fi
+              echo "${cerca}diff"
+              cat "$RUNNER_TEMP/diff-$i-cortado.txt"
               echo
-              echo '```'
+              echo "$cerca"
               if [ "$bytes" -gt "$TOPE" ]; then
                 echo
                 echo "AVISO: el diff ocupa $bytes bytes y se ha cortado en $TOPE: dilo en tu informe y no opines de lo que no ves."
@@ -289,7 +314,7 @@ jobs:
           ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
           EJECUCION: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
         run: |
-          set -eu
+          set -euo pipefail
           if [ -z "$APP_ID" ] || [ -z "$APP_CLAVE" ]; then
             echo "::warning::revision-ia: el entorno revisor no tiene TC2_APP_REVISOR_ID y TC2_APP_REVISOR_CLAVE; no se publica el check"
             exit 0
@@ -298,10 +323,18 @@ jobs:
           mkdir -p "$dir"
           resumen="preparar=$PREPARAR"
           texto=""
+          prompts=no
           if [ "$PREPARAR" = success ]; then
-            art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-prompts --jq '.artifacts[0].id // ""')
-            gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/prompts.zip"
-            unzip -q -o "$RUNNER_TEMP/prompts.zip" -d "$dir"
+            art=$(gh api --method GET "repos/$REPO/actions/runs/$RUN_ID/artifacts" -f name=revision-ia-prompts --jq '.artifacts[0].id // ""') || art=""
+            if [ -n "$art" ] && gh api "repos/$REPO/actions/artifacts/$art/zip" > "$RUNNER_TEMP/prompts.zip" \
+              && unzip -q -o "$RUNNER_TEMP/prompts.zip" -d "$dir" && [ -s "$dir/resumen.txt" ] && [ -f "$dir/prompts/lista.tsv" ]; then
+              prompts=si
+            else
+              echo "::warning::revision-ia: no se encuentra o no se lee el artefacto revision-ia-prompts de preparar; el check sale neutral con sin-prompts"
+              resumen="preparar=$PREPARAR sin-prompts"
+            fi
+          fi
+          if [ "$prompts" = si ]; then
             resumen="$(cat "$dir/resumen.txt")"
             while IFS="$(printf '\t')" read -r i rid modelo; do
               [ -n "$i" ] || continue
@@ -326,7 +359,9 @@ jobs:
               codigo=$(curl -sS --max-time 900 -o "$respuesta" -w '%{http_code}' "${cabeceras[@]}" --data-binary "@$cuerpo" https://api.anthropic.com/v1/messages) || codigo=000
               parada=$(jq -r '.stop_reason // ""' "$respuesta" 2>/dev/null || echo "")
               if [ "$codigo" != 200 ]; then
-                estado="error-http-$codigo"
+                # El mensaje de error de la API (sin la clave), recortado y en una linea.
+                motivo=$(jq -r '"\(.error.type // ""): \(.error.message // "")"' "$respuesta" 2>/dev/null | tr -d '\r\n' | cut -c1-200 || true)
+                estado="error-http-$codigo ${motivo:-sin-cuerpo}"
               elif [ "$parada" = refusal ]; then
                 estado="negativa"
               else
@@ -354,9 +389,16 @@ jobs:
           rm -f "$clave"
           jwt="$cab.$carga.$firma"
           api() { curl -fsS -H "Accept: application/vnd.github+json" -H "X-GitHub-Api-Version: 2022-11-28" "$@"; }
-          instalacion=$(api -H "Authorization: Bearer $jwt" "https://api.github.com/repos/$REPO/installation" | jq -r .id)
+          instalacion=$(api -H "Authorization: Bearer $jwt" "https://api.github.com/repos/$REPO/installation" | jq -r '.id // ""') || instalacion=""
+          case "$instalacion" in
+            ''|*[!0-9]*) echo "::error::revision-ia: la App del revisor no esta instalada en $REPO o TC2_APP_REVISOR_ID/CLAVE no valen" >&2; exit 1 ;;
+          esac
           token=$(api -X POST -H "Authorization: Bearer $jwt" "https://api.github.com/app/installations/$instalacion/access_tokens" \
-            -d "$(jq -n --arg r "${REPO#*/}" '{repositories: [$r], permissions: {checks: "write"}}')" | jq -r .token)
+            -d "$(jq -n --arg r "${REPO#*/}" '{repositories: [$r], permissions: {checks: "write"}}')" | jq -r '.token // ""') || token=""
+          if [ -z "$token" ]; then
+            echo "::error::revision-ia: la App del revisor no da token de instalacion con checks: write en $REPO" >&2
+            exit 1
+          fi
           echo "::add-mask::$token"
           # D4 de F6-3: neutral con el resumen; no bloquea hasta F6-5.
           jq -n --arg sha "$HEAD_SHA" --arg s "$resumen" --arg t "${texto:-Sin informes del modelo.}" --arg url "$EJECUCION" \
diff --git a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
index 872a908..ff9f7da 100644
--- a/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
+++ b/docs/adr/F1-8-seguridad-modelo-de-amenazas.md
@@ -711,5 +711,21 @@ Los ADR aceptados no se modifican; lo que este ADR cambia queda aquí y en
   instrucciones, pero la inyección de instrucciones es de F6-7. El check
   sale `neutral` (D4) y no bloquea hasta F6-5: lo contiene la aprobación
   obligatoria de la persona.
-- **No verificado**: el E2E en un repo con la App del revisor y la clave
-  (lo ejecuta quien orquesta).
+- **Excepción a R9 (`secrets: inherit`), decidida por la persona el
+  2026-11-20**: el E2E real mostró que GitHub no da los secretos del
+  entorno `revisor` al job `publicar` del reutilizable si el llamador no
+  hereda: sin `secrets:` llegan vacíos, y también si el reutilizable los
+  declara en `on.workflow_call.secrets` como opcionales (run 44735329636);
+  con `secrets: inherit` llegan y la App publica el check (run
+  44733982088). Se admite `inherit` **solo** en el llamador de
+  `charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml` fijado por SHA
+  (perfil `llamador-revision` del lint, solo `workflow_run` y sin
+  escritura); en cualquier otro workflow sigue siendo un error. Lo que se
+  hereda llega a un workflow que fija el SHA y cuyo lint prohíbe secretos
+  fuera del job del entorno `revisor`; la alternativa (secretos de
+  repositorio pasados uno a uno) perdía la regla de rama del entorno.
+- **Verificado en el E2E** (`charliebk/taskcodetwo-e2e-github`, PR 14,
+  Rust + Vue): `preparar` sin secretos y sin ejecutar nada del PR elige
+  `angular-vue-reviewer` y `rust-wasm-reviewer` (reglas S08, S12); el
+  check `revision-ia` lo publica la App del revisor (5846307) con
+  conclusión `neutral`.
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index 83993d2..cad0a58 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -13,21 +13,39 @@ y el catálogo de la base. Clases de jobs y secretos: ADR
 
 | Job | Clase (F1-8) | Permisos | Qué hace |
 |---|---|---|---|
-| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout); baja `informe-verificacion-<perfil>` del run de `verificacion` (tope de 1 MiB del zip y 64 KiB del JSON); corre `tc2 revisoras` y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` |
+| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion` (tope de 1 MiB del zip y 64 KiB del JSON); saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` |
 | `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout | llama a la API de Anthropic una vez por revisora y publica el check-run `revision-ia` con la App del revisor, conclusión `neutral` |
 
 El prompt de cada revisora lleva su `SKILL.md`, la línea de la selección,
 la sección del informe de verificación (`seccion_revisor` de F5-4, con la
 regla de no relanzar lo que ya pasó en verde) y el diff de **sus** rutas con
 un tope (`tope-diff`, 120 000 bytes por defecto; si se corta, se le dice).
-La revisora de dominio recibe las rutas que casan con sus patrones; la
+El diff va entre cercas de acentos graves más largas que la secuencia más
+larga del propio diff (mínimo tres): ninguna línea del diff puede cerrar el
+bloque. La revisora de dominio recibe las rutas que casan con sus patrones; la
 genérica, las sueltas (S08) o todas (S09). Nadie ejecuta la suite: el
 modelo no tiene herramientas.
 
 El resumen del check dice `revisoras= agentes= modelo= modelo-api= reglas=
 avisos= carril= complejidad= informe=` y una línea por revisora (`ok`,
 `negativa`, `error-http-<n>` o `sin-clave-del-modelo`); el texto, los
-informes del modelo.
+informes del modelo. Si `publicar` no encuentra o no puede leer el artefacto
+`revision-ia-prompts`, lo avisa en el log y publica igual el check `neutral`
+con `preparar=<resultado> sin-prompts`. Un fallo de la App (no instalada,
+id o clave que no valen, sin token de instalación) se dice con un `::error::`
+explícito y no publica.
+
+### Credenciales del fetch
+
+Los checkouts llevan `persist-credentials: false` (lo exige el lint). El
+paso «Head del PR como dato» necesita credencial para `git fetch` del SHA
+del head y de `taskcodetwo/tareas` en un repo privado: usa el
+`GITHUB_TOKEN` del propio job (`contents: read`, solo lectura), que llega
+por `env:` y se pasa a git como cabecera `AUTHORIZATION: basic` con
+`GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_0`/`GIT_CONFIG_VALUE_0` exportadas solo
+en ese paso. Así no aparece en la línea de órdenes, no se escribe en
+`.git/config` (no queda para pasos siguientes) y su base64 se enmascara con
+`::add-mask::` antes de usarse.
 
 ## Cómo se invoca el modelo
 
@@ -61,8 +79,14 @@ leída en `taskcodetwo/tareas`; sin tarea, complejidad `media` con aviso.
    - `TC2_APP_REVISOR_CLAVE`: su clave privada PEM completa;
    - `ANTHROPIC_API_KEY`: la clave de la API de Anthropic.
    Las claves no pasan por el chat ni por ningún fichero del repo.
-3. **El llamador** en la rama por defecto (no hay que pasar `secrets:`: el
-   job del reutilizable lee los del entorno `revisor` del repo llamador):
+3. **El llamador** en la rama por defecto, con `secrets: inherit`. Es la
+   única excepción a la regla R9 de F1-8 (adenda F6-3): GitHub solo entrega
+   los secretos del entorno `revisor` al job `publicar` del reutilizable si
+   el llamador hereda. Declararlos en `on.workflow_call.secrets` no basta:
+   llegan vacíos (las dos cosas comprobadas en el E2E de TASK-049). El lint
+   (perfil `llamador-revision`) solo admite `inherit` hacia
+   `charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml` fijado por SHA,
+   y el del reutilizable impide usar secretos fuera de ese job:
 
 ```yaml
 name: revision-ia
@@ -79,6 +103,7 @@ jobs:
       contents: read
       actions: read
     uses: charliebk/TaskCodeTwo/.github/workflows/revision-ia.yml@<sha>
+    secrets: inherit
     with:
       verificacion-run: ${{ github.event.workflow_run.id }}
       head-sha: ${{ github.event.workflow_run.head_sha }}
@@ -148,3 +173,10 @@ hace en el repo E2E (rama por defecto `main`), no en TaskCodeTwo.
 - El diff y el informe son datos del autor del PR que llegan al modelo; la
   defensa contra inyección de instrucciones es de F6-7.
 - Coste: una llamada por revisora y ronda (2 o 3 en un PR Rust + Vue).
+- Si la verificación del head sale de la caché por árbol (F5-2), su run no
+  sube el informe y el check dice `informe=sin-informe`: las revisoras
+  trabajan sin la sección del informe. Comprobado en el E2E de TASK-049;
+  queda para una tarea de seguimiento (citar el informe del run de origen).
+- Un error de la API sale en el check con su tipo y mensaje recortados
+  (por ejemplo, la cuenta sin saldo da `error-http-400
+  invalid_request_error: Your credit balance is too low...`).
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 17ad62c057ca6f147c5df63ee7124444224f7693..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-049/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-049/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-049/revision/informe-revision-1-code-quality-reviewer.md         |   38 ++
 tareas/03-en-revision/TASK-049/revision/informe-revision-1-rust-wasm-reviewer.md            |   33 ++
 tareas/03-en-revision/TASK-049/revision/peticion-revision-1-code-quality-reviewer.md        |  772 ++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-049/revision/peticion-revision-1-rust-wasm-reviewer.md           | 2437 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-049/tarea.md                                    |    9 +-
 7 files changed, 3286 insertions(+), 3 deletions(-)
````
