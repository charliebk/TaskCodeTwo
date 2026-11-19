# Peticion de revision — TASK-044 (ronda 1)

- Tarea: TASK-044 — F5-4 Informe de verificacion en JSON para el revisor IA
- Rama revisada: feature/task-044-f5-4-informe-de-verificacion-en-json-par
- Rama base: develop
- Commit revisado (HEAD): cf29ab03914858f9e85525bb157b39be5715d0ca
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-044 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-044/tarea.md
- tareas/03-en-revision/TASK-044/planificacion/
- .github/workflows/verificacion.yml
- crates/tc2/esquemas/informe-verificacion.schema.json
- crates/tc2/tests/fixtures/informe-verificacion/xtask-rojo.json
- crates/tc2/tests/fixtures/informe-verificacion/xtask.json
- docs/ci/verificacion-reutilizable.md
- plantillas/gitlab/verificacion.gitlab-ci.yml

## Commits a revisar (git log develop..HEAD)

````
cf29ab0 fix(TASK-044): mutaciones del informe con el fixture en CRLF
c56abd8 feat(TASK-044): informe de verificacion en JSON para el revisor IA (F5-4)
a81a8fe chore(TASK-044): tarea en curso
````

## Diff de tu dominio (6 fichero(s) de 11; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.github/workflows/verificacion.yml b/.github/workflows/verificacion.yml
index 84a5993..dbb6ee8 100644
--- a/.github/workflows/verificacion.yml
+++ b/.github/workflows/verificacion.yml
@@ -5,7 +5,9 @@
 #   - buscar (ejecuta, Linux, contents: read): elige los perfiles del evento
 #     (F1-7, decision 4). La busqueda en la cache por arbol llega con F5-2.
 #   - ejecutar (ejecuta, matriz por perfil, runner del perfil, sin secretos):
-#     corre `comando argumentos-<perfil>` en el shell que declara el proyecto.
+#     corre `comando argumentos-<perfil>` en el shell que declara el proyecto
+#     y sube el informe JSON de `TC2_INFORME` (F5-4) como artefacto
+#     `informe-verificacion-<perfil>`, si el comando lo escribio.
 #   - verificacion (privilegio, sin checkout): suma los resultados y publica el
 #     check-run `verificacion` con GITHUB_TOKEN (App github-actions, 15368).
 #
@@ -145,8 +147,19 @@ jobs:
       - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
         with:
           persist-credentials: false
+      # TC2_INFORME (F5-4): el comando escribe ahi su informe JSON, si sabe;
+      # la plataforma solo lo sube. `runner` no existe en el `env` del job.
       - name: comando del proyecto, perfil ${{ matrix.perfil }}
+        env:
+          TC2_INFORME: ${{ runner.temp }}/informe-verificacion.json
         run: ${{ inputs.comando }} ${{ inputs[format('argumentos-{0}', matrix.perfil)] }}
+      - name: informe de verificacion, perfil ${{ matrix.perfil }}
+        if: always()
+        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
+        with:
+          name: informe-verificacion-${{ matrix.perfil }}
+          path: ${{ runner.temp }}/informe-verificacion.json
+          if-no-files-found: ignore
 
   verificacion:
     name: verificacion
@@ -168,7 +181,13 @@ jobs:
         run: |
           set -eu
           if [ "$RESULTADO" = success ]; then conclusion=success; else conclusion=failure; fi
-          resumen="perfiles=$PERFILES resultado=$RESULTADO ejecucion=$EJECUCION"
+          # Nombres de los artefactos del informe JSON (F5-4), uno por perfil;
+          # el publicador no los abre: el informe lo escribe codigo del PR.
+          informes=""
+          for p in $(printf '%s' "$PERFILES" | tr -d '[]"' | tr ',' ' '); do
+            informes="${informes:+$informes,}informe-verificacion-$p"
+          done
+          resumen="perfiles=$PERFILES resultado=$RESULTADO informe=$informes ejecucion=$EJECUCION"
           gh api --method POST "repos/$REPO/check-runs" \
             -f name=verificacion -f head_sha="$HEAD_SHA" -f status=completed \
             -f conclusion="$conclusion" -f details_url="$EJECUCION" \
