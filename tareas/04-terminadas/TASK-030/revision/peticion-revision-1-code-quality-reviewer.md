# Peticion de revision — TASK-030 (ronda 1)

- Tarea: TASK-030 — F3-3 tc2 next: lo listo para empezar por dependencias y prioridad
- Rama revisada: feature/task-030-f3-3-next-lo-listo-para-empezar-por-depe
- Rama base: develop
- Commit revisado (HEAD): fc1f1887ff81feae206dda283f77e319fdfb12de
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-030 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-030/tarea.md
- tareas/03-en-revision/TASK-030/planificacion/
- crates/tc2/tests/fixtures/toolsaweb-1fc94425/esperado.txt
- crates/tc2/tests/fixtures/toolsaweb-1fc94425/hitos.txt
- crates/tc2/tests/fixtures/toolsaweb-1fc94425/tareas.tsv

## Commits a revisar (git log develop..HEAD)

````
fc1f188 chore(TASK-030): C3 con las claves del contrato F1-4
d8dde8d chore(TASK-030): autorrevision y notas de implementacion
484e64a fix(TASK-030): aviso duplicadas= para IDs repetidos
38224ca feat(TASK-030): orden next con --json, i18n y E2E
ffbbb1c refactor(TASK-030): estado::tareas lee los tarea.md de los dos formatos
02a03b3 test(TASK-030): fixture de ToolsaWeb 1fc94425 y test de C1
3cc103a feat(TASK-030): nucleo grafo con orden de trabajo y listas
d783d91 chore(TASK-030): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 13; git diff develop..HEAD -- <tus ficheros>)

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
tareas/01-en-diseno/TASK-030/tarea.md                                                     | 42 ------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-030/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-030/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-030/tarea.md                                                      | 57 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 57 insertions(+), 42 deletions(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/toolsaweb-1fc94425/esperado.txt" ":(literal)crates/tc2/tests/fixtures/toolsaweb-1fc94425/hitos.txt" ":(literal)crates/tc2/tests/fixtures/toolsaweb-1fc94425/tareas.tsv"
````

````
crates/tc2/tests/fixtures/toolsaweb-1fc94425/esperado.txt |   3 +
 crates/tc2/tests/fixtures/toolsaweb-1fc94425/hitos.txt    |  20 +++++++
 crates/tc2/tests/fixtures/toolsaweb-1fc94425/tareas.tsv   | 406 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 429 insertions(+)
````
