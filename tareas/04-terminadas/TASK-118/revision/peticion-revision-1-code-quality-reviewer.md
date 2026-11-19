# Peticion de revision — TASK-118 (ronda 1)

- Tarea: TASK-118 — Aplicar las puertas (rulesets) desde tc2 init
- Rama revisada: feature/task-118-aplicar-las-puertas-rulesets-desde-tc2-i
- Rama base: develop
- Commit revisado (HEAD): 34a6b7b21259c24709fa626eb1939edb7a053ecc
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-118 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-118/tarea.md
- tareas/03-en-revision/TASK-118/planificacion/
- crates/tc2/esquemas/config.schema.json
- docs/adr/F1-13-adaptador-de-plataforma.md
- docs/adr/F1-5-puertas-en-github.md

## Commits a revisar (git log develop..HEAD)

````
34a6b7b chore(TASK-118): autorrevision
a7f212e feat(TASK-118): adenda de aplicar-puertas en F1-5 y F1-13
ce027ac feat(TASK-118): aplicar las puertas desde tc2 init --aplicar-puertas
eef310d chore(TASK-118): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/esquemas/config.schema.json b/crates/tc2/esquemas/config.schema.json
index 4586fc5..52f2c59 100644
--- a/crates/tc2/esquemas/config.schema.json
+++ b/crates/tc2/esquemas/config.schema.json
@@ -151,6 +151,11 @@
             "gitlab"
           ],
           "type": "string"
+        },
+        "revisor_id": {
+          "description": "origen de revision-ia: el app_id de la App del revisor en GitHub o su user.id en GitLab; 0 o ausente, no hay y tc2 init --aplicar-puertas se rechaza (TASK-118)",
+          "minimum": 0,
+          "type": "integer"
         }
       },
       "required": [
diff --git a/docs/adr/F1-13-adaptador-de-plataforma.md b/docs/adr/F1-13-adaptador-de-plataforma.md
index 4d28e58..7a4ae1c 100644
--- a/docs/adr/F1-13-adaptador-de-plataforma.md
+++ b/docs/adr/F1-13-adaptador-de-plataforma.md
@@ -375,7 +375,7 @@ no se ha configurado nada en ninguna plataforma):
    | `abrir-pr` | `GET /pulls?head=` y, si no hay, `POST /pulls` | `GET /merge_requests?source_branch=` y, si no hay, `POST /merge_requests` |
    | `estado-puertas` | `GET /pulls/{n}` (con `mergeable_state`), `GET /rules/branches/{base}`, `GET /commits/{sha}/check-runs`, `GET /pulls/{n}/reviews` | `GET /merge_requests/{iid}?include_diverged_commits_count=true`, `GET /approval_rules`, `GET /protected_branches/{destino}`, `GET /repository/commits/{sha}/statuses`, `GET /merge_requests/{iid}/approvals` |
    | `merge-pr` | `PUT /pulls/{n}/merge` con `merge_method=merge`; con cola, la cola | `PUT /merge_requests/{iid}/merge` (el método lo fija el proyecto); Premium con tren: el tren |
-   | `aplicar-puertas` | rulesets `taskcodetwo-*` | ramas y tags protegidos, ajustes de merge del proyecto, reglas y ajustes de aprobación (Premium) |
+   | `aplicar-puertas` (adenda de TASK-118) | `GET /rulesets`, `GET /commits/heads/develop/check-runs` y, de las que faltan, `POST /rulesets`; nunca `PUT` ni `DELETE` | `GET /protected_branches`, `GET /protected_tags`, `GET /repository/branches/develop` y sus estados, `GET /projects/:id` y el namespace; de lo que falta, `POST /protected_branches`, `POST /protected_tags`, `PUT /projects/:id` (ajustes de merge) y en Premium `POST /approval_rules`; nunca `DELETE` ni `PATCH` |
    | `leer-puertas` | `GET /rulesets`, `GET /rules/branches/{rama}` | `GET /protected_branches`, `GET /protected_tags`, `GET /projects/:id` con sesión, `GET /approval_rules`, `GET /approvals` |
    | `leer-permisos-ci` | `GET /actions/permissions/workflow` | ninguna: el job token es fijo (G16); informa `fuente=plataforma` |
    | firma del `approve` (`carril`) | `GET /commits/{sha}` | `GET /repository/commits/{sha}/signature` |
diff --git a/docs/adr/F1-5-puertas-en-github.md b/docs/adr/F1-5-puertas-en-github.md
index 32d828a..88221ca 100644
--- a/docs/adr/F1-5-puertas-en-github.md
+++ b/docs/adr/F1-5-puertas-en-github.md
@@ -617,3 +617,60 @@ riesgos sin prueba:
   `verified` exige la clave de esa cuenta. No verificado.
 - **Pilotos privados**: ToolsaWeb y OpenDockTrading quedarían sin puertas
   impuestas en una cuenta gratuita (D2a); su plan real no se ha comprobado.
