# Peticion de revision — TASK-022 (ronda 1)

- Tarea: TASK-022 — F2-9 Adaptador GitLab
- Rama revisada: feature/task-022-f2-9-adaptador-gitlab
- Rama base: develop
- Commit revisado (HEAD): 304782b8e59e4293ddbdd512d739b3979a0ac971
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-022 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-022/tarea.md
- tareas/03-en-revision/TASK-022/planificacion/
- crates/tc2/tests/fixtures/plataforma/gitlab/aprobacion-bot.json
- crates/tc2/tests/fixtures/plataforma/gitlab/esperando-checks.json
- crates/tc2/tests/fixtures/plataforma/gitlab/observado-e2e.json
- crates/tc2/tests/fixtures/plataforma/gitlab/plan.json
- crates/tc2/tests/fixtures/plataforma/gitlab/sin-aprobacion.json
- crates/tc2/tests/fixtures/plataforma/gitlab/sin-cambios.json
- crates/tc2/tests/fixtures/plataforma/gitlab/verde.json

## Commits a revisar (git log develop..HEAD)

````
304782b chore(TASK-022): constancia de autorrevision y evidencia E2E
904be48 refactor(TASK-022): autorrevision del adaptador GitLab
773c4c6 test(TASK-022): formas de GitLab observadas en el E2E
5161b73 fix(TASK-022): puertas de GitLab segun lo observado en el E2E
70ba7bd feat(TASK-022): adaptador GitLab con la bateria compartida de F2-6
f31f910 refactor(TASK-022): transporte comun en conexion y errores por plataforma
04aacde chore(TASK-022): tarea en curso
````

## Diff de tu dominio (7 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
(solo cambian tests: ver "Tests del diff" mas abajo)
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-022/evidencia/e2e-gitlab.log                                      | 40 ++++++++++++++++++++++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-022/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-022/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-022/tarea.md                                    |  8 +++++++-
 4 files changed, 47 insertions(+), 1 deletion(-)
````

## Tests del diff (7 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/aprobacion-bot.json" ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/esperando-checks.json" ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/observado-e2e.json" ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/plan.json" ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/sin-aprobacion.json" ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/sin-cambios.json" ":(literal)crates/tc2/tests/fixtures/plataforma/gitlab/verde.json"
````

````
crates/tc2/tests/fixtures/plataforma/gitlab/aprobacion-bot.json   |  99 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/gitlab/esperando-checks.json | 107 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/gitlab/observado-e2e.json    |  14 ++++++++++++++
 crates/tc2/tests/fixtures/plataforma/gitlab/plan.json             |  96 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/gitlab/sin-aprobacion.json   |  87 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/gitlab/sin-cambios.json      |  44 ++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/gitlab/verde.json            |  99 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 7 files changed, 546 insertions(+)
````
