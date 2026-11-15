# Peticion de revision — TASK-021 (ronda 1)

- Tarea: TASK-021 — F2-6 Adaptador GitHub
- Rama revisada: feature/task-021-f2-6-adaptador-github
- Rama base: develop
- Commit revisado (HEAD): aa704e87c860ed49942176ea8c711ba40a83e567
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-021 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-021/tarea.md
- tareas/03-en-revision/TASK-021/planificacion/
- crates/tc2/tests/fixtures/plataforma/github/aprobacion-bot.json
- crates/tc2/tests/fixtures/plataforma/github/esperando-checks.json
- crates/tc2/tests/fixtures/plataforma/github/observado.json
- crates/tc2/tests/fixtures/plataforma/github/plan.json
- crates/tc2/tests/fixtures/plataforma/github/sin-aprobacion.json
- crates/tc2/tests/fixtures/plataforma/github/sin-cambios.json
- crates/tc2/tests/fixtures/plataforma/github/verde.json

## Commits a revisar (git log develop..HEAD)

````
aa704e8 chore(TASK-021): constancia de autorrevision
cd3479a test(TASK-021): paginacion de reviews y ultima review por persona
35ba1d7 test(TASK-021): E2E contra el repo de pruebas de GitHub y formas observadas
7638221 test(TASK-021): bateria de la PoC M, envoltorio y errores contra un servidor simulado
167ca23 feat(TASK-021): puerto Plataforma, envoltorio HTTP y adaptador GitHub
daa1160 chore(TASK-021): tarea en curso
````

## Diff de tu dominio (7 fichero(s) de 20; git diff develop..HEAD -- <tus ficheros>)

````diff
(solo cambian tests: ver "Tests del diff" mas abajo)
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                | 564 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 tareas/02-en-curso/TASK-021/evidencia/e2e.log                                             |  24 +++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-021/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-021/planificacion/plan-final.md                 |   0
 tareas/{01-en-diseno => 02-en-curso}/TASK-021/tarea.md                                    |   8 +-
 5 files changed, 594 insertions(+), 2 deletions(-)
````

## Tests del diff (7 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/plataforma/github/aprobacion-bot.json" ":(literal)crates/tc2/tests/fixtures/plataforma/github/esperando-checks.json" ":(literal)crates/tc2/tests/fixtures/plataforma/github/observado.json" ":(literal)crates/tc2/tests/fixtures/plataforma/github/plan.json" ":(literal)crates/tc2/tests/fixtures/plataforma/github/sin-aprobacion.json" ":(literal)crates/tc2/tests/fixtures/plataforma/github/sin-cambios.json" ":(literal)crates/tc2/tests/fixtures/plataforma/github/verde.json"
````

````
crates/tc2/tests/fixtures/plataforma/github/aprobacion-bot.json   |  87 ++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/github/esperando-checks.json |  95 ++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/github/observado.json        | 504 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/github/plan.json             |  82 +++++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/github/sin-aprobacion.json   |  72 ++++++++++++++++++
 crates/tc2/tests/fixtures/plataforma/github/sin-cambios.json      |  30 ++++++++
 crates/tc2/tests/fixtures/plataforma/github/verde.json            |  87 ++++++++++++++++++++++
 7 files changed, 957 insertions(+)
````
