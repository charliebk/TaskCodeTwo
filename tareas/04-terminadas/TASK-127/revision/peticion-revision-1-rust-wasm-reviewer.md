# Peticion de revision — TASK-127 (ronda 1)

- Tarea: TASK-127 — Escrituras de estado con muchos ficheros fallan en Windows (git hash-object)
- Rama revisada: fix/task-127-escrituras-de-estado-con-muchos-ficheros
- Rama base: develop
- Commit revisado (HEAD): 7d1963a0d80f19af3edc996e533d258f37d41a69
- Fecha: 2026-11-18
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
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
informe de esta ronda (informe-revision-1-rust-wasm-reviewer.md), sin borrar la
peticion.

## Contexto permitido

Lee solo esto: el enunciado y el plan de la tarea, y los ficheros de tu parte del diff.
Si necesitas algo mas, leelo y anotalo con su motivo en `## Lecturas fuera del
contexto permitido` de tu informe.

- tareas/03-en-revision/TASK-127/tarea.md
- tareas/03-en-revision/TASK-127/planificacion/
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/git.rs
- crates/tc2/tests/estado_casos.rs
- crates/tc2/tests/progreso.rs

## Commits a revisar (git log develop..HEAD)

````
7d1963a chore(TASK-127): autorrevision
22ffc96 fix(TASK-127): rutas e indice por stdin en commit-estado
43333c7 chore(TASK-127): tarea en curso
````

## Diff de tu dominio (5 fichero(s) de 6; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 44a0dc8..e3abb7b 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -8,6 +8,7 @@
 use std::path::{Component, Path, PathBuf};
 use std::sync::atomic::{AtomicUsize, Ordering};
 
+use super::puerto::{HASH_OBJECT_STDIN, UPDATE_INDEX_STDIN};
 use super::rama::error_de_fichero;
 use super::{Git, Paso, Subcomando, correr, exigir, git_fallo};
 use crate::contrato::{Informe, Resultado};
@@ -263,6 +264,44 @@ fn con_indice(
     }
 }
 
+/// `hash-object` (sin indice) o `update-index` (con `indice`) en su unica
+/// forma con entrada por stdin; un codigo distinto de 0 es `error`.
+fn con_entrada(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    sub: Subcomando,
+    indice: Option<&Path>,
+    entrada: &[u8],
+) -> Paso<Vec<u8>> {
+    let r = match indice {
+        None => git.ejecutar_con_entrada(dir, sub, HASH_OBJECT_STDIN, entrada),
+        Some(i) => git.ejecutar_con_indice_y_entrada(dir, sub, UPDATE_INDEX_STDIN, i, entrada),
+    };
+    let s = r.map_err(|e| {
+        Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(e.mensaje())
+    })?;
+    if s.codigo == 0 {
+        Ok(s.stdout)
+    } else {
+        Err(Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(
+                Mensaje::nuevo(&txt::GIT_FALLO)
+                    .con("subcomando", sub.nombre())
+                    .con(
+                        "error",
+                        s.stderr
+                            .lines()
+                            .find(|l| !l.trim().is_empty())
+                            .unwrap_or_default(),
+                    ),
+            ))
+    }
+}
+
 /// Construye un commit con el arbol de `base` mas `cambios` y `base` como
 /// padre (sin padre si `base` es `None`: la raiz de la rama huerfana). No
 /// mueve ninguna rama. `None` si el arbol queda igual que el de `base`.
