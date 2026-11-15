# Peticion de revision — TASK-019 (ronda 1)

- Tarea: TASK-019 — F2-4 Reserva de IDs por push atomico
- Rama revisada: feature/task-019-f2-4-reserva-de-ids-por-push-atomico
- Rama base: develop
- Commit revisado (HEAD): f7f9446522807d5cea787a20dc1286abdf525a6d
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-019 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-019/tarea.md
- tareas/03-en-revision/TASK-019/planificacion/
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
f7f9446 chore(TASK-019): autorrevision
c121de2 fix(TASK-019): rechazar U+2028 y U+2029 en el titulo
a1723eb feat(TASK-019): alta de tareas con reserva de ID por push atomico (tc2 new)
0ddcab1 chore(TASK-019): tarea en curso
````

## Diff de tu dominio (1 fichero(s) de 5; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index d4b8f95..0b7bb52 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -550,6 +550,10 @@ con su evidencia en los tests de `crates/tc2/tests/estado_*.rs`:
    ref se movió), igual que un non-fast-forward, y se trata como tal. El
    resto de `[remote rejected]` es `denegada`. Observado en el test de dos
    clones a la vez; antes de esta distinción daba un falso `denegada`.
+   **Adenda F2-4 (TASK-019):** también es contención `[remote rejected]
+   (reference already exists)`, que recibe el segundo de dos clones que
+   crean a la vez una ref que no existía: la rama de estado en el primer
+   `new` de un repo. Lo destapó el E2E de dos `tc2 new` simultáneos.
 4. **Cambio de comportamiento en `push`, `actualizar-rama` y
    `tag-anotado`** (F2-2): un rechazo del remoto pasa de reintentarse y
    acabar en `agotada` a `denegada` en el primer intento. Los fallos de
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-019/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-019/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-019/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````
