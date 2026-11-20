# Peticion de revision — TASK-043 (ronda 1)

- Tarea: TASK-043 — F5-3 Verificacion por impacto como paso de CI y completa en la cola
- Rama revisada: feature/task-043-f5-3-verificacion-por-impacto-como-paso
- Rama base: develop
- Commit revisado (HEAD): 23039496d45451cb132c8fd041a42333355bf9a1
- Fecha: 2026-11-20
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-043 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-043/tarea.md
- tareas/03-en-revision/TASK-043/planificacion/
- .github/workflows/verificacion.yml
- docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
- docs/ci/verificacion-reutilizable.md
- plantillas/gitlab/verificacion.gitlab-ci.yml

## Commits a revisar (git log develop..HEAD)

````
2303949 chore(TASK-043): medicion E2E de tiempos con y sin cola de merge
8cbac85 feat(TASK-043): entrada cola-de-merge en el workflow y la plantilla de GitLab
a7d0c4a feat(TASK-043): adenda F5-3 a F1-7 y tabla de eventos con cola de merge
7515e44 chore(TASK-043): tarea en curso
````

## Diff de tu dominio (4 fichero(s) de 5; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index 2bba303..7788311 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -7,7 +7,9 @@
 #     perfiles del evento (F1-7, decision 4) y busca en los check-runs
 #     `verificacion` del commit verificado V, sus padres y sus abuelos las
 #     lineas `arbol=` candidatas de cada perfil (F1-7, decision 3; F5-2). No
-#     ejecuta nada del PR: solo lee el arbol con git.
+#     ejecuta nada del PR: solo lee el arbol con git. Con `cola-de-merge`
+#     (TASK-043, F5-3) el PR corre la impacto, que nunca se cachea, y el
+#     merge_group la completa.
 #   - ejecutar (ejecuta, matriz por perfil, runner del perfil, sin secretos):
 #     calcula la huella local, y si un candidato tiene la misma se salta el
 #     comando (paso `skipped`); si no, corre `comando argumentos-<perfil>` en
@@ -86,6 +88,10 @@ on:
         description: Orden que imprime lo que la suite lee fuera del arbol en el runner del perfil (F1-7, decision 2); vacia, sha256 de nada
         type: string
         default: ""
+      cola-de-merge:
+        description: El repo tiene cola de merge en la rama base; true, el PR corre la impacto y la cola la completa (adenda F5-3 de F1-7)
+        type: boolean
+        default: false
 
 permissions: {}
 
@@ -112,6 +118,7 @@ jobs:
           BASE: ${{ github.base_ref }}
           ORIGEN: ${{ github.head_ref }}
           FORZADOS: ${{ inputs.perfiles }}
+          COLA: ${{ inputs.cola-de-merge }}
         run: |
           set -eu
           if [ -n "$FORZADOS" ]; then
@@ -119,8 +126,12 @@ jobs:
           else
             case "$EVENTO" in
               pull_request)
+                # Adenda F5-3 de F1-7: con cola, la completa la corre el
+                # merge_group; la release sigue con completa y pesada.
                 if [ "$BASE" = main ] && [ "$ORIGEN" = develop ]; then
                   perfiles="completa pesada"
+                elif [ "$COLA" = true ]; then
+                  perfiles="impacto"
                 else
                   perfiles="completa"
                 fi ;;
@@ -138,7 +149,7 @@ jobs:
             lista="${lista:+$lista,}\"$p\""
           done
           echo "perfiles=[$lista]" >> "$GITHUB_OUTPUT"
-          echo "Perfiles de este evento ($EVENTO): [${lista}]"
+          echo "Perfiles de este evento ($EVENTO, cola-de-merge=$COLA): [${lista}]"
       # V, sus padres y sus abuelos con sus arboles: profundidad 3 basta.
       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
         if: steps.perfiles.outputs.perfiles != '[]'
@@ -174,7 +185,9 @@ jobs:
           huella=$(printf '%s' "$huella" | sha256sum | cut -d' ' -f1)
           claves='{}'
           runners='{}'
