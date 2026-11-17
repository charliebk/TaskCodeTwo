# Peticion de revision — TASK-033 (ronda 2)

- Tarea: TASK-033 — F3-6 start: toma atomica de la tarea, rama y worktree desde develop
- Rama revisada: feature/task-033-f3-6-start-toma-atomica-de-la-tarea-rama
- Rama base: develop
- Commit revisado (HEAD): b7234440ee908ed994dce8704e5fa8e8d34dd65c
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-033 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde b769913f99fc6cdbc738184abbd86a1ecb8b7606 (el commit revisado en la ronda anterior)
- Alcance: completa (motivo: un informe de la ronda anterior no tiene tabla de hallazgos legible)

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
informe de esta ronda (informe-revision-2-code-quality-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-033/tarea.md
- tareas/03-en-revision/TASK-033/planificacion/
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.


Tabla de hallazgos ausente o ilegible en: informe-revision-1-code-quality-reviewer.md, informe-revision-1-rust-wasm-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log b769913f99fc6cdbc738184abbd86a1ecb8b7606..HEAD)

````
b723444 fix(TASK-033): carpeta del worktree ocupada se rechaza antes de publicar la toma
ded3359 chore(TASK-033): peticion de revision ronda 1
````

## Diff de tu dominio (1 fichero(s) de 3; git diff b769913f99fc6cdbc738184abbd86a1ecb8b7606..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index 6b87a86..a708420 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -6,7 +6,7 @@
 - Depende de: [ADR F1-2](F1-2-escrituras-atomicas-ids-y-toma.md) (punto único de escritura, re-ejecución en vez de rebase, códigos 0/3/4/5) y [ADR F1-3](F1-3-flujo-de-ramas-worktrees-release.md) (ramas, worktrees, ciclo por comando y tabla de scripts de Git-Flow)
 - Decide: Carlos, con la consigna «la opción recomendada» (D1a, D2a, D3a, D4a, D5a del plan de TASK-005)
 - Adenda (2026-11-17, TASK-032, F3-5): entra `prs-abiertos` en la lista cerrada (decisión 2) para `tc2 board` (D2 de TASK-032, aceptada por Carlos el 2026-11-16); detalle de llamadas en la [adenda de F1-13](F1-13-adaptador-de-plataforma.md). `board` y `status`, como `roadmap`, escriben avisos de una línea en stderr con código 0; el canal fiable son las claves de la línea y del JSON
-- Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
+- Adenda (2026-11-17, TASK-033, F3-6): `start [--json] TASK-NNN` compone `fetch`, `validar-base`, `comprobar-id`, `commit-estado`, `crear-rama-y-worktree` y `reanudar` sin operaciones nuevas (la base de un `hotfix` es el último tag por versión de `refs/tags/` contenido en `origin/main`, leído con `for-each-ref --merged`, o su punta). Línea de éxito: `start ok id=… rama=… worktree=… base=… estado=<commit de la toma> asignado=<login>`; dueño con la tarea empezada: `sin-cambios` o `ok creado=rama,worktree` sin escribir estado. Motivos de `rechazada` (3): `no-existe`, `formato-1` (con `arreglo=tc2 actualizar`), `ilegible`, `tomada` (con `asignado=<login>`), `asignado-heredado`, `terminada`, `descartada`, `estado-no-valido`, `dependencias-pendientes` (con `dependencias=`), `rama-invalida`, `tipo-release`, `ocupada` (con `ramas=`), `carpeta-ocupada` (la carpeta del worktree existe y no es el worktree registrado de la rama; se comprueba antes de publicar la toma, sin `estado=`) e `identidad`; `uso` (2): `sin-id`, `id-invalido`, `argumentos-de-mas`; sin red, `sinred pendiente=no` (4)
 - Adenda (2026-11-16, TASK-116): la rama de estado, el cerrojo, las refs propias y la carpeta de worktrees que nombra este ADR se leen con los nombres de la [adenda de TASK-116 en F1-12](F1-12-actualizacion-entre-versiones.md#adenda-de-task-116-2026-11-16-todos-los-nombres-de-la-huella)
 
 ## Contexto
````

## Excluido del diff (7 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff b769913f99fc6cdbc738184abbd86a1ecb8b7606..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{02-en-curso => 03-en-revision}/TASK-033/planificacion/brainstorm/peticion-plan-1.md |    0
 tareas/{02-en-curso => 03-en-revision}/TASK-033/planificacion/plan-final.md                 |    0
 tareas/03-en-revision/TASK-033/revision/informe-revision-1-code-quality-reviewer.md         |   21 ++
 tareas/03-en-revision/TASK-033/revision/informe-revision-1-rust-wasm-reviewer.md            |   21 ++
 tareas/03-en-revision/TASK-033/revision/peticion-revision-1-code-quality-reviewer.md        |   84 ++++++
 tareas/03-en-revision/TASK-033/revision/peticion-revision-1-rust-wasm-reviewer.md           | 1620 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/{02-en-curso => 03-en-revision}/TASK-033/tarea.md                                    |   12 +-
 7 files changed, 1756 insertions(+), 2 deletions(-)
````
