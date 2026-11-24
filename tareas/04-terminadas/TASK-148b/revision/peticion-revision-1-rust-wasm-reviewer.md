# Peticion de revision — TASK-148b (ronda 1)

- Tarea: TASK-148b — escenario ssh, git de sistema y operacion de prueba (A8-01, A8-16, A3-06, A4-09, A8-20 integracion) (parte b de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148b (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-148b/tarea.md
- tareas/03-en-revision/TASK-148b/planificacion/
- crates/tc2/tests/actualizar_cli.rs
- crates/tc2/tests/agente_git/mod.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/aprobar.rs
- crates/tc2/tests/ciclo.rs
- crates/tc2/tests/comun/git.rs
- crates/tc2/tests/comun/lanzador.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/comun/ops.rs
- crates/tc2/tests/comun/remoto.rs
- crates/tc2/tests/estimacion.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/git_binario.rs
- crates/tc2/tests/importar.rs
- crates/tc2/tests/inyeccion.rs
- crates/tc2/tests/liberar.rs
- crates/tc2/tests/plan.rs
- crates/tc2/tests/plataforma_gitlab_e2e.rs
- crates/tc2/tests/progreso.rs
- crates/tc2/tests/revision.rs
- crates/tc2/tests/roadmap.rs
- crates/tc2/tests/solape.rs
- crates/tc2/tests/soporte.rs
- crates/tc2/tests/toma.rs
- crates/tc2/tests/verificar.rs

## Commits a revisar (git log develop..HEAD)

````
e7cc8d0 docs(TASK-148b): resultado, criterios y autorrevision
c6f65f0 fix(TASK-148b): A8-16 y A4-09 operacion Escribir, salida, con_fila y Preparar comunes
b0c8ebc fix(TASK-148b): A8-01 escenario ssh comun en comun/remoto.rs y lanzar en el lanzador
c88204c fix(TASK-148b): A3-06 una sola valor, texto y volcado en comun y como_persona fundido
d7f9c56 fix(TASK-148b): A8-20 git de sistema comun git_en que aisla la configuracion
b018350 chore(TASK-148b): tarea en curso
c5a7243 merge(TASK-148b): develop en fix/task-148-auditoria-4d-soporte-de-tests-compartido para seguir con la parte
````

## Diff de tu dominio (25 fichero(s) de 25; git diff develop..HEAD -- <tus ficheros>)

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
tareas/01-en-diseno/TASK-148b/tarea.md                                                     | 48 ------------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-148b/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148b/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-148b/tarea.md                                                      | 75 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 75 insertions(+), 48 deletions(-)
````

## Tests del diff (25 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(glob)**/test/**" ":(glob)**/tests/**" ":(glob)**/__tests__/**" ":(glob)**/*.test.*" ":(glob)**/*.spec.*" ":(exclude,glob)**/dist/**" ":(exclude,glob)**/*.lock" ":(exclude,glob)**/*-lock.json" ":(exclude,glob)**/*-lock.yaml" ":(exclude,glob)**/*-lock.yml" ":(exclude,glob)**/*-lock.toml" ":(exclude,glob)tareas/**" ":(exclude,glob)tareas/*/TASK-*/aceptacion/**"
````

````
crates/tc2/tests/actualizar_cli.rs        |   5 +-
 crates/tc2/tests/agente_git/mod.rs        |  17 +-----
 crates/tc2/tests/alta.rs                  |  64 +++++------------------
 crates/tc2/tests/aprobar.rs               | 267 +++++++++++++++++++++++++++++++-------------------------------------------------------------
 crates/tc2/tests/ciclo.rs                 | 400 ++++++++++++++++++++++++++++++++++++++++++++++--------------------------------------------------------------------------------------------
 crates/tc2/tests/comun/git.rs             |  34 ++++++++++++
 crates/tc2/tests/comun/lanzador.rs        |  54 ++++++++++++++++++-
 crates/tc2/tests/comun/mod.rs             |  21 ++------
 crates/tc2/tests/comun/ops.rs             |  28 ++++++++++
 crates/tc2/tests/comun/remoto.rs          | 236 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/estimacion.rs            | 184 +++++++++++++++-------------------------------------------------
 crates/tc2/tests/frescura.rs              |  92 ++++++++------------------------
 crates/tc2/tests/git_binario.rs           |   6 +--
 crates/tc2/tests/importar.rs              |  21 +-------
 crates/tc2/tests/inyeccion.rs             | 102 +++++++-----------------------------
 crates/tc2/tests/liberar.rs               | 433 ++++++++++++++++++++++++++++++++++++++++++++----------------------------------------------------------------------------------------------------------
 crates/tc2/tests/plan.rs                  | 249 ++++++++++++++++++++++++--------------------------------------------------------------
 crates/tc2/tests/plataforma_gitlab_e2e.rs |   1 +
 crates/tc2/tests/progreso.rs              | 114 +++++-----------------------------------
 crates/tc2/tests/revision.rs              |  10 ++--
 crates/tc2/tests/roadmap.rs               |   6 +--
 crates/tc2/tests/solape.rs                | 171 +++++++++--------------------------------------------------
 crates/tc2/tests/soporte.rs               |  26 +++++++++
 crates/tc2/tests/toma.rs                  | 322 +++++++++++++++++++++++++++++++--------------------------------------------------------------------------------
 crates/tc2/tests/verificar.rs             | 190 ++++++++++++++++++++++++++++++++++--------------------------------
 25 files changed, 1131 insertions(+), 1922 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

