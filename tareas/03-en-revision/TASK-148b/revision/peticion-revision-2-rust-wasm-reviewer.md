# Peticion de revision — TASK-148b (ronda 2)

- Tarea: TASK-148b — escenario ssh, git de sistema y operacion de prueba (A8-01, A8-16, A3-06, A4-09, A8-20 integracion) (parte b de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): 6bb76aa675970f7b6451fa69b1e32ccac3d0d454
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148b (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (4 hallazgo(s)))

## Instrucciones para el agente revisor (verificacion acotada)

Eres un revisor INDEPENDIENTE del agente que implemento. La rama ya paso una
revision completa: esta ronda verifica las correcciones de la ronda anterior,
no vuelve a revisar la rama entera ni los MENORES de la ronda anterior (tu
tabla solo trae CRITICOS e IMPORTANTES). Haz esto, y nada mas:

1. Comprueba, uno por uno, que cada hallazgo pendiente de tu tabla de abajo
   queda resuelto por el diff de esta ronda (los marcados `corregido` tambien:
   verificalo, no lo des por bueno). Para un CRITICO o IMPORTANTE, reproduce el
   fallo original y comprueba que ya no ocurre; puedes leer el repo entero si la
   causa no esta en el diff.
2. Ejecuta la suite (o el comando de verificacion del proyecto) sobre el commit
   revisado y di el resultado: verificar sin ejecutar no vale. Si esta peticion
   ya trae la verificacion de la ronda en verde para ese mismo commit, no la
   repitas: cita ese resultado y corre solo los tests de lo que verificas.
3. Busca en el diff de esta ronda el fallo que la correccion abre: es justo
   donde se cuelan los fallos nuevos. Si encuentras uno, construye el caso que
   lo rompe antes de reportarlo.

Solo abre un hallazgo NUEVO si es CRITICO o si lo introduce la propia correccion
(algo que el diff de esta ronda rompe). Lo demas que veas fuera de eso va como
nota al final del informe, no en la tabla: no abre otra ronda.

Clasifica cada hallazgo como
CRITICO (perdida de datos, corrupcion de estado, el comando hace lo
contrario de lo que dice), IMPORTANTE (comportamiento incorrecto en un
caso real, no de borde) o MENOR (todo lo demas). Un "sin hallazgos"
explicito tambien vale; inventar hallazgos, no. En la tabla, cada hallazgo pendiente que ya este
resuelto va como `corregido`; el que no, `abierto`. Vuelca tu salida en el
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-148b/tarea.md
- tareas/03-en-revision/TASK-148b/planificacion/
- crates/tc2/tests/aprobar.rs
- crates/tc2/tests/comun/git.rs
- crates/tc2/tests/comun/remoto.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/importar.rs
- crates/tc2/tests/revision.rs
- crates/tc2/tests/roadmap.rs
- crates/tc2/tests/soporte.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

Tus 3 hallazgo(s) MENOR de la ronda 1 no estan en la tabla: los comprueba quien coordina. No los revises de nuevo, ni el resto de la rama.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | crates/tc2/tests/importar.rs:62 | informe-revision-1-rust-wasm-reviewer.md |

## Commits a revisar (git log e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a..HEAD)

````
6bb76aa docs(TASK-148b): hallazgos de la ronda 1 corregidos y autorrevision
be874cb fix(TASK-148b): MEN-1, MEN-2 y MEN-3 tests de git_en, salida y con_fila
1b90eac fix(TASK-148b): IMP-1 publicado, lanzar y verify-commit sin copias
a6fbbd3 chore(TASK-148b): peticion de revision ronda 1
````

## Diff desde la ronda anterior (git diff e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a..HEAD)

````diff
(solo cambian tests: ver "Tests del diff" mas abajo)
````

## Excluido del diff (5 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-148b/planificacion/brainstorm/peticion-plan-1.md |   0
 tareas/{02-en-curso => 03-en-revision}/TASK-148b/planificacion/plan-final.md                 |   0
 tareas/03-en-revision/TASK-148b/revision/informe-revision-1-rust-wasm-reviewer.md            |  87 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-148b/revision/peticion-revision-1-rust-wasm-reviewer.md           | 140 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-148b/tarea.md                                    |  15 ++++++++---
 5 files changed, 238 insertions(+), 4 deletions(-)
````

## Tests del diff (8 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff e7cc8d0a23cfe76482d3d5068ba46e184ffd0a4a..HEAD -- ":(literal)crates/tc2/tests/aprobar.rs" ":(literal)crates/tc2/tests/comun/git.rs" ":(literal)crates/tc2/tests/comun/remoto.rs" ":(literal)crates/tc2/tests/frescura.rs" ":(literal)crates/tc2/tests/importar.rs" ":(literal)crates/tc2/tests/revision.rs" ":(literal)crates/tc2/tests/roadmap.rs" ":(literal)crates/tc2/tests/soporte.rs"
````

````
crates/tc2/tests/aprobar.rs      | 52 +++++++++++++++++-----------------------------------
 crates/tc2/tests/comun/git.rs    | 14 ++++++++++----
 crates/tc2/tests/comun/remoto.rs | 10 ++++++++--
 crates/tc2/tests/frescura.rs     | 50 +++++++++++++++++++++++---------------------------
 crates/tc2/tests/importar.rs     | 51 ++++++++++++++++++++++++++++++++++-----------------
 crates/tc2/tests/revision.rs     | 14 +++++---------
 crates/tc2/tests/roadmap.rs      | 54 ++++++++++++++++++++++--------------------------------
 crates/tc2/tests/soporte.rs      | 62 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 8 files changed, 181 insertions(+), 126 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

Sin candidatos: ninguna definicion nueva se parece a otra y ningun test nuevo promete algo que haya que mutar.

