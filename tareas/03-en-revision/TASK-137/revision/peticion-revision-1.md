# Peticion de revision — TASK-137 (ronda 1)

- Tarea: TASK-137 — F6-3 seguimiento: revision-ia cita el informe del run de origen cuando la verificacion sale de cache
- Rama revisada: fix/task-137-f6-3-seguimiento-revision-ia-cita-el-inf
- Rama base: develop
- Commit revisado (HEAD): 92f28c9b8bf2191a5531bcf016dbbb47efe50a07
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-137 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-137/tarea.md
- tareas/03-en-revision/TASK-137/planificacion/
- .github/workflows/revision-ia.yml
- .github/workflows/verificacion.yml
- docs/ci/revision-ia.md

## Commits a revisar (git log develop..HEAD)

````
92f28c9 docs(TASK-137): resultado y evidencia del E2E
a6aafa5 fix(TASK-137): log del job con curl; gh nuevo no vuelca escapes ANSI
b33fe3a fix(TASK-137): revision-ia cita el informe del run de origen cuando la verificacion sale de cache
04c72fd chore(TASK-137): tarea en curso
````

## Diff completo (git diff develop..HEAD)

````diff
diff --git a/.github/workflows/revision-ia.yml b/.github/workflows/revision-ia.yml
index 0fad971..0c8294d 100644
--- a/.github/workflows/revision-ia.yml
+++ b/.github/workflows/revision-ia.yml
@@ -8,7 +8,9 @@
 #     base (nunca el head del PR), el plugin taskcode a un SHA fijado y el
 #     codigo de tc2 a la version de este workflow; trae el head como dato
 #     (`git fetch` de su SHA, sin checkout) para el diff; baja el informe de
-#     verificacion (F5-4) del run de `verificacion` con tope de tamano; corre
+#     verificacion (F5-4) del run de `verificacion` (o, si ese run reutilizo
+#     el perfil de la cache por arbol, del run que lo ejecuto: TASK-137)
+#     con tope de tamano; corre
 #     `tc2 revisoras` y deja un prompt por revisora (su SKILL.md, la seccion
 #     del informe y el diff de sus rutas, con tope) en el artefacto
 #     `revision-ia-prompts`. No ejecuta nada del PR: ni la suite ni su
@@ -189,12 +191,52 @@ jobs:
           PERFIL: ${{ inputs.perfil }}
         run: |
           set -eu
+          case "$PERFIL" in
+            ''|*[!a-z0-9-]*) echo "revision-ia: perfil no valido" >&2; exit 1 ;;
+          esac
           nombre="informe-verificacion-$PERFIL"
-          art=$(gh api --method GET "repos/$REPO/actions/runs/$VRUN/artifacts" -f name="$nombre" --jq '[.artifacts[] | select(.expired == false)] | sort_by(.created_at) | last | if . == null then "" else "\(.id) \(.size_in_bytes)" end') || art=""
+          artefacto() { gh api --method GET "repos/$REPO/actions/runs/$1/artifacts" -f name="$nombre" --jq '[.artifacts[] | select(.expired == false)] | sort_by(.created_at) | last | if . == null then "" else "\(.id) \(.size_in_bytes)" end'; }
+          art=$(artefacto "$VRUN") || art=""
+          de="$VRUN"
+          origen=""
           if [ -z "$art" ]; then
-            echo "sin artefacto $nombre en el run $VRUN"
-            exit 0
+            # TASK-137: con acierto de la cache por arbol (F5-2) el run no
+            # sube el informe; se sigue al run que ejecuto el perfil. El
+            # commit sale de la linea `Perfil <p>: reutilizada de <sha>
+            # (origen <sha>, ...)` del paso `cache por arbol` de
+            # verificacion.yml (si cambia, aqui se queda sin informe). El
+            # codigo del PR podria imitarla, pero el informe ya es suyo y
+            # solo se cita: como mucho se cita otro informe del mismo workflow.
+            job=$(gh api --method GET "repos/$REPO/actions/runs/$VRUN/jobs" -f per_page=100 --jq '[.jobs[] | select(.name == env.PERFIL or (.name | endswith(" / " + env.PERFIL)))] | first | .id // ""') || job=""
+            case "$job" in
+              ''|*[!0-9]*) job="" ;;
+              # Con curl y no con `gh api`: el log trae secuencias de escape
+              # ANSI y las versiones nuevas de gh se niegan a volcarlo sin
+              # --allow-escape-sequences, que las antiguas no conocen (E2E de
+              # TASK-137). La redireccion al log no recibe la cabecera.
+              *) origen=$(curl -fsSL -H "Authorization: Bearer $GH_TOKEN" -H "Accept: application/vnd.github+json" "$GITHUB_API_URL/repos/$REPO/actions/jobs/$job/logs" | tr -d '\r' | sed -nE "s/^[^ ]+ Perfil $PERFIL: reutilizada de [0-9a-f]{40} \(origen ([0-9a-f]{40}),.*/\1/p" | tail -n 1) || origen="" ;;
+            esac
+            if [ -z "$origen" ]; then
+              echo "sin artefacto $nombre en el run $VRUN ni linea de reutilizacion en el log de su job ${job:-(sin job del perfil)}"
+              exit 0
+            fi
+            wf=$(gh api --method GET "repos/$REPO/actions/runs/$VRUN" --jq '.workflow_id') || wf=""
+            case "$wf" in ''|*[!0-9]*) wf="" ;; esac
+            if [ -n "$wf" ]; then
+              # El mas reciente en verde del mismo workflow con el artefacto
+              # sin caducar: solo los runs que ejecutaron el perfil lo suben.
+              for r in $(gh api --method GET "repos/$REPO/actions/runs" -f head_sha="$origen" -f per_page=100 --jq "[.workflow_runs[] | select(.workflow_id == $wf and .conclusion == \"success\")] | sort_by(.created_at) | reverse | .[].id") ; do
+                art=$(artefacto "$r") || art=""
+                if [ -n "$art" ]; then de="$r"; break; fi
+              done
+            fi
+            if [ -z "$art" ]; then
+              echo "::warning::revision-ia: el run $VRUN reutiliza el perfil $PERFIL de $origen, pero no se encuentra $nombre en sus runs (caducado o sin informe): sin informe"
+              exit 0
+            fi
+            echo "el run $VRUN reutiliza el perfil $PERFIL de $origen: se cita $nombre del run $de"
           fi