-          for p in $(printf '%s' "$PERFILES" | jq -r '.[]'); do
+          # La impacto depende de RAMA_BASE y no solo del arbol: ni clave, ni
+          # candidatos, ni linea `arbol=` (adenda F5-3 de F1-7, D3).
+          for p in $(printf '%s' "$PERFILES" | jq -r '.[] | select(. != "impacto")'); do
             r=$(printf '%s' "$ENTRADAS" | jq -r --arg p "$p" '.["runner-" + $p] | fromjson | if type == "array" then join(",") else tostring end' | tr -d ' ')
             # Las ordenes entran en la clave: un verde de otro comando,
             # argumentos, shell o huella local no se reutiliza.
@@ -189,6 +202,11 @@ jobs:
             echo "runners=$runners"
           } >> "$GITHUB_OUTPUT"
           echo "V=$v arbol=$arbol version=$VERSION claves=$claves runners=$runners"
+          if [ "$claves" = '{}' ]; then
+            echo "Ningun perfil cacheable en este evento: sin busqueda"
+            echo "candidatos={}" >> "$GITHUB_OUTPUT"
+            exit 0
+          fi
           if [ -z "$VERSION" ]; then
             echo "::warning::version del workflow desconocida: sin busqueda, se verifica entero"
             echo "candidatos={}" >> "$GITHUB_OUTPUT"
@@ -350,6 +368,7 @@ jobs:
           HEAD_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
           RESULTADO: ${{ needs.ejecutar.result }}
           PERFILES: ${{ needs.buscar.outputs.perfiles }}
+          COLA: ${{ inputs.cola-de-merge }}
           ARBOL_BUSCAR: ${{ needs.buscar.outputs.arbol }}
           CLAVES: ${{ needs.buscar.outputs.claves }}
           RUNNERS: ${{ needs.buscar.outputs.runners }}
@@ -364,7 +383,8 @@ jobs:
           for p in $(printf '%s' "$PERFILES" | tr -d '[]"' | tr ',' ' '); do
             informes="${informes:+$informes,}informe-verificacion-$p"
           done
-          resumen="perfiles=$PERFILES resultado=$RESULTADO informe=$informes ejecucion=$EJECUCION"
+          resumen="perfiles=$PERFILES resultado=$RESULTADO informe=$informes ejecucion=$EJECUCION
+          cola-de-merge=$COLA"
           # F1-8, decision 7: el arbol lo lee el publicador por la API; si no
           # es el de buscar, no se escribe ninguna linea y el check sale rojo.
           arbol=$(gh api --method GET "repos/$REPO/commits/$V" --jq '.commit.tree.sha')
@@ -373,6 +393,8 @@ jobs:
             conclusion=failure
           elif [ "$conclusion" = success ]; then
             for p in $(printf '%s' "$PERFILES" | tr -d '[]"' | tr ',' ' '); do
+              # La impacto nunca se cachea (adenda F5-3 de F1-7, D3).
+              if [ "$p" = impacto ]; then continue; fi
               campo() { printf '%s' "$SALIDAS" | jq -r --arg k "$1-$p" '.[$k] // ""'; }
               huella=$(campo local)
               de=$(campo de)
diff --git a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
index 7b2f7ab..47b92af 100644
--- a/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
+++ b/docs/adr/F1-7-verificacion-ci-perfiles-cache-runners.md
@@ -6,6 +6,7 @@
 - Depende de: [ADR F1-5](F1-5-puertas-en-github.md) (check `verificacion` obligatorio atado a GitHub Actions, `integration_id` 15368; `strict` en `develop` y no en `main`), [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (merge commit sin fast-forward, release por merge de `develop` en `main`), [ADR F1-1](F1-1-carpeta-fija-y-rama-de-estado.md) (el roadmap y las tareas fuera del árbol de código) y [ADR F1-6](F1-6-identidades-agente-revisor-persona.md) (el agente lee checks y no los escribe; no toca workflows)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a y D6a del plan de TASK-008)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