@@ -288,34 +327,39 @@ pub(crate) fn construir_commit(
         con_indice(git, dir, op, Subcomando::ReadTree, &[b], &indice)?;
     }
     if !cambios.is_empty() {
-        let mut ficheros = Vec::with_capacity(cambios.len());
+        // Rutas y entradas van por stdin, no por argumentos: con muchos
+        // ficheros la linea de ordenes pasaria del tope de Windows (unos
+        // 32 K caracteres) y los procesos no crecen con el numero de cambios
+        // (TASK-127).
+        let mut rutas = String::new();
         for (i, c) in cambios.iter().enumerate() {
             let f = tmp.0.join(format!("blob-{i}"));
             std::fs::write(&f, c.contenido.as_bytes()).map_err(|e| {
                 Informe::nuevo(op, Resultado::Error).diagnostico(error_de_fichero(&f, &e))
             })?;
-            ficheros.push(texto(op, &f)?);
+            rutas.push_str(&texto(op, &f)?);
+            rutas.push('\n');
         }
-        let mut args = vec!["-w", "--no-filters", "--"];
-        args.extend(ficheros.iter().map(String::as_str));
-        let shas = exigir(git, dir, op, Subcomando::HashObject, &args)?;
+        let shas = con_entrada(git, dir, op, Subcomando::HashObject, None, rutas.as_bytes())?;
+        let shas = String::from_utf8_lossy(&shas);
         let shas: Vec<&str> = shas.lines().map(str::trim).collect();
         if shas.len() != cambios.len() {
             return Err(
                 Informe::nuevo(op, Resultado::Error).diagnostico(&txt::HASH_OBJECT_INCOMPLETO)
             );
         }
-        let entradas: Vec<String> = cambios
-            .iter()
-            .zip(&shas)
-            .map(|(c, sha)| format!("100644,{sha},{}", c.ruta))
-            .collect();
-        let mut args = vec!["--add"];
-        for e in &entradas {
-            args.push("--cacheinfo");
-            args.push(e);
+        let mut entradas = String::new();
+        for (c, sha) in cambios.iter().zip(&shas) {
+            entradas.push_str(&format!("100644 {sha}\t{}\0", c.ruta));
         }
-        con_indice(git, dir, op, Subcomando::UpdateIndex, &args, &indice)?;
+        con_entrada(
+            git,
+            dir,
+            op,
+            Subcomando::UpdateIndex,
+            Some(&indice),
+            entradas.as_bytes(),
+        )?;
     }
     let arbol = con_indice(git, dir, op, Subcomando::WriteTree, &[], &indice)?;
     let arbol = arbol.trim();
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index f031aa7..81364cb 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -288,9 +288,10 @@ impl Git {
         self.lanzar(dir, sub, args, Some(indice))
     }
 
-    /// Lanza `git cat-file <args>` en `dir` con `entrada` por stdin (lectura
-    /// en lote, F2-4b). Solo `cat-file`: es el unico que lo necesita y no
-    /// escribe nada.
+    /// Lanza `git <sub> <args>` en `dir` con `entrada` por stdin. Solo dos
+    /// formas: `cat-file` (lectura en lote, F2-4b) y exactamente
+    /// `hash-object -w --no-filters --stdin-paths` (escritura de blobs de la
+    /// rama de estado sin rutas en la linea de ordenes, TASK-127).
     pub fn ejecutar_con_entrada(
         &self,
         dir: &Path,
@@ -298,13 +299,53 @@ impl Git {
         args: &[&str],
         entrada: &[u8],
     ) -> Result<SalidaCruda, ErrorGit> {
-        if sub != Subcomando::CatFile {
+        let permitida = match sub {
+            Subcomando::CatFile => true,
+            Subcomando::HashObject => args == HASH_OBJECT_STDIN,
+            _ => false,
+        };
+        if !permitida {
             return Err(ErrorGit::Prohibido(
                 Mensaje::nuevo(&txt::SIN_ENTRADA).con("sub", sub.nombre()),
             ));
         }
+        self.lanzar_con_entrada(dir, sub, args, None, entrada)
+    }
+
+    /// Como [`Git::ejecutar_con_entrada`], con `GIT_INDEX_FILE=indice`: solo
+    /// exactamente `update-index --add -z --index-info` y con `indice`
+    /// absoluto (TASK-127). Las entradas van por stdin, no por argumentos.
+    pub fn ejecutar_con_indice_y_entrada(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+        indice: &Path,
+        entrada: &[u8],
+    ) -> Result<SalidaCruda, ErrorGit> {
+        if sub != Subcomando::UpdateIndex || args != UPDATE_INDEX_STDIN {
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::SIN_ENTRADA).con("sub", sub.nombre()),
+            ));
+        }
+        if !indice.is_absolute() {
+            return Err(ErrorGit::Prohibido(
+                Mensaje::nuevo(&txt::INDICE_RELATIVO).con("sub", sub.nombre()),
+            ));
+        }
+        self.lanzar_con_entrada(dir, sub, args, Some(indice), entrada)
+    }
+
+    fn lanzar_con_entrada(
+        &self,
+        dir: &Path,
+        sub: Subcomando,
+        args: &[&str],
+        indice: Option<&Path>,
+        entrada: &[u8],
+    ) -> Result<SalidaCruda, ErrorGit> {
         comprobar_argumentos(sub, args)?;
-        let mut cmd = self.comando(dir, sub, args, None);
+        let mut cmd = self.comando(dir, sub, args, indice);
         cmd.stdin(Stdio::piped())
             .stdout(Stdio::piped())
             .stderr(Stdio::piped());
@@ -406,6 +447,13 @@ impl Git {
     }
 }
 
+/// Unica forma de `hash-object` con entrada por stdin: una ruta por linea.
+pub const HASH_OBJECT_STDIN: &[&str] = &["-w", "--no-filters", "--stdin-paths"];
+
+/// Unica forma de `update-index` con entrada por stdin: entradas
+/// `<modo> <sha>\t<ruta>` terminadas en NUL, siempre contra un indice temporal.
+pub const UPDATE_INDEX_STDIN: &[&str] = &["--add", "-z", "--index-info"];
+
 /// Argumentos prohibidos dentro de los subcomandos permitidos (F1-4,
 /// decision 3). Se mira cada argumento entero, no subcadenas (H9). No se
 /// miran el texto de un mensaje (`-m <texto>` en `commit` y `tag`) ni las
