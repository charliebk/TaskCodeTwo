# Peticion de revision — TASK-018 (ronda 1)

- Tarea: TASK-018 — F2-3 Rama de estado
- Rama revisada: feature/task-018-f2-3-rama-de-estado
- Rama base: develop
- Commit revisado (HEAD): 4f80989fc93337d12966d31c9be87260371a8ddd
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: code-quality-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-018 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-018/tarea.md
- tareas/03-en-revision/TASK-018/planificacion/
- docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
- docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
- docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md

## Commits a revisar (git log develop..HEAD)

````
4f80989 chore(TASK-018): criterios y autorrevision
d78931e fix(TASK-018): no perder el error al borrar la ref temporal
24fd49c feat(TASK-018): rama de estado con escritura transaccional
a14ad39 chore(TASK-018): tarea en curso
````

## Diff de tu dominio (3 fichero(s) de 23; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md b/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
index 14f4a5f..ffe7761 100644
--- a/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
+++ b/docs/adr/F1-1-carpeta-fija-y-rama-de-estado.md
@@ -169,3 +169,24 @@ borrarlo a mano.
   estados de P1 bastan a los proyectos con `05-archivadas` (RF-71): quedan
   como preguntas abiertas para F1-2, F1-11 y F2-3. Este ADR se planificó con
   un solo rol (arquitectura).
+
+## Adenda (F2-3, TASK-018) — Git 2.42 deja de ser el mínimo
+
+La decisión 2 fijaba Git 2.42 por `worktree add --orphan`. F2-3 crea la rama
+huérfana con un commit raíz hecho con `commit-tree` (sin padre) y publicado
+por la misma transacción que cualquier escritura (`estado::montar`); después
+la monta con un `worktree add` normal. No necesita `--orphan`, así que la
+rama de estado ya no exige 2.42. Lo que sí queda sin verificar es Git
+anterior a 2.56 en general (lo anota F1-4). Además:
+
+- Si dos clones la crean a la vez, el segundo recibe non-fast-forward,
+  re-ejecuta, la ve creada y solo la monta.
+- La carpeta `tareas/` borrada a mano o con `git clean -ffdx` se vuelve a
+  montar con `montar` (poda el worktree y lo añade de nuevo); lo que no
+  estaba publicado en ella se pierde, como se dijo en «Requisitos y
+  evidencia».
+- `tareas/` ocupada por una carpeta normal (un repo de la 0.16) da `rechazada
+  motivo=tareas-ocupada`: la migración es de F9-2.
+- `tareas/` se añade a `info/exclude` del clon para que el worktree
+  principal no la vea como sin seguimiento; el `.gitignore` de las ramas de
+  código lo pone F3-1.
diff --git a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
index ae277ab..09576f2 100644
--- a/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
+++ b/docs/adr/F1-2-escrituras-atomicas-ids-y-toma.md
@@ -381,3 +381,37 @@ resuelven en F2-3 y F3-10 salvo que se diga otra cosa:
 - **Rendimiento**: contención medida solo con 3 clones en una máquina; la
   ráfaga de altas seguidas de un mismo clon es el peor caso (en uso real
   sería un `import`, una sola operación).
