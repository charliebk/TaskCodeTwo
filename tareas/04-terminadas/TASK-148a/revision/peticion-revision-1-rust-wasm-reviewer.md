# Peticion de revision — TASK-148a (ronda 1)

- Tarea: TASK-148a — lanzador del binario en tests/comun (A8-02, absorbe A5-07) (parte a de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): f7970589d6f2080942bd030da79babc481b8a140
- Fecha: 2026-11-23
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148a (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-148a/tarea.md
- tareas/03-en-revision/TASK-148a/planificacion/
- crates/tc2/tests/actualizar_cli.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/aprobar.rs
- crates/tc2/tests/ciclo.rs
- crates/tc2/tests/cli.rs
- crates/tc2/tests/comun/lanzador.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/convivencia_016.rs
- crates/tc2/tests/doctor.rs
- crates/tc2/tests/estimacion.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/git_binario.rs
- crates/tc2/tests/hallazgos.rs
- crates/tc2/tests/importar.rs
- crates/tc2/tests/inyeccion.rs
- crates/tc2/tests/liberar.rs
- crates/tc2/tests/plan.rs
- crates/tc2/tests/plataforma_e2e.rs
- crates/tc2/tests/progreso.rs
- crates/tc2/tests/recursos.rs
- crates/tc2/tests/revisoras.rs
- crates/tc2/tests/roadmap.rs
- crates/tc2/tests/seleccion.rs
- crates/tc2/tests/siguiente.rs
- crates/tc2/tests/solape.rs
- crates/tc2/tests/soporte.rs
- crates/tc2/tests/tablero.rs
- crates/tc2/tests/toma.rs
- crates/tc2/tests/verificar.rs

## Commits a revisar (git log develop..HEAD)

````
f797058 docs(TASK-148a): constancia de autorrevision
3e2dba4 docs(TASK-148a): resultado con recuento antes y despues
1466bd4 fix(TASK-148a): lanzador unico de tc2 en tests/comun (A8-02, A5-07)
6f699e7 chore(TASK-148a): tarea en curso
````

## Diff de tu dominio (29 fichero(s) de 29; git diff develop..HEAD -- <tus ficheros>)

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
tareas/{01-en-diseno => 02-en-curso}/TASK-148a/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148a/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148a/tarea.md                                    | 26 +++++++++++++++++++++++++-
 3 files changed, 25 insertions(+), 1 deletion(-)
````

## Tests del diff (29 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(glob)**/test/**" ":(glob)**/tests/**" ":(glob)**/__tests__/**" ":(glob)**/*.test.*" ":(glob)**/*.spec.*" ":(exclude,glob)**/dist/**" ":(exclude,glob)**/*.lock" ":(exclude,glob)**/*-lock.json" ":(exclude,glob)**/*-lock.yaml" ":(exclude,glob)**/*-lock.yml" ":(exclude,glob)**/*-lock.toml" ":(exclude,glob)tareas/**" ":(exclude,glob)tareas/*/TASK-*/aceptacion/**"
````

````
crates/tc2/tests/actualizar_cli.rs  | 12 +++++-------
 crates/tc2/tests/alta.rs            |  6 ++----
 crates/tc2/tests/aprobar.rs         |  7 +------
 crates/tc2/tests/ciclo.rs           |  6 +-----
 crates/tc2/tests/cli.rs             |  6 +++---
 crates/tc2/tests/comun/lanzador.rs  | 31 +++++++++++++++++++++++++++++++
 crates/tc2/tests/comun/mod.rs       |  4 ++++
 crates/tc2/tests/convivencia_016.rs |  5 +----
 crates/tc2/tests/doctor.rs          |  9 +--------
 crates/tc2/tests/estimacion.rs      | 14 ++------------
 crates/tc2/tests/frescura.rs        |  8 +-------
 crates/tc2/tests/git_binario.rs     |  4 ++--
 crates/tc2/tests/hallazgos.rs       |  4 +---
 crates/tc2/tests/importar.rs        |  6 ++----
 crates/tc2/tests/inyeccion.rs       | 12 +++---------
 crates/tc2/tests/liberar.rs         |  6 +-----
 crates/tc2/tests/plan.rs            |  9 ++-------
 crates/tc2/tests/plataforma_e2e.rs  |  4 ++--
 crates/tc2/tests/progreso.rs        | 22 ++++++----------------
 crates/tc2/tests/recursos.rs        |  9 +--------
 crates/tc2/tests/revisoras.rs       |  5 ++---
 crates/tc2/tests/roadmap.rs         |  5 +----
 crates/tc2/tests/seleccion.rs       |  5 +----
 crates/tc2/tests/siguiente.rs       | 11 +++--------
 crates/tc2/tests/solape.rs          | 10 ++--------
 crates/tc2/tests/soporte.rs         | 45 +++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/tablero.rs         |  8 +-------
 crates/tc2/tests/toma.rs            |  6 +-----
 crates/tc2/tests/verificar.rs       |  5 ++---
 29 files changed, 130 insertions(+), 154 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

