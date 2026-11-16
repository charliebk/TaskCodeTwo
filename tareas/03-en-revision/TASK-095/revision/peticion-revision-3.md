# Peticion de revision — TASK-095 (ronda 3)

- Tarea: TASK-095 — F1-19 ADR memoria del proyecto: indice de contexto, etiquetas y busqueda
- Rama revisada: feature/task-095-f1-19-adr-memoria-del-proyecto-indice-de
- Rama base: develop
- Commit revisado (HEAD): 1e73ccf4fc6afe95b42d1e8d87a46f8febf857bb
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Modelo sugerido para esta ronda: haiku (verificacion de correcciones; la tarea usa sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-095 (criterios de aceptacion y plan)
- Revision incremental: solo los cambios desde 63e9c84280307befcf399d9343db83e84189c140 (el commit revisado en la ronda anterior)
- Alcance: verificacion acotada al diff de las correcciones (motivo: la ronda anterior pidio cambios (3 hallazgo(s)))

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
informe de esta ronda (informe-revision-3.md), sin borrar la peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-095/tarea.md
- tareas/03-en-revision/TASK-095/planificacion/
- docs/adr/F1-19-memoria-del-proyecto.md

## Hallazgos de la ronda 2 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.

| ID | Severidad | Estado | Fichero | Informe |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 1 «Dato real», decision 6, tabla RF-78/RF-79) | informe-revision-2.md |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md (PoC 2 «6 ms», decision 2, tabla F10-6) | informe-revision-2.md |
| IMP-3 | IMPORTANTE | corregido | docs/adr/F1-19-memoria-del-proyecto.md:111-117 y 254-256 (decision 3, PoC 1, tabla RF-78) | informe-revision-2.md |

## Commits a revisar (git log 63e9c84280307befcf399d9343db83e84189c140..HEAD)

````
1e73ccf docs(TASK-095): ADR F1-19 corrige IMP-3 de la revision 2
df2cf63 chore(TASK-095): peticion de revision ronda 2
````

## Diff desde la ronda anterior (git diff 63e9c84280307befcf399d9343db83e84189c140..HEAD)

````diff
diff --git a/docs/adr/F1-19-memoria-del-proyecto.md b/docs/adr/F1-19-memoria-del-proyecto.md
index 69199e9..0bd92ed 100644
--- a/docs/adr/F1-19-memoria-del-proyecto.md
+++ b/docs/adr/F1-19-memoria-del-proyecto.md
@@ -103,18 +103,26 @@ Lectura:
   son contrato**: la huella es la del volcado canónico.
 - **La atribución solo por asunto se queda corta** (hallazgo IMP-1 de la
   revisión). De los 482 commits de `develop`, la PoC no atribuye 120 a ninguna
-  de las 30 terminadas. Solo 3 son de tareas no terminadas. El grueso son
-  commits del trabajo de las propias tareas sin `(TASK-NNN)` en el asunto:
-  18 `merge(feature): feature/task-N…`, 16 `chore(taskctl): import…` y unos
-  34 `docs(adr): F1-N corrige hallazgos…`. Consecuencia: de las tareas ADR,
-  solo 2 tienen su `docs/adr/F1-*` entre los ficheros tocados.
+  de las 30 terminadas. Solo 2 llevan `(TASK-NNN)` de una tarea no terminada.
+  El grueso son commits del trabajo de las propias tareas sin `(TASK-NNN)` en
+  el asunto: 30 `merge(feature): feature/task-N…`, 36
+  `docs(adr): F1-N corrige hallazgos…` y 16 `chore(taskctl): import…`.
+  Consecuencia: de las tareas ADR, solo 2 tienen su `docs/adr/F1-*` entre los
+  ficheros tocados.
 - Medido aparte con `git` sobre el mismo `e1bf456`, se añade la regla del
   rango del merge: todo commit de `merge^1..merge^2` de un merge cuyo asunto