diff --git a/crates/tc2/esquemas/informe-verificacion.schema.json b/crates/tc2/esquemas/informe-verificacion.schema.json
new file mode 100644
index 0000000..95f95be
--- /dev/null
+++ b/crates/tc2/esquemas/informe-verificacion.schema.json
@@ -0,0 +1,71 @@
+{
+  "$schema": "https://json-schema.org/draft/2020-12/schema",
+  "title": "Informe de verificacion de TaskCode 2, version 1 (F5-4)",
+  "description": "Lo escribe el comando de verificacion del proyecto en la ruta de TC2_INFORME, tambien en rojo; la CI lo sube como artefacto informe-verificacion-<perfil>. Lo lee tc2 (verificacion::informe::leer). Es detalle para citar: el hecho autoritativo es el check verificacion. Una clave desconocida falla cerrado.",
+  "type": "object",
+  "additionalProperties": false,
+  "required": ["version", "perfil", "resultado", "commit", "inicio", "fin", "pasos"],
+  "properties": {
+    "version": {
+      "description": "version del contrato; otra distinta de 1 es error",
+      "const": 1
+    },
+    "perfil": {
+      "description": "perfil de verificacion [F1-7 decision 4]",
+      "enum": ["rapida", "impacto", "completa", "pesada"]
+    },
+    "resultado": {
+      "description": "resultado de la ejecucion; verde exige todos los pasos en verde",
+      "enum": ["verde", "rojo"]
+    },
+    "commit": {
+      "description": "commit verificado (git rev-parse HEAD); null si no se pudo leer",
+      "type": ["string", "null"],
+      "pattern": "^[0-9a-f]{40}$"
+    },
+    "inicio": {
+      "description": "inicio en UTC, AAAA-MM-DDTHH:MM:SSZ",
+      "type": "string",
+      "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"
+    },
+    "fin": {
+      "description": "fin en UTC, AAAA-MM-DDTHH:MM:SSZ",
+      "type": "string",
+      "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"
+    },
+    "pasos": {
+      "description": "pasos ejecutados, en orden; tras el primero en rojo no hay mas",
+      "type": "array",
+      "items": {
+        "type": "object",
+        "additionalProperties": false,
+        "required": ["nombre", "resultado", "segundos"],
+        "properties": {
+          "nombre": {
+            "description": "nombre del paso: minusculas, digitos, punto, guion y guion bajo",
+            "type": "string",
+            "pattern": "^[a-z0-9][a-z0-9._-]{0,63}$"
+          },
+          "resultado": {
+            "enum": ["verde", "rojo"]
+          },
+          "segundos": {
+            "description": "duracion en segundos enteros",
+            "type": "integer",
+            "minimum": 0
+          }
+        }
+      }
+    },
+    "tests": {
+      "description": "recuento de tests, opcional [F1-11 practica 8]; fallidos no supera total",
+      "type": "object",
+      "additionalProperties": false,
+      "required": ["total", "fallidos"],
+      "properties": {
+        "total": { "type": "integer", "minimum": 0 },
+        "fallidos": { "type": "integer", "minimum": 0 }
+      }
+    }
+  }
+}
diff --git a/docs/ci/verificacion-reutilizable.md b/docs/ci/verificacion-reutilizable.md
index 2181869..27b181e 100644
--- a/docs/ci/verificacion-reutilizable.md
+++ b/docs/ci/verificacion-reutilizable.md
@@ -9,9 +9,10 @@ decisión 1; RR-15): no conocen sus pasos, sus grupos ni sus herramientas.
 | GitHub Actions | [`.github/workflows/verificacion.yml`](../../.github/workflows/verificacion.yml) (`on: workflow_call`) | un job del proyecto con `uses: charliebk/TaskCodeTwo/.github/workflows/verificacion.yml@<sha>` |
 | GitLab CI | [`plantillas/gitlab/verificacion.gitlab-ci.yml`](../../plantillas/gitlab/verificacion.gitlab-ci.yml) (`spec:inputs`) | `include: remote` a la URL raw de la plantilla fijada por SHA |
 
