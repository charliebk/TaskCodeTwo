# Peticion de revision — TASK-041 (ronda 2)

- Tarea: TASK-041 — F5-1 Workflow reutilizable de GitHub Actions y plantilla de GitLab
- Rama revisada: feature/task-041-f5-1-workflow-reutilizable-de-github-act
- Rama base: develop
- Commit revisado (HEAD): 5fe9c4fca6475b66f6bbe86d8025263d7efd485e
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-041 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 648720a2f6f21211bc3b31ff5e612126c6c7ce61 (el commit revisado en la ronda anterior)
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

- tareas/03-en-revision/TASK-041/tarea.md
- tareas/03-en-revision/TASK-041/planificacion/
- .gitattributes
- .github/workflows/verificacion.yml
- docs/ci/verificacion-reutilizable.md
- plantillas/gitlab/verificacion.gitlab-ci.yml

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido (944a974) | xtask/src/workflows.rs:145-164 (`leer_yaml`), 184-219 (`mapa`), 221-255 (`lista`) | informe-revision-1-rust-wasm-reviewer.md |
| IMP-2 | IMPORTANTE | corregido (944a974) | xtask/src/workflows.rs:441-447 (raiz sin inspeccionar), 377-383 y 465-474 (patrones) | informe-revision-1-rust-wasm-reviewer.md |
| IMP-3 | IMPORTANTE | corregido (944a974) | xtask/src/workflows.rs:705-711 (`la_plantilla_de_gitlab_rechaza_datos_del_autor_y_secretos`), `.gitattributes | informe-revision-1-rust-wasm-reviewer.md |
| MEN-1 | MENOR | corregido (944a974) | xtask/src/workflows.rs:477-481 y 448-506 (jobs con `uses:` a nivel de job) | informe-revision-1-rust-wasm-reviewer.md |

Tabla de hallazgos ausente o ilegible en: informe-revision-1-code-quality-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log 648720a2f6f21211bc3b31ff5e612126c6c7ce61..HEAD)

````
5fe9c4f docs(TASK-041): pipeline real de GitLab en verde y en rojo
944a974 fix(TASK-041): hallazgos de la ronda 1 de revision
171ec22 chore(TASK-041): peticion de revision ronda 1
````

## Diff de tu dominio (4 fichero(s) de 5; git diff 648720a2f6f21211bc3b31ff5e612126c6c7ce61..HEAD -- <tus ficheros>)

````diff
diff --git a/.gitattributes b/.gitattributes
index 2ce8608..b86c088 100644
--- a/.gitattributes
+++ b/.gitattributes
@@ -5,6 +5,10 @@
 *.rs text eol=lf
 *.toml text eol=lf
 Cargo.lock text eol=lf
+# Workflows y plantillas de CI: el lint de xtask/src/workflows.rs los lee y
+# sus pruebas buscan lineas literales (TASK-041).
+*.yml text eol=lf
+*.yaml text eol=lf
 # Esquema publicado de la configuracion (F2-7): se compara byte a byte con
 # el que genera el codigo.
 crates/tc2/esquemas/*.json text eol=lf
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index 1ce4070..84a5993 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -128,11 +128,12 @@ jobs:
       fail-fast: false
       matrix:
         perfil: ${{ fromJSON(needs.buscar.outputs.perfiles) }}
-    # Un runner propio comparte target/ entre ejecuciones: una sola por repo y
-    # perfil. Ojo: con cancel-in-progress false GitHub sigue cancelando la
-    # pendiente mas antigua si llega una tercera (docs/ci).
+    # Una ejecucion a la vez por PR (o ref) y perfil: los push seguidos al
+    # mismo PR se encolan y los de otros PR no se cancelan entre si. Con
+    # cancel-in-progress false GitHub cancela la pendiente mas antigua del
+    # mismo grupo si llega una tercera, que ya esta obsoleta (docs/ci).
     concurrency:
-      group: verificacion-${{ github.repository }}-${{ matrix.perfil }}
+      group: verificacion-${{ github.repository }}-${{ github.event.pull_request.number || github.ref }}-${{ matrix.perfil }}
       cancel-in-progress: false
     defaults:
       run:
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index 2d33f5c..2181869 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -33,6 +33,13 @@ se elige el runner.
 | GitLab: `tags-<perfil>` | `[]` | Etiquetas del runner del perfil |
 | GitLab: `stage` | `test` | Etapa de los jobs |
 
+> **Aviso:** `comando` y `argumentos-*` se interpolan tal cual en la línea del
+> shell (`run:` en GitHub, `script:` en GitLab). Son literales que escribe el
+> proyecto: **nunca** les pases datos del evento (`${{ github.head_ref }}`, el
+> título del PR, `$CI_COMMIT_REF_NAME`…), porque quien nombra la rama o escribe
+> el título ejecutaría código en el runner. El lint de lista blanca revisa el
+> workflow y la plantilla, no al llamador: esto no lo detecta.
+
 **Perfil según el evento** (F1-7, decisión 4), si `perfiles` va vacío:
 
 | Evento | GitHub | GitLab |
@@ -143,9 +150,14 @@ workflow: eventos admitidos (`workflow_call`, `pull_request`, `push`,
 `write-all`), la única acción admitida es `actions/checkout` fijada por SHA y
 con `persist-credentials: false` (sin `ref:` ni `repository:` en un job que
 escribe), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
-ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`. Sobre
-la plantilla: `spec:inputs`, ningún `secrets:` y ninguna variable que escribe
-el autor (`CI_COMMIT_MESSAGE`, `CI_MERGE_REQUEST_TITLE`…) en `script:`. Cada
+ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`
+(también con corchetes, `github['event']`, `secrets['X']`, o con espacios);
+`env` y `defaults` de la raíz sin secretos (y `defaults` sin datos del evento);
+un `uses:` a nivel de job fijado por SHA. Sobre la plantilla: `spec:inputs`,
+ningún `secrets:`, ninguna variable que escribe el autor (`CI_COMMIT_MESSAGE`,
+`CI_MERGE_REQUEST_TITLE`…) en `script:` y ningún `rules:if` que lea una
+variable de job. El lector falla en cerrado: lo que no entiende (sangría
+inesperada, continuación de un escalar) es un error, no se descarta. Cada
 regla tiene su mutación negativa en las pruebas.
 
 ## Divergencias con los ADR
@@ -169,11 +181,17 @@ regla tiene su mutación negativa en las pruebas.
    `GITHUB_TOKEN` en la suite de otra ejecución (el rojo de un llamador
    apareció en la lista de jobs del otro). Con `filter=latest` solo cuenta el
    último: el proyecto declara un único llamador.
-6. **Concurrencia por repo y perfil**: con `cancel-in-progress: false` GitHub
-   sigue cancelando la ejecución *pendiente* más antigua del grupo cuando llega
-   otra; ese PR queda con `ejecutar` cancelado y `verificacion` en rojo, y hay
-   que relanzarlo. Es el precio de no compartir `target/` en un runner propio
-   (no se reprodujo en el E2E).
+6. **Concurrencia por PR (o ref) y perfil**: el grupo es
+   `verificacion-<repo>-<github.event.pull_request.number || github.ref>-<perfil>`
+   (revisión de la ronda 1: por repo y perfil, un tercer PR cancelaba la
+   ejecución pendiente de otro y lo dejaba en rojo sin culpa). Ahora solo se
+   encolan las ejecuciones del mismo PR; con `cancel-in-progress: false` GitHub
+   cancela la pendiente más antigua del grupo cuando llega una tercera, que es
+   un commit ya superado del mismo PR. PR distintos corren en paralelo: un
+   runner propio con un único ejecutor los serializa por sí mismo, y varios
+   ejecutores en la misma máquina (que compartirían `target/`) quedan para
+   F5-5. Comprobado solo por la prueba `la_concurrencia_es_por_pr_o_ref`; no se
+   reprodujo en el E2E.
 
 ## Prueba E2E (2026-11-19, workflow y plantilla en `790b72d`)
 
@@ -186,7 +204,9 @@ sin mergear.
 | Stub con la forma de OpenDockTrading (no lee argumentos), mismo PR | [run 44551304248](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44551304248) | verde; `argumentos recibidos: []` |
 | ToolsaWeb, `workflow_dispatch` con `perfiles: completa pesada` | [run 44551659290](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44551659290) | verde; `--pesada` en su job, completa sin argumentos |
 | ToolsaWeb forzado a fallar (commit con `fallar.txt`) | [run 44552422966](https://github.com/charliebk/taskcodetwo-e2e-github/actions/runs/44552422966) | rojo: `completa` sale con 1 y el check `verificacion` en `failure` |
-| Plantilla de GitLab con `include: remote` por SHA | [pipeline 3089899840](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3089899840) | **no ejecutado**: «The pipeline failed due to the user not being verified» (0 jobs, 0 minutos) |
+| Plantilla de GitLab con `include: remote` por SHA (primer intento) | [pipeline 3089899840](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3089899840) | no ejecutado: «The pipeline failed due to the user not being verified» (0 jobs, 0 minutos) |
+| Plantilla de GitLab en `944a974`, push de rama con `perfiles: rapida` | [pipeline 3090338668](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3090338668) | verde: `verificacion-rapida` llama al stub con `--perfil rapida` (15 s); la regla lee `TC2_PERFILES` global |
+| La misma, stub forzado a fallar (`fallar.txt`) | [pipeline 3090341084](https://gitlab.com/taskcodetwo-e2e/taskcodetwo-e2e-gitlab/-/pipelines/3090341084) | rojo: `exit code 1` |
 
 `gh api --method GET repos/…/commits/<head>/check-runs?check_name=verificacion&app_id=15368`
 devuelve el check-run `verificacion` de la App `github-actions` (`app_id`
@@ -204,11 +224,13 @@ Suposiciones del plan:
 - **`spec:inputs` con `include: remote` en GitLab.com Free**: verificado con
   la API de lint (`POST /projects/:id/ci/lint`, `include_merged_yaml`): las
   entradas se interpolan, los cuatro jobs salen con su `script` y `tags: []`
-  es válido. Que `rules:if` lea `TC2_PERFILES` (variable del job) **no está
-  verificado**: el lint con `dry_run` también lo bloquea la verificación de
-  identidad. **Ejecución real: pendiente**;
-  GitLab.com exige verificar la identidad de la cuenta (teléfono o tarjeta)
-  para correr jobs en sus runners. Lo hace la persona; después basta un
-  pipeline verde y uno rojo con el mismo stub.
+  es válido. `TC2_PERFILES` era una variable del job y `rules:if` no ve las
+  `variables:` de los jobs (revisión de la ronda 1): ahora es una variable
+  **global** de la plantilla, que sí se evalúa en las reglas; el lint de
+  `workflows.rs` rechaza un `rules:if` que lea una variable de job. La
+  regla casa en un pipeline real: verificado el 2026-11-19 con un push de
+  rama y `perfiles: rapida` (pipelines 3090338668 en verde y 3090341084 en
+  rojo), tras verificar la persona la identidad de la cuenta en GitLab.com
+  (requisito de sus runners compartidos).
 - **`charliebk/TaskCodeTwo` público**: sí; el reutilizable se resolvió desde
   otro repo por SHA de una rama que no es `develop` ni `main`.
diff --git a/plantillas/gitlab/verificacion.gitlab-ci.yml b/plantillas/gitlab/verificacion.gitlab-ci.yml
index b09fd6b..1c8f191 100644
--- a/plantillas/gitlab/verificacion.gitlab-ci.yml
+++ b/plantillas/gitlab/verificacion.gitlab-ci.yml
@@ -53,11 +53,15 @@ spec:
       description: Etapa de los jobs de verificacion
       default: test
 ---
+# Global, no del job: `rules:if` se evalua al crear el pipeline y no ve las
+# `variables:` de los jobs.
+variables:
+  TC2_PERFILES: " $[[ inputs.perfiles ]] "
+
 .tc2-verificacion:
   stage: $[[ inputs.stage ]]
   interruptible: false
   variables:
-    TC2_PERFILES: " $[[ inputs.perfiles ]] "
     RAMA_BASE: $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
 
 verificacion-rapida:
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 648720a2f6f21211bc3b31ff5e612126c6c7ce61..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-041/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-041/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-041/revision/informe-revision-1-code-quality-reviewer.md         |  21 +++
 tareas/03-en-revision/TASK-041/revision/informe-revision-1-rust-wasm-reviewer.md            |  21 +++
 tareas/03-en-revision/TASK-041/revision/peticion-revision-1-code-quality-reviewer.md        | 580 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-041/revision/peticion-revision-1-rust-wasm-reviewer.md           | 822 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-041/tarea.md                                    |   7 +-
 7 files changed, 1449 insertions(+), 2 deletions(-)
````
