# Peticion de revision — TASK-033 (ronda 2)

- Tarea: TASK-033 — F3-6 start: toma atomica de la tarea, rama y worktree desde develop
- Rama revisada: feature/task-033-f3-6-start-toma-atomica-de-la-tarea-rama
- Rama base: develop
- Commit revisado (HEAD): b7234440ee908ed994dce8704e5fa8e8d34dd65c
- Fecha: 2026-11-17
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
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
informe de esta ronda (informe-revision-2-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-033/tarea.md
- tareas/03-en-revision/TASK-033/planificacion/
- crates/tc2/src/toma/mod.rs
- crates/tc2/tests/toma.rs

## Hallazgos de la ronda 1 pendientes de verificar

Comprueba que cada uno queda resuelto por los cambios de esta ronda, y que la
correccion no abre otro fallo: es justo donde se cuelan.


Tabla de hallazgos ausente o ilegible en: informe-revision-1-code-quality-reviewer.md, informe-revision-1-rust-wasm-reviewer.md. Lee esos informes enteros.

## Commits a revisar (git log b769913f99fc6cdbc738184abbd86a1ecb8b7606..HEAD)

````
b723444 fix(TASK-033): carpeta del worktree ocupada se rechaza antes de publicar la toma
ded3359 chore(TASK-033): peticion de revision ronda 1
````

## Diff de tu dominio (2 fichero(s) de 3; git diff b769913f99fc6cdbc738184abbd86a1ecb8b7606..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/toma/mod.rs b/crates/tc2/src/toma/mod.rs
index d298882..415937e 100644
--- a/crates/tc2/src/toma/mod.rs
+++ b/crates/tc2/src/toma/mod.rs
@@ -17,7 +17,8 @@
 //! esperar al push); la base valida (`origin/develop`, o para `hotfix` el
 //! ultimo tag de `origin/main` o su punta, D4; `release` se rechaza);
 //! `comprobar-id` (una rama `*/task-nnn-*` que no es de quien la lanza,
-//! `ocupada`); la transaccion; la rama y el worktree.
+//! `ocupada`); la carpeta del worktree libre (`carpeta-ocupada`); la
+//! transaccion; la rama y el worktree.
 //!
 //! Lanzada desde un worktree de tarea escribe el estado en el principal (lo
 //! localiza la transaccion) y crea el worktree junto al principal.
@@ -173,6 +174,9 @@ fn paso(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
     if !c.resultado.es_exito() {
         return Err(c);
     }
+    // 5b. La carpeta del worktree, libre: si no, la toma quedaria publicada
+    // sin poder crear el worktree.
+    carpeta_libre(g, &principal, id, &rama)?;
     // 6. La toma, por el punto unico de escritura.
     let personas = crate::roadmap::personas(g, e.dir);
     let generar = crate::roadmap::generador(&personas);
@@ -228,6 +232,28 @@ fn base_de(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<String, In
         .map_or_else(|| MAIN.to_owned(), str::to_owned))
 }
 
+/// La carpeta `.taskcodetwo/worktrees/TASK-NNN` no existe o es el worktree
+/// registrado de `rama`; si la ocupa otra cosa, `rechazada
+/// motivo=carpeta-ocupada` antes de escribir estado.
+fn carpeta_libre(g: &Git, principal: &Path, id: &str, rama: &str) -> Result<(), Informe> {
+    let wt = principal.join(git::CARPETA_WORKTREES).join(id);
+    if !wt.exists() {
+        return Ok(());
+    }
+    let registrado = git::worktrees(g, principal, NOMBRE)?
+        .iter()
+        .any(|w| w.rama.as_deref() == Some(rama) && git::misma_ruta(&w.ruta, &wt));
+    if registrado {
+        return Ok(());
+    }
+    Err(rechazada(
+        "carpeta-ocupada",
+        Mensaje::nuevo(&crate::i18n::catalogo::git::CARPETA_OCUPADA)
+            .con("ruta", git::worktree_relativo(id)),
+    )
+    .con("rama", rama))
+}
+
 /// La rama y el worktree de la tarea.
 struct Destino<'a> {
     g: &'a Git,
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

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff b769913f99fc6cdbc738184abbd86a1ecb8b7606..HEAD -- ":(literal)crates/tc2/tests/toma.rs"
````

````
crates/tc2/tests/toma.rs | 67 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 67 insertions(+)
````
