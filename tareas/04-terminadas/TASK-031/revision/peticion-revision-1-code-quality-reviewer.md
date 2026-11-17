# Peticion de revision — TASK-031 (ronda 1)

- Tarea: TASK-031 — F3-4 tc2 roadmap: la vista para todos
- Rama revisada: feature/task-031-f3-4-roadmap-la-vista-para-todos
- Rama base: develop
- Commit revisado (HEAD): 8a54049c717b71525d0a180c8e00676686271db0
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-031 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-031/tarea.md
- tareas/03-en-revision/TASK-031/planificacion/
- .gitattributes
- crates/tc2/tests/fixtures/roadmap/referencia.md

## Commits a revisar (git log develop..HEAD)

````
8a54049 chore(TASK-031): resultado y autorrevision
3a1dd0f refactor(TASK-031): un solo nombre para ROADMAP.md y ajustes de revision
84954b4 test(TASK-031): integracion de roadmap para C1 a C6
5802680 feat(TASK-031): orden tc2 roadmap con --comparar y --escribir
1527267 feat(TASK-031): new e import regeneran el ROADMAP en la transaccion
0b143f4 feat(TASK-031): generador puro del ROADMAP con Mermaid por hito
a17896a refactor(TASK-031): situacion comun y zona manual estricta
7f20b12 chore(TASK-031): tarea en curso
````

## Diff de tu dominio (2 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/.gitattributes b/.gitattributes
index d3e48a6..2ce8608 100644
--- a/.gitattributes
+++ b/.gitattributes
@@ -10,3 +10,6 @@ Cargo.lock text eol=lf
 crates/tc2/esquemas/*.json text eol=lf
 # Evidencia de mediciones (TASK-113): se compara por sha256 con el original.
 docs/mediciones/** -text
+# Salidas de referencia del ROADMAP (TASK-031): mezclan CRLF (zona manual)
+# y LF (lo generado) y se comparan byte a byte.
+crates/tc2/tests/fixtures/roadmap/** -text
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-031/tarea.md                                                     | 44 --------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-031/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-031/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-031/tarea.md                                                      | 86 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 86 insertions(+), 44 deletions(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/fixtures/roadmap/referencia.md"
````

````
crates/tc2/tests/fixtures/roadmap/referencia.md | 118 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 118 insertions(+)
````
