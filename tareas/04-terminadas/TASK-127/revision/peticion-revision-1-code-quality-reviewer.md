# Peticion de revision — TASK-127 (ronda 1)

- Tarea: TASK-127 — Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)
- Rama revisada: fix/task-127-escrituras-de-estado-con-muchos-ficheros
- Rama base: develop
- Commit revisado (HEAD): 7d1963a0d80f19af3edc996e533d258f37d41a69
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-127 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-127/tarea.md
- tareas/03-en-revision/TASK-127/planificacion/
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
7d1963a chore(TASK-127): autorrevision
22ffc96 fix(TASK-127): rutas e indice por stdin en commit-estado
43333c7 chore(TASK-127): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 6; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index beccbc7..610ebbc 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -575,3 +575,4 @@ con su evidencia en los tests de `crates/tc2/tests/estado_*.rs`:
    `sinred`. El test de F2-2 que simulaba lo transitorio con un hook que
    rechaza una vez pasa a simularlo con un `receive-pack` que muere la
    primera vez.
+- Adenda (2026-11-18, TASK-127): `commit-estado` pasa las rutas de los blobs y las entradas del índice por stdin, no por argumentos: con 1000 ficheros la línea de órdenes pasaba del tope de Windows y `hash-object` fallaba. El puerto `Git` admite entrada por stdin en tres formas exactas y en ninguna otra: `cat-file` (F2-4b), `hash-object -w --no-filters --stdin-paths` (`Git::ejecutar_con_entrada`) y `update-index --add -z --index-info` solo con índice temporal de ruta absoluta (`Git::ejecutar_con_indice_y_entrada`). `update-ref --stdin` sigue prohibido. Un commit de estado lanza así un número fijo de procesos sea cual sea el número de ficheros (test `mil_ficheros_en_una_transaccion_con_procesos_acotados` de `estado_casos.rs`); el registro de la capa deja de mostrar esas rutas.
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-127/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-127/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-127/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