@@ -633,13 +681,64 @@ mod tests {
             let r = git.ejecutar(Path::new("."), Subcomando::UpdateRef, ok);
             assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{ok:?}: {r:?}");
         }
-        // Entrada por stdin: solo `cat-file`.
+        // Entrada por stdin: `cat-file`, y `hash-object` y `update-index`
+        // solo en su forma exacta (TASK-127).
         for sub in [Subcomando::UpdateRef, Subcomando::Push, Subcomando::Show] {
             let r = git.ejecutar_con_entrada(Path::new("."), sub, &["--stdin"], b"x\n");
             assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{sub:?}: {r:?}");
         }
+        for malo in [
+            &["--stdin"][..],
+            &["-w", "--stdin-paths"][..],
+            &["-w", "--no-filters", "--stdin-paths", "x"][..],
+        ] {
+            let r = git.ejecutar_con_entrada(Path::new("."), Subcomando::HashObject, malo, b"x\n");
+            assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{malo:?}: {r:?}");
+        }
         let r = git.ejecutar_con_entrada(Path::new("."), Subcomando::CatFile, &["--batch"], b"");
         assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
+        let r = git.ejecutar_con_entrada(
+            Path::new("."),
+            Subcomando::HashObject,
+            HASH_OBJECT_STDIN,
+            b"",
+        );
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
+        // `update-index` por stdin: nunca sin indice temporal, ni relativo,
+        // ni con otra forma; `update-ref --stdin` sigue prohibido.
+        let r = git.ejecutar_con_entrada(
+            Path::new("."),
+            Subcomando::UpdateIndex,
+            UPDATE_INDEX_STDIN,
+            b"",
+        );
+        assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{r:?}");
+        let casos: &[(Subcomando, &[&str], &Path)] = &[
+            (
+                Subcomando::UpdateIndex,
+                UPDATE_INDEX_STDIN,
+                Path::new("relativo"),
+            ),
+            (Subcomando::UpdateIndex, &["--index-info"], &indice),
+            (Subcomando::UpdateIndex, &["--add", "--index-info"], &indice),
+            (Subcomando::UpdateRef, &["--stdin"], &indice),
+            (Subcomando::HashObject, HASH_OBJECT_STDIN, &indice),
+        ];
+        for (sub, args, ind) in casos {
+            let r = git.ejecutar_con_indice_y_entrada(Path::new("."), *sub, args, ind, b"");
+            assert!(
+                matches!(r, Err(ErrorGit::Prohibido(_))),
+                "{sub:?} {args:?}: {r:?}"
+            );
+        }
+        let r = git.ejecutar_con_indice_y_entrada(
+            Path::new("."),
+            Subcomando::UpdateIndex,
+            UPDATE_INDEX_STDIN,
+            &indice,
+            b"",
+        );
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
         // Tras `--` en push no hay rutas: se sigue mirando.
         let r = git.ejecutar(Path::new("."), Subcomando::Push, &["origin", "--", "+r:r"]);
         assert!(matches!(r, Err(ErrorGit::Prohibido(_))));
diff --git a/crates/tc2/src/i18n/catalogo/git.rs b/crates/tc2/src/i18n/catalogo/git.rs
index 107b1e4..141e563 100644
--- a/crates/tc2/src/i18n/catalogo/git.rs
+++ b/crates/tc2/src/i18n/catalogo/git.rs
@@ -7,8 +7,8 @@ crate::i18n::textos! {
         en: "argument not allowed for the layer: {argumento}",
     }
     SIN_ENTRADA {
-        es: "argumento prohibido para la capa: {sub} con entrada por stdin (solo cat-file)",
-        en: "argument not allowed for the layer: {sub} with stdin input (only cat-file)",
+        es: "argumento prohibido para la capa: {sub} con entrada por stdin (solo cat-file, hash-object -w --no-filters --stdin-paths y update-index --add -z --index-info con indice temporal)",
+        en: "argument not allowed for the layer: {sub} with stdin input (only cat-file, hash-object -w --no-filters --stdin-paths and update-index --add -z --index-info with a temporary index)",
     }
     CAT_FILE_INCOMPLETO {
         es: "git cat-file --batch devolvio una salida que no se entiende",
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

## Tests del diff (2 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/estado_casos.rs" ":(literal)crates/tc2/tests/progreso.rs"
````

````
crates/tc2/tests/estado_casos.rs | 77 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/progreso.rs     |  6 ++----
 2 files changed, 79 insertions(+), 4 deletions(-)
````