+          printf '%s %s\n' "$de" "$origen" > "$RUNNER_TEMP/informe-de.txt"
           id=${art% *}
           bytes=${art#* }
           if [ "$bytes" -gt 1048576 ]; then
@@ -225,12 +267,17 @@ jobs:
           id=$(printf '%s' "$RAMA" | sed -nE 's#^(feature|fix|hotfix)/task-([0-9]{3,})-.*#TASK-\2#p')
           if [ -n "$id" ]; then tarea=(--tarea "$id"); fi
           informe=()
+          de=""
           if [ -s "$RUNNER_TEMP/informe.json" ]; then
-            informe=(--informe "$RUNNER_TEMP/informe.json" --seccion "$salida/seccion.md" --artefacto "informe-verificacion-$PERFIL")
+            read -r de origen < "$RUNNER_TEMP/informe-de.txt" || origen=""
+            nombre="informe-verificacion-$PERFIL del run $de"
+            if [ -n "${origen:-}" ]; then nombre="$nombre (verificacion reutilizada por la cache por arbol; origen $origen)"; fi
+            informe=(--informe "$RUNNER_TEMP/informe.json" --seccion "$salida/seccion.md" --artefacto "$nombre")
           fi
           (cd base && tc2 revisoras --json --rutas "$salida/rutas.txt" --nul "${tarea[@]}" \
             --plugin "$PLUGIN_NOMBRE=$PLUGIN" "${informe[@]}" --detalle "$salida/detalle.json") > "$salida/seleccion.json"
           jq -r '.claves | "revisoras=\(.revisoras | join(",")) agentes=\(.agentes | join(",")) modelo=\(.modelo) modelo-api=\(.["modelo-api"]) reglas=\(.reglas | join(",")) avisos=\(.avisos | join(",")) carril=\(.carril) complejidad=\(.complejidad) informe=\(.informe // "sin-informe")"' "$salida/seleccion.json" > "$salida/resumen.txt"
+          if [ -n "$de" ]; then sed -i "s/\$/ informe-de=run-$de/" "$salida/resumen.txt"; fi
           cat "$salida/resumen.txt"
           modelo=$(jq -r '.claves["modelo-api"]' "$salida/seleccion.json")
           : > "$salida/prompts/lista.tsv"
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index e01cce1..2bba303 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -310,6 +310,8 @@ jobs:
             echo "origen-$TC2_PERFIL=$origen"
             if [ -n "$de" ]; then echo "acierto=si"; else echo "acierto=no"; fi
           } >> "$GITHUB_OUTPUT"
+          # revision-ia.yml (TASK-137) lee esta linea del log para citar el
+          # informe del run de origen: si cambia su formato, cambiar alli.
           if [ -n "$de" ]; then
             echo "Perfil $TC2_PERFIL: reutilizada de $de (origen $origen, local $huella)"
           else
diff --git a/docs/ci/revision-ia.md b/docs/ci/revision-ia.md
index 096135b..ced12f8 100644
--- a/docs/ci/revision-ia.md
+++ b/docs/ci/revision-ia.md
@@ -13,7 +13,7 @@ y el catálogo de la base. Clases de jobs y secretos: ADR
 
 | Job | Clase (F1-8) | Permisos | Qué hace |
 |---|---|---|---|
-| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion` (tope de 1 MiB del zip y 64 KiB del JSON); saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` |
+| `preparar` | ejecuta | `contents: read`, `actions: read`; sin secretos | saca la base (`base`, por defecto `develop`), el plugin taskcode a un SHA fijado y el código de `tc2` a la versión del workflow, y compila `tc2`; trae el head del PR **como dato** (`git fetch` de su SHA, sin checkout, autenticado con el `GITHUB_TOKEN` del job; ver «Credenciales del fetch»); baja `informe-verificacion-<perfil>` del run de `verificacion`, o del run que ejecutó el perfil si ese run lo reutilizó de la caché (ver «Informe de una verificación reutilizada»), con tope de 1 MiB del zip y 64 KiB del JSON; saca las rutas con `git -c core.quotepath=false diff --name-only -z` y corre `tc2 revisoras --rutas <fichero> --nul` (rutas literales separadas por NUL, también las no ASCII), y escribe un prompt por revisora; sube el artefacto `revision-ia-prompts` |
 | `publicar` (`revision-ia`) | privilegio, por el entorno `revisor` | `actions: read`; sin checkout | llama a la API de Anthropic una vez por revisora y publica el check-run `revision-ia` con la App del revisor, conclusión `neutral` |
 
 El prompt de cada revisora lleva su `SKILL.md`, la línea de la selección,
@@ -27,7 +27,8 @@ genérica, las sueltas (S08) o todas (S09). Nadie ejecuta la suite: el
 modelo no tiene herramientas.
 
 El resumen del check dice `revisoras= agentes= modelo= modelo-api= reglas=
-avisos= carril= complejidad= informe=` y una línea por revisora (`ok`,
+avisos= carril= complejidad= informe=` (más `informe-de=run-<id>`, el run
+del que sale el informe, cuando hay informe) y una línea por revisora (`ok`,
 `negativa`, `error-http-<n>` o `sin-clave-del-modelo`); el texto, los
 informes del modelo. Si `publicar` no encuentra o no puede leer el artefacto
 `revision-ia-prompts`, lo avisa en el log y publica igual el check `neutral`
@@ -168,6 +169,25 @@ hace en el repo E2E (rama por defecto `main`), no en TaskCodeTwo.
    en verde con el aviso y sin check.
 6. Cerrar el PR y borrar la rama.
 
+## Informe de una verificación reutilizada
+
+Con acierto de la caché por árbol (F5-2), el run de `verificacion` se salta
+el comando y no sube `informe-verificacion-<perfil>`. Entonces `preparar`
+(TASK-137, con los mismos permisos):
+
+1. toma el job del perfil de ese run (`<perfil>` o `... / <perfil>`) y busca
+   en su log la última línea `Perfil <perfil>: reutilizada de <sha> (origen
+   <sha>, ...)` que escribe el paso `cache por arbol`;
+2. lista los runs en verde del mismo workflow con `head_sha` igual al
+   origen y cita el más reciente con el artefacto sin caducar (solo los runs
+   que ejecutaron el perfil lo suben);
+3. la sección del prompt nombra el artefacto con su run y su origen, y el
+   resumen añade `informe-de=run-<id>`.
+
+El código del PR podría imitar esa línea en el log (la huella local corre
+antes en el mismo job), pero el informe ya lo escribe código del PR y solo se
+cita como dato: lo más que consigue es citar otro informe del mismo workflow.
+
 ## Límites conocidos
 
 - `neutral` no bloquea hasta F6-5; lo contiene la aprobación de la persona.
@@ -176,10 +196,13 @@ hace en el repo E2E (rama por defecto `main`), no en TaskCodeTwo.
 - El diff y el informe son datos del autor del PR que llegan al modelo; la
   defensa contra inyección de instrucciones es de F6-7.
 - Coste: una llamada por revisora y ronda (2 o 3 en un PR Rust + Vue).
-- Si la verificación del head sale de la caché por árbol (F5-2), su run no
-  sube el informe y el check dice `informe=sin-informe`: las revisoras
-  trabajan sin la sección del informe. Comprobado en el E2E de TASK-049;
-  queda para una tarea de seguimiento (citar el informe del run de origen).
+- El informe de una verificación reutilizada se encuentra por la línea
+  `Perfil <p>: reutilizada de ... (origen <sha>, ...)` del log de
+  `verificacion.yml`: si cambia su formato, `preparar` se queda sin informe.
+  Si el artefacto del run de origen ha caducado (retención de artefactos del
+  repositorio, 90 días por defecto), el paso lo avisa con un `::warning::`,
+  el check dice `informe=sin-informe` y las revisoras van sin la sección
+  (TASK-137).
 - Un error de la API sale en el check con su tipo y mensaje recortados
   (por ejemplo, la cuenta sin saldo da `error-http-400
   invalid_request_error: Your credit balance is too low...`).
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-137/tarea.md                                                           | 40 ----------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-137/planificacion/brainstorm/peticion-unificador-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-137/planificacion/plan-final.md                       |  0
 tareas/02-en-curso/TASK-137/tarea.md                                                            | 52 ++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 52 insertions(+), 40 deletions(-)
````