-Fuera de F5-1: la caché por árbol y la línea `arbol=` (F5-2), el informe JSON
-para el revisor (F5-4), el runner propio (F5-5), `carril` y `revision-ia`
-(F6), y la definición de CI en otro proyecto de GitLab (F2-9).
+Fuera de F5-1: la caché por árbol y la línea `arbol=` (F5-2), el runner
+propio (F5-5), `carril` y `revision-ia` (F6), y la definición de CI en otro
+proyecto de GitLab (F2-9). El informe JSON para el revisor (F5-4) se describe
+más abajo.
 
 ## Entradas
 
@@ -147,9 +148,10 @@ quiere evitar el pipeline de rama duplicado con el de MR, lo pone en su
 `cargo test` (y por tanto en `cargo xtask verificar --perfil rapida`). Sobre el
 workflow: eventos admitidos (`workflow_call`, `pull_request`, `push`,
 `merge_group`, `workflow_dispatch`), `permissions` arriba y en cada job (sin
-`write-all`), la única acción admitida es `actions/checkout` fijada por SHA y
-con `persist-credentials: false` (sin `ref:` ni `repository:` en un job que
-escribe), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
+`write-all`), las únicas acciones admitidas son `actions/checkout` fijada por
+SHA y con `persist-credentials: false` (sin `ref:` ni `repository:` en un job
+que escribe) y `actions/upload-artifact` fijada por SHA y solo en un job sin
+escritura (F5-4), ningún `secrets: inherit`, ningún secreto en un job sin escritura y
 ningún `github.event.*`, `github.head_ref` ni `toJSON(github…)` en `run:`
 (también con corchetes, `github['event']`, `secrets['X']`, o con espacios);
 `env` y `defaults` de la raíz sin secretos (y `defaults` sin datos del evento);
@@ -160,6 +162,34 @@ variable de job. El lector falla en cerrado: lo que no entiende (sangría
 inesperada, continuación de un escalar) es un error, no se descarta. Cada
 regla tiene su mutación negativa en las pruebas.
 
