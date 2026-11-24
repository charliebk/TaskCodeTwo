# Peticion de revision — TASK-148c (ronda 1)

- Tarea: TASK-148c — servidor HTTP de los tests con comun::http_mock (A8-11) (parte c de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): 56223308c105aa55a88b7d10ae2cb92dd1133ef7
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148c (criterios de aceptacion y plan)
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
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-148c/tarea.md
- tareas/03-en-revision/TASK-148c/planificacion/
- crates/tc2/tests/actualizar_cli.rs
- crates/tc2/tests/comun/http_mock.rs
- crates/tc2/tests/identidad.rs
- crates/tc2/tests/init.rs
- crates/tc2/tests/soporte.rs

## Commits a revisar (git log develop..HEAD)

````
5622330 docs(TASK-148c): resultado, criterios y autorrevision
4e55f40 fix(TASK-148c): actualizar_cli.rs usa comun::http_mock en lugar de su Servidor propio (A8-11)
4eeafeb fix(TASK-148c): http_mock sirve bytes y respuestas cortadas (A8-11)
b932aeb fix(TASK-148c): init.rs usa comun::http_mock en lugar de su Servidor propio (A8-11)
5afdc1e fix(TASK-148c): identidad.rs usa comun::http_mock en lugar de su Servidor propio (A8-11)
69f855c chore(TASK-148c): tarea en curso
71bc15f merge(TASK-148c): develop en fix/task-148-auditoria-4d-soporte-de-tests-compartido para seguir con la parte
````

## Diff de tu dominio (5 fichero(s) de 5; git diff develop..HEAD -- <tus ficheros>)

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
tareas/01-en-diseno/TASK-148c/tarea.md                                                     | 46 ----------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-148c/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148c/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-148c/tarea.md                                                      | 68 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 68 insertions(+), 46 deletions(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/actualizar_cli.rs" ":(literal)crates/tc2/tests/comun/http_mock.rs" ":(literal)crates/tc2/tests/identidad.rs" ":(literal)crates/tc2/tests/init.rs" ":(literal)crates/tc2/tests/soporte.rs"
````

````
crates/tc2/tests/actualizar_cli.rs  | 257 +++++++++++++++++++++++++++++++++++++++++++++---------------------------------------------------------------------------------------------------------------
 crates/tc2/tests/comun/http_mock.rs |  48 +++++++++++++++++++++++++----
 crates/tc2/tests/identidad.rs       | 132 +++++++++++++++++++++++++-------------------------------------------------------
 crates/tc2/tests/init.rs            | 116 ++++++++++++----------------------------------------------------------
 crates/tc2/tests/soporte.rs         |  59 ++++++++++++++++++++++++++++++++++++
 5 files changed, 233 insertions(+), 379 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

