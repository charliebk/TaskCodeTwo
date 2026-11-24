# Peticion de revision — TASK-148d (ronda 1)

- Tarea: TASK-148d — instantanea de carpeta comun (A8-12, absorbe A7-06) (parte d de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): 46a793f57e41fda34bb846be3286691a227d2b40
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148d (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-148d/tarea.md
- tareas/03-en-revision/TASK-148d/planificacion/
- crates/tc2/tests/comun/instantanea.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/desinstalar.rs
- crates/tc2/tests/doctor.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/importar.rs
- crates/tc2/tests/init.rs
- crates/tc2/tests/soporte.rs
- crates/tc2/tests/tablero.rs

## Commits a revisar (git log develop..HEAD)

````
46a793f docs(TASK-148d): resultado, criterios y autorrevision
bb9fcfa fix(TASK-148d): tablero, frescura y doctor fotografian con la instantanea comun, recursiva y con carpetas (A8-12)
2499a38 fix(TASK-148d): importar usa la instantanea comun, que ve carpetas (A8-12)
c321db0 fix(TASK-148d): init y desinstalar usan la instantanea comun (A7-06)
9cbdb86 fix(TASK-148d): instantanea de carpeta comun en tests/comun con sus tests (A8-12)
8e6310c chore(TASK-148d): tarea en curso
84e546a merge(TASK-148d): develop en fix/task-148-auditoria-4d-soporte-de-tests-compartido para seguir con la parte
````

## Diff de tu dominio (9 fichero(s) de 9; git diff develop..HEAD -- <tus ficheros>)

````diff
(solo cambian tests: ver "Tests del diff" mas abajo)
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-148d/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148d/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148d/tarea.md                                    | 30 ++++++++++++++++++++++++++----
 3 files changed, 26 insertions(+), 4 deletions(-)
````

## Tests del diff (9 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/instantanea.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/desinstalar.rs" ":(literal)crates/tc2/tests/doctor.rs" ":(literal)crates/tc2/tests/frescura.rs" ":(literal)crates/tc2/tests/importar.rs" ":(literal)crates/tc2/tests/init.rs" ":(literal)crates/tc2/tests/soporte.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/comun/instantanea.rs |  42 ++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/comun/mod.rs         |   3 +++
 crates/tc2/tests/desinstalar.rs       | 100 +++++++++++++++++++++++++++++++++++++++++++++++++++-------------------------------------------------
 crates/tc2/tests/doctor.rs            |  20 +++++---------------
 crates/tc2/tests/frescura.rs          |  22 ++++++++--------------
 crates/tc2/tests/importar.rs          |  41 ++++++++++++++++++-----------------------
 crates/tc2/tests/init.rs              |  56 ++++++++++++++++++++------------------------------------
 crates/tc2/tests/soporte.rs           |  59 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/tablero.rs           |  26 ++++++++------------------
 9 files changed, 214 insertions(+), 155 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