+
+## Adenda (TASK-118) — `aplicar-puertas` desde `tc2 init --aplicar-puertas`
+
+**La operación.** El puerto `Plataforma` gana `preparar_puertas` (solo
+lecturas) y `aplicar_puertas` (en lote), con implementación por defecto
+`aplicar-puertas rechazada motivo=no-soportada`. `aplicar_puertas` primero
+repite todas las comprobaciones de `preparar_puertas` y solo después
+escribe; crea las puertas que faltan, una a una, y si un fallo corta a mitad
+la línea dice `aplicadas=` y `faltan=`. Repetirla converge: lo ya creado
+sale como existente. Un 422 de GitHub o un 409 de GitLab al crear es «otro
+llegó antes» y cuenta como existente. GitHub: `POST /repos/{r}/rulesets`
+con el JSON de la PoC C, `integration_id` por check y sin `_nota`; el 403 de
+plan da `rechazada motivo=plan`. GitLab: `POST /protected_branches` y
+`POST /protected_tags` con niveles explícitos (nunca el valor por defecto
+de la API), `PUT /projects/:id` con `merge_method=merge` y
+`only_allow_merge_if_pipeline_succeeds=true` cuando se crea `develop` o
+`main`, y en Premium una regla de aprobación `taskcodetwo-aprobacion` si no
+hay ninguna que pida al menos una; en Free, la línea lleva
+`sin-aprobaciones=si`. Nunca `DELETE` ni `PATCH` (el cliente HTTP no los
+tiene).
+
+**El origen de cada check.** Clave nueva `plataforma.revisor_id` (entero; 0
+o ausente, no hay): el `app_id` de la App del revisor en GitHub o su
+`user.id` en GitLab, origen de `revision-ia`. En GitHub, `verificacion` y
+`carril` son de Actions (15368, PoC A). En GitLab no tienen origen fijo: son
+jobs del pipeline, que «Pipelines must succeed» impone entero y cuyo autor
+es quien lanzó el pipeline (F1-13, G09); solo se exige haberlos visto
+publicados.
+
+**La precondición** (todo antes de escribir nada, ni en el disco ni en la
+plataforma; si falla, `init rechazada motivo=… [faltan=…]` y las cinco
+líneas `ruleset` salen `rechaza`): la plataforma con el token de la persona;
+`personas.yml` ya existente (`sin-personas`) y la identidad de persona en
+él (`sin-persona`); `develop` y `main` en `origin` (`ramas-sin-remoto`) sin
+commits locales por delante (`ramas-por-delante`); el origen de cada check
+en la configuración (`puertas-sin-origen faltan=revision-ia`); y cada check
+ya visto publicado por su origen en la cabeza de `develop` (GitHub:
+check-runs con `app.id`; GitLab: estados del commit con `author.id`;
+`puertas-sin-origen faltan=…`). En la práctica la opción se rechaza hasta
+que F5-1 y F6-1 publiquen los checks.
+
+**Divergencias con esta ADR.**
+- La tabla de la decisión 1 decía que un ruleset existente y distinto «los
+  actualiza». TASK-118 no lo hace: lo existente nunca se reescribe ni se
+  borra. En GitHub lo existente se reconoce por el nombre y no se compara;
+  en GitLab una rama o un tag protegido distinto del modelo sale
+  `existe motivo=difiere` (p. ej. `main`, que GitLab protege al crear el
+  proyecto con push de Maintainer). Actualizarlos queda fuera de alcance.
+- En GitLab, `develop` y `main` quedan con push «nadie» y merge de
+  Maintainer (G01 de `estado-puertas`, F1-13 D6), no de Developer; las ramas
+  de tarea (`feature/*`, `fix/*`, `hotfix/*`) y `taskcodetwo/tareas`, con push
+  y merge de Developer y sin force-push; el tag `v*`, creación de Developer.
+
+**No verificado:** que GitHub acepte el `POST /rulesets` de la PoC C con
+`integration_id`, que dé 422 con un nombre repetido; en GitLab, que el `*`
+de `feature/*` abarque nombres con `/`, el 409 de una protección repetida y
+el cuerpo de la regla `any_approver`. Todo, contra el doble HTTP.
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/peticion-brainstorm-arquitectura-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/peticion-brainstorm-riesgos-1.md      | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/peticion-unificador-1.md              | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/salida-brainstorm-arquitectura-1.md   | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/brainstorm/salida-brainstorm-riesgos-1.md        | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/planificacion/plan-final.md                                    | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-118/tarea.md                                                       | 8 +++++++-
 7 files changed, 7 insertions(+), 1 deletion(-)
````