-  nombra `feature/task-NNN`. Con ella quedan **467 de 482** commits
-  atribuidos y 15 sin atribuir, todos `docs:` del plan o del ROADMAP. Las 18
+  nombra `feature/task-NNN`, más el propio merge, con prioridad para el
+  asunto. Los dos recuentos, mirando solo el asunto (no el cuerpo):
+  - contando solo las 30 tareas terminadas: **447 de 482** atribuidos y 35
+    sin atribuir;
+  - contando cualquier tarea, como dice la decisión 3: **449 de 482** y 33 sin
+    atribuir.
+  Los 33 son 24 `chore:` (16 imports de `taskctl`, el alta de tareas, la
+  estructura inicial, la config…) y 9 `docs:` del plan o del ROADMAP. Las 18
   tareas ADR con fichero de ADR quedan **todas** con su ADR. La 19.ª, TASK-001
   («F1-0 Crear TaskCodeTwo»), no tiene ADR. Esta regla no la ejecuta la PoC;
-  la fija la decisión 3.
+  la fija la decisión 3. (Hallazgo IMP-3: la versión anterior citaba 467,
+  sacado de un `git log --grep` que también busca en el cuerpo del mensaje.)
 
 ### PoC 2 — merge en la plataforma sin hooks (criterio 2)
 
@@ -252,7 +260,7 @@ No verificado (diseño sin PoC):
         mismo también es de esa tarea;
      3. si nada lo atribuye, va a `commit_huerfano` y se cuenta.
      Un commit que cumple 1 para una tarea y 2 para otra se queda con la del
-     asunto. Medido en `e1bf456`: 467 de 482 atribuidos frente a 362 con la
+     asunto. Medido en `e1bf456`: 449 de 482 atribuidos frente a 364 con la
      regla 1 sola.
    - Lectura en lote:
      - `cat-file --batch`, un proceso para todos los blobs y no uno por
@@ -393,7 +401,7 @@ No verificado (diseño sin PoC):
 | RR-20, verdad en git e índice no versionado | A, decisiones 1 y 2 | Índice en `<git-common-dir>`; la PoC lee solo objetos de git |
 | RR-20, nunca datos viejos sin decirlo | F4, decisión 4 | PoC 2: con el TTL caducado aparece TASK-999; dentro del TTL, la marca «hace 17 s» |
 | RF-77, mismo resultado con los mismos SHA | Decisiones 3 y 6 | PoC 1: huella `9527d653…` dos veces; un árbol sucio con `autocrlf` no la cambia |
-| RF-78, conversor de tareas | Decisión 3 | PoC 1: 30 tareas; solo por asunto, 362 commits atribuidos y 120 huérfanos. Con la regla del rango del merge, medida con `git`, 467 de 482 |
+| RF-78, conversor de tareas | Decisión 3 | PoC 1: 30 tareas; solo por asunto, 362 commits atribuidos y 120 huérfanos. Con la regla del rango del merge, medida con `git`: 447 de 482 contando solo las terminadas, 449 contando cualquier tarea |
 | RF-79, ficheros tocados | Decisión 3 y 6 | PoC 1: 1307 filas tarea-fichero, pero **solo 2 tareas ADR con su ADR**. Con la regla del rango, 18 de 18 (medido con `git`; sin ejecutar en la PoC). A/M/D/R queda para F10-4 |
 | RF-80, conversor de engram | A, decisión 8 | Diseño. La base real (146/116) se verifica en F10-5 |
 | RF-81, `tc2 buscar` único con JSON | Decisión 4 | PoC 2: búsqueda FTS5 con BM25; de 6 a 7 ms dentro del proceso y dentro del TTL |
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff 63e9c84280307befcf399d9343db83e84189c140..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/03-en-revision/TASK-095/revision/informe-revision-1.md  |  91 +++++++++++++++++++++++++++++++---
 tareas/03-en-revision/TASK-095/revision/informe-revision-2.md  |  21 ++++++++
 tareas/03-en-revision/TASK-095/revision/peticion-revision-2.md | 345 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 tareas/03-en-revision/TASK-095/tarea.md                        |   5 +-
 4 files changed, 453 insertions(+), 9 deletions(-)
````