+## Informe JSON (F5-4)
+
+TASK-044. El comando de verificación del proyecto escribe, si sabe, un informe
+JSON en la ruta de la variable **`TC2_INFORME`**, también cuando acaba en rojo.
+La plataforma solo lo transporta (F1-7, decisión 1: no conoce los pasos):
+
+| Plataforma | `TC2_INFORME` | Transporte |
+|---|---|---|
+| GitHub | `${{ runner.temp }}/informe-verificacion.json` (env del paso) | `actions/upload-artifact` con `if: always()` e `if-no-files-found: ignore`, artefacto `informe-verificacion-<perfil>`; el publicador añade `informe=informe-verificacion-<perfil>[,…]` a `output.summary` |
+| GitLab | `$CI_PROJECT_DIR/.tc2/informe-verificacion.json` | `artifacts: {name: informe-verificacion-$TC2_PERFIL, when: always, paths: [.tc2/informe-verificacion.json]}` |
+
+- **Contrato**: [`crates/tc2/esquemas/informe-verificacion.schema.json`](../../crates/tc2/esquemas/informe-verificacion.schema.json),
+  `version: 1`: `perfil`, `resultado` (`verde`/`rojo`), `commit` (40 hex o
+  `null`), `inicio` y `fin` (UTC), `pasos[{nombre, resultado, segundos}]` y
+  `tests{total, fallidos}` opcional (F1-11, práctica 8). Una clave desconocida
+  u otra versión es error.
+- **Lector**: `tc2::verificacion::informe::leer` y `seccion_revisor(&informe,
+  artefacto)`, el texto con el que el revisor IA (F6-3) cita el informe y se
+  compromete a no relanzar los pasos en verde.
+- **Productor**: `cargo xtask verificar` (fmt, clippy, test y, en `completa`,
+  reproducible), sin recuento de tests. Su prueba compara el JSON con los
+  fixtures `crates/tc2/tests/fixtures/informe-verificacion/` que lee tc2.
+- **Quién decide**: el informe lo escribe código del PR; es detalle para citar.
+  El hecho autoritativo para no repetir la suite es el check `verificacion` de
+  `github-actions` (`app_id` 15368). Si el comando no escribe el informe, no
+  hay artefacto y el revisor solo tiene el resumen del check, que basta para no
+  relanzar un perfil en verde. El publicador no abre el artefacto.
+
 ## Divergencias con los ADR
 
 1. **Argumentos por perfil** (D1): F1-7, decisión 1, fija `--perfil
@@ -192,6 +222,22 @@ regla tiene su mutación negativa en las pruebas.
    ejecutores en la misma máquina (que compartirían `target/`) quedan para
    F5-5. Comprobado solo por la prueba `la_concurrencia_es_por_pr_o_ref`; no se
    reprodujo en el E2E.
+7. **Segunda acción en la lista blanca** (F5-4, D3): F1-8 y esta guía decían
+   que la única acción admitida es `actions/checkout`. Se admite también
+   `actions/upload-artifact`, fijada por SHA y solo en clase ejecuta (sin
+   escritura ni secretos), porque el artefacto es el canal que F1-8 fija para
+   el revisor y existe en las dos plataformas (RR-15).
+8. **E2E del informe pendiente** (F5-4, D6): TASK-044 no hace push. Se valida
+   en local (pruebas de xtask y tc2) y la plantilla con la API `ci/lint` de
+   GitLab (contenido en línea, entradas sustituidas por valores de ejemplo y
+   sin la cabecera `spec`). La ejecución real en GitHub (artefacto presente,
+   `informe=` en el resumen) y la descarga del artefacto de otro run con
+   `actions: read` se hacen en TASK-049, que consume el informe.
+9. **"No repite tests"** (criterio 1): solo es comprobable con el revisor de
+   F6-3. TASK-049 debe usar `seccion_revisor` y demostrarlo.
+10. **ToolsaWeb y OpenDockTrading** no producen el informe hasta migrar (F9):
+    su `.estado/verificar-ultima.json` tiene otra forma. Hasta entonces solo
+    existe el resumen del check.
 
 ## Prueba E2E (2026-11-19, workflow y plantilla en `790b72d`)
 
diff --git a/plantillas/gitlab/verificacion.gitlab-ci.yml b/plantillas/gitlab/verificacion.gitlab-ci.yml
index 1c8f191..e553b53 100644
--- a/plantillas/gitlab/verificacion.gitlab-ci.yml
+++ b/plantillas/gitlab/verificacion.gitlab-ci.yml
@@ -13,6 +13,8 @@
 #         comando: scripts/verificar.sh
 # Fuera de F5-1 (D4): carril, estado de revision-ia y ci_config_path en otro
 # proyecto. La linea de la cache por arbol ira en un artefacto del job (D3, F5-2).
+# Informe JSON (F5-4): el comando lo escribe en TC2_INFORME y cada job lo sube
+# como artefacto informe-verificacion-<perfil> (when: always).
 # Guia: docs/ci/verificacion-reutilizable.md.
 spec:
   inputs:
@@ -63,6 +65,13 @@ variables:
   interruptible: false
   variables:
     RAMA_BASE: $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
+    # F5-4: el comando escribe ahi su informe JSON, si sabe; el job lo sube.
+    TC2_INFORME: $CI_PROJECT_DIR/.tc2/informe-verificacion.json
+  artifacts:
+    name: informe-verificacion-$TC2_PERFIL
+    when: always
+    paths:
+      - .tc2/informe-verificacion.json
 
 verificacion-rapida:
   extends: .tc2-verificacion
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-044/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-044/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-044/tarea.md                                    | 3 ++-
 3 files changed, 2 insertions(+), 1 deletion(-)
````

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/informe-verificacion/xtask-rojo.json" ":(literal)crates/tc2/tests/fixtures/informe-verificacion/xtask.json"
````

````
crates/tc2/tests/fixtures/informe-verificacion/xtask-rojo.json | 12 ++++++++++++
 crates/tc2/tests/fixtures/informe-verificacion/xtask.json      | 14 ++++++++++++++
 2 files changed, 26 insertions(+)
````