+
+## Adenda (F2-3, TASK-018) — preguntas abiertas resueltas
+
+El punto único de escritura está en `crates/tc2/src/estado/` y se prueba en
+`crates/tc2/tests/estado_concurrencia.rs` y `estado_casos.rs`.
+
+| Pregunta abierta | Respuesta de F2-3 | Evidencia |
+|---|---|---|
+| Error de autenticación confundido con «sin red» | `clasificar_fallo_remoto` separa red caída (`sinred`, 4), credenciales y remoto inexistente (`denegada`, 8) y rama inexistente; lo no reconocido es `sinred` | test unitario con textos de Git y de GitHub; F2-2 `fetch` con una ruta que no es un repo |
+| Hook o regla que rechaza el push | `denegada` (8) en el primer intento, sin reintento | test con `pre-receive` que rechaza: un solo push |
+| Cerrojo local huérfano | cerrojo del sistema operativo sobre `.git/taskcode-estado.lock` (`File::try_lock`): si el proceso muere, el sistema lo suelta; espera máxima 60 s y después `agotada motivo=cerrojo` | test de dos procesos del mismo clon sin remoto (solo los separa el cerrojo) |
+| Worktree `tareas/` desmontado o borrado | la transacción no depende de `tareas/` (lee y escribe commits); tras publicar adelanta la rama local y dice `tareas=sin-montar`; `montar` la vuelve a montar | test de montaje |
+| Pendiente sin red frente a un cambio de otro | al re-ejecutarla con red, si ya no procede, su ref pasa a `refs/taskcode/rechazadas/NNNNNN` y la línea dice `rechazadas=N`; el contenido sigue en un objeto de git | test sin red: `crear-unico` que otro hizo antes |
+| Cerrojo con varios procesos del mismo clon (No verificado de la PoC) | verificado | mismo test del cerrojo |
+
+Cómo quedan las pendientes (D2b): una operación con `exige_red()=false` sin
+red se guarda como commit en `refs/taskcode/pendientes/NNNNNN`, con la
+operación anotada en el mensaje (`Operacion: …`), y devuelve `sinred
+pendiente=si`. La siguiente transacción con red no publica ese commit: la
+reconstruye con el catálogo de operaciones y la **re-ejecuta** sobre el estado
+nuevo, en el mismo commit que la operación nueva. Si la operación nueva se
+rechaza, las pendientes no se publican y siguen ahí.
+
+**Divergencia con la decisión 2**: no hay commit local que descartar tras un
+rechazo (el commit sale de `commit-tree`, F1-4 divergencia 2), y el cerrojo se
+suelta durante la espera para que otro proceso del clon avance.
+
+**Valores por defecto**: se mantienen 5 intentos y base 2 s (`Politica`). Los
+tests usan base corta. No se ha medido la duración de una transacción del CLI
+contra la plataforma: sigue en F9-4.
+
+**Contención en el remoto** (nuevo): dos pushes simultáneos al mismo remoto
+local pueden dar `[remote rejected] (cannot lock ref …)` en vez de
+non-fast-forward; se trata como contención (ver la adenda de F1-4).
diff --git a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
index fa81bdb..d4b8f95 100644
--- a/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
+++ b/docs/adr/F1-4-operaciones-git-y-contrato-de-salida.md
@@ -521,3 +521,39 @@ Este ADR se planificó con un solo rol (arquitectura). Quedan sin cubrir:
   plataforma; aquí solo se prueba el contrato.
 - **Línea base medida en tareas de ADR**, no de código: puede quedarse corta
   frente a ToolsaWeb y OpenDockTrading; la recalibran F8-2 y F9-7.
+
+## Adenda (F2-3, TASK-018) — lista permitida ampliada y códigos fijados
+
+Lo que la implementación de la rama de estado (F2-3) cambia de este ADR,
+con su evidencia en los tests de `crates/tc2/tests/estado_*.rs`:
+
+1. **Lista permitida ampliada** (`git/puerto.rs`) para `commit-estado`:
+   `commit-tree`, `hash-object`, `ls-tree`, `show`, `read-tree`,
+   `update-index`, `write-tree` y `update-ref`. No relaja la decisión 3:
+   - `read-tree`, `update-index` y `write-tree` solo se lanzan con un índice
+     temporal (`Git::ejecutar_con_indice`, ruta absoluta); con `ejecutar`
+     normal se rechazan sin lanzar nada, así que nunca tocan el índice del
+     repo. `read-tree -u|-m|--reset|--prefix` sigue prohibido.
+   - `update-ref` solo con las formas `<ref> <nuevo> [<viejo>]` y
+     `-d <ref> <viejo>`, y la ref bajo `refs/taskcode/` (pendientes,
+     rechazadas y una ref temporal). Nunca `refs/heads/*`, nunca `--stdin`.
+   - Las ramas solo se adelantan por fast-forward: `merge --ff-only` en
+     `tareas/` o `fetch . <ref>:<rama>` sin `+`.
+2. **Códigos fijados** (tabla de la decisión 5, ya no propuesta): se añade
+   **8 `denegada`**: el remoto deniega la operación por credenciales,
+   permisos, un hook o una regla, o la URL no es un repositorio. No se
+   reintenta; `motivo=credenciales|remoto-rechaza|remoto-no-existe`. Resuelve
+   las preguntas abiertas de «token caducado» y «push rechazado por reglas».
+3. **`[remote rejected]` ya no es siempre un rechazo**: con dos pushes
+   simultáneos al mismo remoto, el perdedor recibe `[remote rejected]
+   (cannot lock ref …: is at X but expected Y)`. Eso es **contención** (la
+   ref se movió), igual que un non-fast-forward, y se trata como tal. El
+   resto de `[remote rejected]` es `denegada`. Observado en el test de dos
+   clones a la vez; antes de esta distinción daba un falso `denegada`.
+4. **Cambio de comportamiento en `push`, `actualizar-rama` y
+   `tag-anotado`** (F2-2): un rechazo del remoto pasa de reintentarse y
+   acabar en `agotada` a `denegada` en el primer intento. Los fallos de
+   transporte (red caída, corte de conexión) siguen reintentándose y acaban en
+   `sinred`. El test de F2-2 que simulaba lo transitorio con un hook que
+   rechaza una vez pasa a simularlo con un `receive-pack` que muere la
+   primera vez.
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-018/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-018/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-018/tarea.md                                    | 14 ++++++++++----
 3 files changed, 10 insertions(+), 4 deletions(-)
````