+- Adenda (2026-11-20, TASK-043): [impacto en el PR solo con cola de merge](#adenda-f5-3-task-043--impacto-en-el-pr-solo-con-cola-de-merge)
 
 ## Contexto
 
@@ -533,3 +534,48 @@ riesgos sin prueba:
 - **Seguridad del runner propio con código del PR**: F1-8.
 - **GitLab**: si la plantilla puede buscar el resultado en los pipelines de
   los padres como aquí en los check-runs (F1-13).
+
+## Adenda (F5-3, TASK-043) — impacto en el PR solo con cola de merge
+
+Decide Carlos el 2026-11-20 (D1 y D3 del plan de TASK-043, «las opciones
+recomendadas»). La consecuencia «F5-3 cambia de expectativa: la impacto no es
+el check de los PR» se mantiene **sin cola**; con cola, D1c pasa a admitida.
+
+- **Entrada explícita `cola-de-merge`** (booleana, `false` por defecto) en el
+  workflow reutilizable y en la plantilla de GitLab (D1). La decisión sale solo
+  del evento y de las entradas; no se consulta la API de rulesets para
+  detectar la cola.
+- **Con `true`**, la tabla de la decisión 4 gana estas filas (las demás no
+  cambian):
+
+  | Evento | Perfil | Dónde corre |
+  |---|---|---|
+  | PR a `develop` o de hotfix a `main`, con `cola-de-merge: true` | impacto, sin caché | runner de la impacto |
+  | `merge_group` (GitHub) o pipeline de `merge_train` (GitLab), con `cola-de-merge: true` | completa, desde la caché si acierta | runner de la completa |
+  | PR de release `develop`→`main`, con o sin cola | completa y pesada, como antes (D4a) | runner de cada perfil |
+
+- **Con `false`** todo queda como en D1a: completa en cada PR.
+- **Por qué D1c deja de valer con cola**: su objeción era que el resultado de
+  la impacto no se puede reutilizar en la release. Con cola, la release no
+  necesita la impacto: la completa corre en el `merge_group` sobre el árbol
+  que acaba en `develop`, y la release la encuentra en los padres de su
+  commit (decisión 3). Que el commit de `merge_group` acabe como cabeza de
+  `develop` con el mismo SHA: no verificado.
+- **La impacto nunca se cachea** (D3): depende de `RAMA_BASE` (decisión 1), no
+  solo del árbol. `buscar` no le calcula clave ni le busca candidatos, y el
+  publicador no le escribe línea `arbol=`; en GitLab, `after_script` tampoco
+  deja su fichero `cache-verificacion-impacto.txt`. Vale también con
+  `perfiles: impacto` forzado, que antes sí se cacheaba.
+- **El resumen del check** lleva una línea `cola-de-merge=<true|false>` debajo
+  de la de `perfiles=`; la búsqueda solo lee las líneas `arbol=`.
+- **Riesgo aceptado**: un llamador sin cola que ponga `cola-de-merge: true`
+  integraría en `develop` con solo la impacto. Lo contienen el valor por
+  defecto, el aviso de la guía y la línea del resumen; que `doctor` lo detecte
+  queda para F3-8.
+- **GitLab**: el MR corre `verificacion-impacto` y el pipeline del tren
+  (`CI_MERGE_REQUEST_EVENT_TYPE == "merge_train"`, nombre comprobado en la
+  documentación de variables predefinidas el 2026-11-20) corre
+  `verificacion-completa`. Los merge trains son de pago (Premium): esa ruta,
+  no verificado.
+- **Pregunta abierta**: la rápida se cachea por árbol aunque puede leer
+  `RAMA_BASE` (decisión 1). Ya pasaba antes de esta adenda; no se cambia aquí.
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index 00352a6..8ddbfd1 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -30,6 +30,7 @@ se elige el runner.
 | `perfiles` | `""` (los del evento) | Perfiles forzados, separados por espacios |
 | `huella-arbol` | `""` | Rutas versionadas que entran en la clave de la caché (lockfiles, toolchain), separadas por espacios (F5-2) |
 | `huella-local` | `""` | Orden que imprime lo que la suite lee fuera del árbol en el runner del perfil (F5-2); vacía, `local = sha256("")` |
+| `cola-de-merge` | `false` | Booleana. `true` solo si la rama base tiene cola de merge (GitHub) o merge trains (GitLab): el PR corre la impacto y la cola la completa (F5-3) |
 | GitHub: `runner-<perfil>` | `'"ubuntu-latest"'` | `runs-on` del perfil, en JSON: una cadena o una lista de etiquetas (`'["self-hosted","windows","toolsaweb"]'`) |
 | GitHub: `shell` | `bash` | Shell en el que corre el comando (`cmd` para un `.bat`) |
 | GitLab: `tags-<perfil>` | `[]` | Etiquetas del runner del perfil |
@@ -47,14 +48,27 @@ se elige el runner.
 | Evento | GitHub | GitLab |
 |---|---|---|
 | PR / MR a cualquier rama | completa | completa (`verificacion-completa`) |
-| PR / MR de `develop` a `main` (release) | completa y pesada | completa y pesada |
-| `merge_group` | completa | — |
+| PR / MR a cualquier rama, con `cola-de-merge: true` | impacto | impacto (`verificacion-impacto`) |
+| PR / MR de `develop` a `main` (release), con o sin cola | completa y pesada | completa y pesada |
+| `merge_group` / pipeline de `merge_train` | completa | completa con `cola-de-merge: true`; sin ella, la del MR (completa) |
 | `workflow_dispatch` | completa | — |
 | `push` de rama | nada | nada |
 
 El comando recibe la base del PR en `RAMA_BASE` y el perfil en `TC2_PERFIL`.
 Resultado: código de salida (0 verde, otro rojo).
 
+**Cola de merge** (TASK-043, F5-3; [adenda de F1-7](../adr/F1-7-verificacion-ci-perfiles-cache-runners.md#adenda-f5-3-task-043--impacto-en-el-pr-solo-con-cola-de-merge)):
+con `cola-de-merge: true` el PR corre solo la impacto y la completa la corre
+la cola sobre el árbol que acaba en `develop`. La impacto **nunca se cachea**
+(depende de `RAMA_BASE`): no deja línea `arbol=` y se ejecuta siempre. El
+resumen del check lleva `cola-de-merge=<true|false>`.
+
+> **Aviso:** pon `cola-de-merge: true` solo si la rama base tiene de verdad
+> una regla de cola (GitHub) o merge trains (GitLab). Sin cola, `develop`
+> integraría con solo la impacto, sin la completa. Nada lo detecta todavía
+> (F3-8); el check lo deja escrito en `cola-de-merge=`. En GitLab el tren
+> (`CI_MERGE_REQUEST_EVENT_TYPE == "merge_train"`) es de pago: no verificado.
+
 ## Qué publica
 
 - **GitHub**: tres jobs. `buscar` (Linux, `contents: read`) elige los perfiles;
@@ -225,7 +239,8 @@ F1-8, decisión 7), sin componentes nuevos: los tres jobs de siempre.
   verde añade al resumen una línea por perfil.
 
 **La línea** (contrato para F8-1), una por perfil, debajo de la de siempre
-(`perfiles= resultado= informe= ejecucion=`):
+(`perfiles= resultado= informe= ejecucion=`) y de `cola-de-merge=` (F5-3);
+la impacto no tiene línea:
 
 ```
 arbol=<árbol de V> perfil=<p> runner=<r> clave=<k> local=<h> origen=<sha> ejecutada
diff --git a/plantillas/gitlab/verificacion.gitlab-ci.yml b/plantillas/gitlab/verificacion.gitlab-ci.yml
index af0b200..c350d86 100644
--- a/plantillas/gitlab/verificacion.gitlab-ci.yml
+++ b/plantillas/gitlab/verificacion.gitlab-ci.yml
@@ -4,7 +4,9 @@
 # solo llama al comando de verificacion del proyecto. Un job por perfil
 # (verificacion-<perfil>); el perfil sale del evento (F1-7, decision 4):
 #   - pipeline de MR: completa; MR de develop a main: completa y pesada;
-#   - push de rama: nada, salvo los perfiles forzados con `perfiles`.
+#   - push de rama: nada, salvo los perfiles forzados con `perfiles`;
+#   - con `cola-de-merge: true` (TASK-043, F5-3): el MR que no es de release
+#     corre la impacto, y el pipeline del merge train la completa.
 #
 # Uso desde el .gitlab-ci.yml del proyecto (fijado por SHA, nunca por rama):
 #   include:
@@ -61,6 +63,10 @@ spec:
     huella-local:
       description: Orden que imprime lo que la suite lee fuera del arbol en el runner del perfil
       default: ""
+    cola-de-merge:
+      description: El proyecto usa merge trains; true, el MR corre la impacto y el tren la completa (adenda F5-3 de F1-7)
+      type: boolean
+      default: false
     stage:
       description: Etapa de los jobs de verificacion
       default: test
@@ -69,6 +75,7 @@ spec:
 # `variables:` de los jobs.
 variables:
   TC2_PERFILES: " $[[ inputs.perfiles ]] "
+  TC2_COLA_DE_MERGE: " $[[ inputs.cola-de-merge ]] "
 
 .tc2-verificacion:
   stage: $[[ inputs.stage ]]
@@ -83,9 +90,10 @@ variables:
   # after_script (sh), que no cambia el estado del job: en un runner sin sh
   # no hay linea, pero el resultado del perfil es el mismo. `runner` son las
   # etiquetas del runner; la version del workflow va vacia (GitLab no la da).
+  # La impacto nunca deja linea (adenda F5-3 de F1-7, D3).
   after_script:
     - |
-      if [ "$CI_JOB_STATUS" = success ]; then
+      if [ "$CI_JOB_STATUS" = success ] && [ "$TC2_PERFIL" != impacto ]; then
         mkdir -p .tc2
         arbol=$(git rev-parse 'HEAD^{tree}')
         huella=""
@@ -121,6 +129,7 @@ verificacion-impacto:
     TC2_PERFIL: impacto
   rules:
     - if: $TC2_PERFILES =~ / impacto /
+    - if: $CI_PIPELINE_SOURCE == "merge_request_event" && $TC2_COLA_DE_MERGE =~ / true / && $CI_MERGE_REQUEST_EVENT_TYPE != "merge_train" && ($CI_MERGE_REQUEST_TARGET_BRANCH_NAME != "main" || $CI_MERGE_REQUEST_SOURCE_BRANCH_NAME != "develop")
   script:
     - $[[ inputs.comando ]] $[[ inputs.argumentos-impacto ]]
 
@@ -131,7 +140,9 @@ verificacion-completa:
     TC2_PERFIL: completa
   rules:
     - if: $TC2_PERFILES =~ / completa /
-    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
+    - if: $CI_PIPELINE_SOURCE == "merge_request_event" && $TC2_COLA_DE_MERGE !~ / true /
+    - if: $CI_PIPELINE_SOURCE == "merge_request_event" && $CI_MERGE_REQUEST_EVENT_TYPE == "merge_train"
+    - if: $CI_PIPELINE_SOURCE == "merge_request_event" && $CI_MERGE_REQUEST_TARGET_BRANCH_NAME == "main" && $CI_MERGE_REQUEST_SOURCE_BRANCH_NAME == "develop"
   script:
     - $[[ inputs.comando ]] $[[ inputs.argumentos-completa ]]
````

## Excluido del diff (9 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-043/medicion/stub-verificar.sh                                    | 66 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-043/medicion/task-043-antes.txt                                   |  3 +++
 tareas/02-en-curso/TASK-043/medicion/task-043-despues.txt                                 |  3 +++
 tareas/02-en-curso/TASK-043/medicion/task-043-medir.sh                                    | 42 ++++++++++++++++++++++++++++++++++++++++++
 tareas/02-en-curso/TASK-043/medicion/task-043-release.txt                                 |  1 +
 tareas/02-en-curso/TASK-043/medicion/task-043-tiempos.sh                                  | 19 +++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-043/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-043/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-043/tarea.md                                    |  3 ++-
 9 files changed, 136 insertions(+), 1 deletion(-)
````
