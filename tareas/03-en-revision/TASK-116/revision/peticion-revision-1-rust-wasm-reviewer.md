# Peticion de revision — TASK-116 (ronda 1)

- Tarea: TASK-116 — Nombres de la 2.0 alineados con F1-12 D1a
- Rama revisada: fix/task-116-nombres-de-la-2-0-alineados-con-f1-12-d1
- Rama base: develop
- Commit revisado (HEAD): 92900522ace4f8328546b838b04358cb15a3d298
- Fecha: 2026-11-16
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-116 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-116/tarea.md
- tareas/03-en-revision/TASK-116/planificacion/
- crates/tc2/src/estado/cerrojo.rs
- crates/tc2/src/estado/lectura.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/pendientes.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/worktree.rs
- crates/tc2/src/huella.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/lib.rs
- crates/tc2/tests/alta.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/convivencia_016.rs
- crates/tc2/tests/estado_bordes.rs
- crates/tc2/tests/estado_casos.rs
- crates/tc2/tests/estado_concurrencia.rs
- crates/tc2/tests/git_binario.rs
- crates/tc2/tests/git_casos_limite.rs
- crates/tc2/tests/git_ciclo.rs
- crates/tc2/tests/git_conflicto.rs
- crates/tc2/tests/git_push.rs

## Commits a revisar (git log develop..HEAD)

````
9290052 docs(TASK-116): autorrevision con rust-wasm-reviewer y code-quality-reviewer
03c6f87 test(TASK-116): la huella de la 0.16 queda intacta tras new y start
7f57135 fix(TASK-116): nombres de la 2.0 bajo taskcodetwo segun F1-12 D1a
4e0a8d3 docs(TASK-116): adenda de nombres en F1-12 y cabeceras de F1-2, F1-3 y F1-4
3d3a7f5 refactor(TASK-116): huella.rs como unica fuente de los nombres de la 2.0
7b3b49e chore(TASK-116): tarea en curso
````

## Diff de tu dominio (26 fichero(s) de 31; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/estado/cerrojo.rs b/crates/tc2/src/estado/cerrojo.rs
index 0230e75..a6fda43 100644
--- a/crates/tc2/src/estado/cerrojo.rs
+++ b/crates/tc2/src/estado/cerrojo.rs
@@ -1,4 +1,5 @@
-//! Cerrojo local del clon (`.git/taskcode-estado.lock`, ADR F1-2): la
+//! Cerrojo local del clon (`.git/taskcodetwo/estado.lock`, ADR F1-2 con el
+//! nombre de [`crate::huella`]): la
 //! persona y sus agentes del mismo clon no escriben la rama de estado a la
 //! vez.
 //!
@@ -11,8 +12,8 @@ use std::fs::{File, OpenOptions, TryLockError};
 use std::path::Path;
 use std::time::{Duration, Instant};
 
-/// Nombre del fichero del cerrojo en la carpeta comun del repo.
-pub const FICHERO: &str = "taskcode-estado.lock";
+/// Nombre del fichero del cerrojo en la carpeta propia de la comun.
+pub const FICHERO: &str = crate::huella::FICHERO_CERROJO;
 
 /// Cerrojo tomado; se suelta al soltar el valor.
 #[derive(Debug)]
@@ -31,13 +32,15 @@ pub enum ErrorCerrojo {
 
 impl Cerrojo {
     /// Toma el cerrojo de la carpeta comun `comun`, esperando como mucho
-    /// `espera`.
+    /// `espera`. El fichero vive en la carpeta propia de la 2.0, que se
+    /// crea si no existe.
     pub fn tomar(comun: &Path, espera: Duration) -> Result<Self, ErrorCerrojo> {
+        let dir = crate::huella::dir_propio(comun).map_err(ErrorCerrojo::Io)?;
         let fichero = OpenOptions::new()
             .create(true)
             .truncate(false)
             .write(true)
-            .open(comun.join(FICHERO))
+            .open(dir.join(FICHERO))
             .map_err(ErrorCerrojo::Io)?;
         let limite = Instant::now() + espera;
         loop {
@@ -54,3 +57,30 @@ impl Cerrojo {
         }
     }
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// C2: el cerrojo se escribe en `<comun>/taskcodetwo/estado.lock`, nunca
+    /// suelto en la carpeta comun.
+    #[test]
+    fn el_cerrojo_vive_en_la_carpeta_propia() -> std::io::Result<()> {
+        let comun = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("cerrojo-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&comun);
+        std::fs::create_dir_all(&comun)?;
+        let tomado = Cerrojo::tomar(&comun, Duration::from_millis(100));
+        assert!(tomado.is_ok(), "{tomado:?}");
+        assert!(comun.join("taskcodetwo").join("estado.lock").is_file());
+        let mut sueltos = Vec::new();
+        for e in std::fs::read_dir(&comun)? {
+            sueltos.push(e?.file_name());
+        }
+        assert_eq!(sueltos, ["taskcodetwo"]);
+        drop(tomado);
+        let _ = std::fs::remove_dir_all(&comun);
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/estado/lectura.rs b/crates/tc2/src/estado/lectura.rs
index e1b5fde..bc02b6b 100644
--- a/crates/tc2/src/estado/lectura.rs
+++ b/crates/tc2/src/estado/lectura.rs
@@ -128,7 +128,7 @@ impl Estado for Vista<'_> {
 }
 
 /// Vista del estado tal como lo conoce este clon: lo ultimo traido de
-/// `origin/taskcode/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
+/// `origin/taskcodetwo/tareas` (sin `fetch`: la lectura fresca es de F3-5b) o,
 /// en un repo sin remoto, la rama local. Sin rama de estado → `rechazada`.
 pub fn leer<'a>(git: &'a Git, dir: &Path) -> Result<Vista<'a>, Informe> {
     let principal = localizar_principal(git, dir)?;
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index 5e47ad5..a6b78bd 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -1,4 +1,5 @@
-//! Rama de estado `taskcode/tareas` (F2-3; ADR F1-1 y F1-2).
+//! Rama de estado `taskcodetwo/tareas` (F2-3; ADR F1-1 y F1-2; nombres en
+//! [`crate::huella`]).
 //!
 //! - [`montar`] crea la rama huerfana si no existe y la monta como worktree
 //!   en `tareas/` del worktree principal (idempotente; la vuelve a montar si
@@ -30,18 +31,18 @@ pub use lectura::{Estado, Vista, leer};
 pub use montaje::{montar, montar_con};
 pub use transaccion::{Catalogo, Contexto, Fallo, Operacion, Politica, Roadmap, transaccion};
 
-/// Rama huerfana del estado (ADR F1-1).
-pub const RAMA: &str = "taskcode/tareas";
+/// Rama huerfana del estado (ADR F1-1; nombre en [`crate::huella`]).
+pub const RAMA: &str = crate::huella::RAMA_ESTADO;
 
 /// Carpeta del worktree principal donde se monta (ADR F1-1).
 pub const CARPETA: &str = "tareas";
 
 /// Refs de las operaciones pendientes de publicar (sin red, D2b).
-pub const REFS_PENDIENTES: &str = "refs/taskcode/pendientes/";
+pub const REFS_PENDIENTES: &str = crate::huella::REFS_PENDIENTES;
 
 /// Refs de las pendientes que al re-ejecutarse ya no procedian: su contenido
 /// queda en un objeto de git y no se pierde (pregunta abierta de F1-2).
-pub const REFS_RECHAZADAS: &str = "refs/taskcode/rechazadas/";
+pub const REFS_RECHAZADAS: &str = crate::huella::REFS_RECHAZADAS;
 
 /// Nombre de la operacion en la linea de contrato.
 pub(crate) const OP: &str = "estado";
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index 1559b84..eb907fc 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -26,7 +26,8 @@ const OP: &str = "montar-estado";
 /// Fichero que deja el commit raiz.
 pub(crate) const LEEME: &str = "LEEME.md";
 
-const TEXTO_LEEME: &str = "# Rama de estado de TaskCodeTwo\n\nEsta rama (`taskcode/tareas`) guarda las tareas y el roadmap (ADR F1-1).\nNo tiene historia comun con las ramas de codigo: no la mezcles con ellas.\nLa escribe el CLI; se monta como worktree en `tareas/`.\n";
+/// Texto del [`LEEME`], compuesto desde el nombre de la rama.
+use crate::huella::TEXTO_LEEME;
 
 /// Operacion que crea la rama: solo escribe [`LEEME`] si no esta.
 struct Inicializar;
diff --git a/crates/tc2/src/estado/pendientes.rs b/crates/tc2/src/estado/pendientes.rs
index 4eb556e..f35973c 100644
--- a/crates/tc2/src/estado/pendientes.rs
+++ b/crates/tc2/src/estado/pendientes.rs
@@ -2,10 +2,10 @@
 //!
 //! Cada pendiente es un commit (hecho con `commit-tree`, como los demas) con
 //! la operacion anotada en el mensaje, apuntado por
-//! `refs/taskcode/pendientes/NNNNNN`. La siguiente transaccion con red no
+//! `refs/taskcodetwo/pendientes/NNNNNN`. La siguiente transaccion con red no
 //! publica ese commit: reconstruye la operacion desde su anotacion y la
 //! re-ejecuta sobre el estado nuevo. Publicada, la ref se borra; si ya no
-//! procede, pasa a `refs/taskcode/rechazadas/` y su contenido sigue en git.
+//! procede, pasa a `refs/taskcodetwo/rechazadas/` y su contenido sigue en git.
 
 use std::path::Path;
 
@@ -152,7 +152,7 @@ mod tests {
             Some("anotar|a.md")
         );
         assert_eq!(anotacion_de("sin anotacion\n"), None);
-        assert_eq!(numero("refs/taskcode/pendientes/000012"), 12);
-        assert_eq!(numero("refs/taskcode/pendientes/x"), 0);
+        assert_eq!(numero("refs/taskcodetwo/pendientes/000012"), 12);
+        assert_eq!(numero("refs/taskcodetwo/pendientes/x"), 0);
     }
 }
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 3439fd0..416fc3a 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -535,7 +535,7 @@ fn sin_red(
     ))
 }
 
-/// Lleva la rama local `taskcode/tareas` (y `tareas/` si esta montada) a
+/// Lleva la rama local `taskcodetwo/tareas` (y `tareas/` si esta montada) a
 /// `sha`, solo por fast-forward.
 fn adelantar(git: &Git, principal: &Path, sha: &str) -> Result<Tareas, Informe> {
     let local = commit_de(git, principal, OP, &rama_local())?;
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index be0dee3..44a0dc8 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -202,15 +202,15 @@ pub(crate) fn mensaje_de(git: &Git, dir: &Path, op: &str, commit: &str) -> Paso<
 
 static TEMPORALES: AtomicUsize = AtomicUsize::new(0);
 
-/// Carpeta temporal bajo la carpeta comun; se borra al soltarla, tambien si
-/// la operacion termina con error.
+/// Carpeta temporal bajo `<comun>/taskcodetwo/estado-tmp/`; se borra al
+/// soltarla, tambien si la operacion termina con error.
 struct Temporal(PathBuf);
 
 impl Temporal {
     fn nueva(comun: &Path, op: &str) -> Paso<Self> {
         let n = TEMPORALES.fetch_add(1, Ordering::SeqCst);
-        let ruta = comun
-            .join("taskcode-estado-tmp")
+        let ruta = crate::huella::en_comun(comun)
+            .join(crate::huella::CARPETA_TEMPORALES)
             .join(format!("{}-{n}", std::process::id()));
         let _ = std::fs::remove_dir_all(&ruta);
         std::fs::create_dir_all(&ruta).map_err(|e| {
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 5a54a15..a7684b7 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -42,7 +42,7 @@ use crate::i18n::catalogo::git as txt;
 pub const REMOTO: &str = "origin";
 
 /// Carpeta de los worktrees de tarea, relativa al worktree principal.
-pub const CARPETA_WORKTREES: &str = ".taskcode/worktrees";
+pub const CARPETA_WORKTREES: &str = crate::huella::CARPETA_WORKTREES;
 
 /// Resultado interno: el `Err` ya es el informe que se devuelve.
 pub(crate) type Paso<T> = Result<T, Informe>;
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 14a7735..927497d 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -64,7 +64,7 @@ pub enum Subcomando {
     Tag,
     /// `git update-index` (solo con indice temporal)
     UpdateIndex,
-    /// `git update-ref` (solo bajo `refs/taskcode/`)
+    /// `git update-ref` (solo bajo `refs/taskcodetwo/`)
     UpdateRef,
     /// `git worktree`
     Worktree,
@@ -113,7 +113,7 @@ impl Subcomando {
 
 /// Prefijo de las unicas refs que puede escribir `update-ref` (pendientes y
 /// rechazadas de la rama de estado, F2-3). Nunca `refs/heads/*` (F1-4 D3).
-pub const PREFIJO_REFS_PROPIAS: &str = "refs/taskcode/";
+pub const PREFIJO_REFS_PROPIAS: &str = crate::huella::PREFIJO_REFS;
 
 /// Lo que devolvio una llamada a `git` que llego a ejecutarse.
 #[derive(Debug, Clone)]
@@ -506,16 +506,24 @@ mod tests {
             (Subcomando::UpdateRef, &["refs/remotes/origin/x", "abc"]),
             (Subcomando::UpdateRef, &["-d", "refs/heads/develop", "abc"]),
             (Subcomando::UpdateRef, &["--stdin"]),
-            (Subcomando::UpdateRef, &["-d", "refs/taskcode/p/1"]),
-            (Subcomando::UpdateRef, &["refs/taskcode/", "abc"]),
-            (Subcomando::UpdateRef, &["refs/taskcode/../heads/x", "abc"]),
+            (Subcomando::UpdateRef, &["-d", "refs/taskcodetwo/p/1"]),
+            (Subcomando::UpdateRef, &["refs/taskcodetwo/", "abc"]),
+            // El prefijo viejo ya no es propio (TASK-116).
             (
                 Subcomando::UpdateRef,
-                &["--no-deref", "refs/taskcode/p/1", "abc"],
+                &["refs/taskcode/pendientes/000001", "abc"],
             ),
             (
                 Subcomando::UpdateRef,
-                &["refs/taskcode/p/1", "abc", "def", "x"],
+                &["refs/taskcodetwo/../heads/x", "abc"],
+            ),
+            (
+                Subcomando::UpdateRef,
+                &["--no-deref", "refs/taskcodetwo/p/1", "abc"],
+            ),
+            (
+                Subcomando::UpdateRef,
+                &["refs/taskcodetwo/p/1", "abc", "def", "x"],
             ),
             (Subcomando::ReadTree, &["abc"]),
             (Subcomando::UpdateIndex, &["--add", "a"]),
@@ -583,9 +591,9 @@ mod tests {
         let r = git.ejecutar_con_indice(Path::new("."), Subcomando::ReadTree, &["abc"], &indice);
         assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
         for ok in [
-            &["refs/taskcode/pendientes/000001", "abc"][..],
-            &["refs/taskcode/pendientes/000001", "abc", "0000"][..],
-            &["-d", "refs/taskcode/pendientes/000001", "abc"][..],
+            &["refs/taskcodetwo/pendientes/000001", "abc"][..],
+            &["refs/taskcodetwo/pendientes/000001", "abc", "0000"][..],
+            &["-d", "refs/taskcodetwo/pendientes/000001", "abc"][..],
         ] {
             let r = git.ejecutar(Path::new("."), Subcomando::UpdateRef, ok);
             assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{ok:?}: {r:?}");
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index d267d44..e2676f1 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -205,7 +205,7 @@ fn ruta_texto(op: &str, p: &Path) -> Paso<String> {
 }
 
 /// Que `git status` del principal no muestre los worktrees de tarea:
-/// `/.taskcode/worktrees/` en `info/exclude` del repo comun (no versionado).
+/// `/.taskcodetwo/worktrees/` en `info/exclude` del repo comun (no versionado).
 fn excluir_worktrees(git: &Git, principal: &Path, op: &str) -> Paso<()> {
     excluir(git, principal, op, &format!("/{CARPETA_WORKTREES}/"))
 }
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index 54b14a2..be44271 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -596,7 +596,7 @@ mod tests {
     fn clasifica_los_fallos_del_remoto() {
         let casos = [
             (
-                "fatal: couldn't find remote ref refs/heads/taskcode/tareas",
+                "fatal: couldn't find remote ref refs/heads/taskcodetwo/tareas",
                 FalloRemoto::RamaInexistente,
             ),
             (
@@ -675,7 +675,7 @@ mod tests {
     #[test]
     fn contencion_frente_a_denegacion() {
         assert!(es_contencion(
-            "[remote rejected] (cannot lock ref 'refs/heads/taskcode/tareas': is at 1a but expected 2b)"
+            "[remote rejected] (cannot lock ref 'refs/heads/taskcodetwo/tareas': is at 1a but expected 2b)"
         ));
         assert!(es_contencion("[remote rejected] (failed to update ref)"));
         assert!(es_contencion(
diff --git a/crates/tc2/src/git/worktree.rs b/crates/tc2/src/git/worktree.rs
index 10a91f1..bc6cd7a 100644
--- a/crates/tc2/src/git/worktree.rs
+++ b/crates/tc2/src/git/worktree.rs
@@ -12,7 +12,7 @@ use crate::contrato::{Informe, Resultado};
 use crate::i18n::Texto;
 use crate::i18n::catalogo::git as txt;
 
-/// Quita `.taskcode/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
+/// Quita `.taskcodetwo/worktrees/<id>`. No existe → `sin-cambios`. Con cambios
 /// sin guardar o commits sin publicar o HEAD suelto → `rechazada`, sin tocar nada.
 pub fn quitar_worktree(git: &Git, dir: &Path, id: &str) -> Informe {
     quitar(git, dir, id).unwrap_or_else(|i| i)
diff --git a/crates/tc2/src/huella.rs b/crates/tc2/src/huella.rs
new file mode 100644
index 0000000..1f9e646
--- /dev/null
+++ b/crates/tc2/src/huella.rs
@@ -0,0 +1,127 @@
+//! Huella de la 2.0 en un repo: los unicos nombres que `tc2` deja en el
+//! (ADR F1-12 D1a y su adenda de TASK-116).
+//!
+//! Todo sale de un solo nombre, [`NOMBRE`], para que la 2.0 y la 0.16 no
+//! compartan nada y `tc2 desinstalar` (F2-14) reconozca lo suyo por nombre:
+//!
+//! - la rama de estado, [`RAMA_ESTADO`];
+//! - las refs propias, bajo [`PREFIJO_REFS`];
+//! - la carpeta de los worktrees de tarea, [`CARPETA_WORKTREES`];
+//! - lo que se guarda en la carpeta comun de git (`.git` del principal),
+//!   bajo [`CARPETA_COMUN`]: el cerrojo, la cache de identidad y los indices
+//!   temporales de la transaccion.
+//!
+//! La configuracion del proyecto (`.taskcode/config.yml`, `personas.yml` y
+//! el vocabulario) no esta aqui: F1-12 D4a la deja compartida con la 0.16.
+
+use std::io;
+use std::path::{Path, PathBuf};
+
+/// El nombre del que salen todos los demas. Es un macro para poder
+/// componer constantes con `concat!`.
+macro_rules! nombre {
+    () => {
+        "taskcodetwo"
+    };
+}
+
+/// Nombre propio de la 2.0 en un repo.
+pub const NOMBRE: &str = nombre!();
+
+/// Rama huerfana del estado (ADR F1-1 con el nombre de F1-12 D1a).
+pub const RAMA_ESTADO: &str = concat!(nombre!(), "/tareas");
+
+/// Prefijo de las unicas refs que escribe la 2.0 (y `update-ref` acepta).
+pub const PREFIJO_REFS: &str = concat!("refs/", nombre!(), "/");
+
+/// Refs de las operaciones pendientes de publicar (sin red, F1-2 D2b).
+pub const REFS_PENDIENTES: &str = concat!("refs/", nombre!(), "/pendientes/");
+
+/// Refs de las pendientes que al re-ejecutarse ya no procedian.
+pub const REFS_RECHAZADAS: &str = concat!("refs/", nombre!(), "/rechazadas/");
+
+/// Carpeta de los worktrees de tarea, relativa al worktree principal
+/// (F1-3 D2a: dentro del proyecto, excluida por `info/exclude`).
+pub const CARPETA_WORKTREES: &str = concat!(".", nombre!(), "/worktrees");
+
+/// Carpeta propia dentro de la carpeta comun de git (`.git/taskcodetwo/`).
+pub const CARPETA_COMUN: &str = nombre!();
+
+/// Cerrojo local del clon, en [`CARPETA_COMUN`] (F1-2).
+pub const FICHERO_CERROJO: &str = "estado.lock";
+
+/// Cache de la identidad, en [`CARPETA_COMUN`] (F2-13).
+pub const FICHERO_IDENTIDAD: &str = "identidad.yml";
+
+/// Indices temporales de la transaccion, en [`CARPETA_COMUN`].
+pub const CARPETA_TEMPORALES: &str = "estado-tmp";
+
+/// Texto del `LEEME.md` del commit raiz de la rama de estado.
+pub(crate) const TEXTO_LEEME: &str = concat!(
+    "# Rama de estado de TaskCodeTwo\n\nEsta rama (`",
+    nombre!(),
+    "/tareas`) guarda las tareas y el roadmap (ADR F1-1).\nNo tiene historia comun con las ramas de codigo: no la mezcles con ellas.\nLa escribe el CLI; se monta como worktree en `tareas/`.\n"
+);
+
+/// Carpeta propia bajo la carpeta comun `comun`, sin crearla.
+pub fn en_comun(comun: &Path) -> PathBuf {
+    comun.join(CARPETA_COMUN)
+}
+
+/// Carpeta propia bajo la carpeta comun `comun`, creada si no existe.
+pub fn dir_propio(comun: &Path) -> io::Result<PathBuf> {
+    let d = en_comun(comun);
+    std::fs::create_dir_all(&d)?;
+    Ok(d)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn todo_sale_del_mismo_nombre() {
+        assert_eq!(NOMBRE, "taskcodetwo");
+        assert_eq!(RAMA_ESTADO, format!("{NOMBRE}/tareas"));
+        assert_eq!(PREFIJO_REFS, format!("refs/{NOMBRE}/"));
+        assert_eq!(CARPETA_COMUN, NOMBRE);
+        assert_eq!(CARPETA_WORKTREES, format!(".{NOMBRE}/worktrees"));
+        assert!(TEXTO_LEEME.contains(&format!("(`{RAMA_ESTADO}`)")));
+    }
+
+    /// C5: las refs propias viven bajo `refs/taskcodetwo/`.
+    #[test]
+    fn refs_propias_bajo_su_prefijo() {
+        assert_eq!(PREFIJO_REFS, "refs/taskcodetwo/");
+        assert_eq!(REFS_PENDIENTES, "refs/taskcodetwo/pendientes/");
+        assert_eq!(REFS_RECHAZADAS, "refs/taskcodetwo/rechazadas/");
+        assert_eq!(crate::git::PREFIJO_REFS_PROPIAS, PREFIJO_REFS);
+        assert_eq!(crate::estado::REFS_PENDIENTES, REFS_PENDIENTES);
+        assert_eq!(crate::estado::REFS_RECHAZADAS, REFS_RECHAZADAS);
+    }
+
+    /// C3: la carpeta de worktrees no coincide con ninguna ruta de la 0.16
+    /// (`.taskcode/...` en el arbol y `.git/taskcode/...` en la comun).
+    #[test]
+    fn worktrees_fuera_de_las_rutas_de_la_016() {
+        let w = crate::git::CARPETA_WORKTREES;
+        assert_eq!(w, ".taskcodetwo/worktrees");
+        for ruta_016 in [".taskcode/", ".git/taskcode/", ".git/", "tareas/"] {
+            assert!(!w.starts_with(ruta_016), "{w} empieza por {ruta_016}");
+        }
+        assert_ne!(w.split('/').next(), Some(".taskcode"));
+    }
+
+    #[test]
+    fn dir_propio_crea_la_carpeta() -> io::Result<()> {
+        let comun = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("huella-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&comun);
+        let d = dir_propio(&comun)?;
+        assert_eq!(d, comun.join("taskcodetwo"));
+        assert!(d.is_dir());
+        let _ = std::fs::remove_dir_all(&comun);
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index f2bafb7..bf12802 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -13,7 +13,8 @@
 //! - Sin plataforma reconocida no hay identidad (D4): no hay vuelta a texto
 //!   libre.
 //!
-//! La cache vive en `<git-common-dir>/tc2/identidad.yml`: no se commitea y la
+//! La cache vive en `<git-common-dir>/taskcodetwo/identidad.yml`
+//! ([`crate::huella`]): no se commitea y la
 //! comparten los worktrees del clon. Editarla no aprueba nada: la aprobacion
 //! sigue siendo la firma de F1-6/F4-2 (riesgo aceptado en el plan).
 
@@ -43,10 +44,6 @@ pub const TTL_SEGUNDOS: u64 = 7 * 24 * 60 * 60;
 /// no vale.
 const MARGEN_RELOJ: u64 = 5 * 60;
 
-/// Carpeta y fichero de la cache bajo la carpeta comun del repo.
-const CARPETA_CACHE: &str = "tc2";
-const FICHERO_CACHE: &str = "identidad.yml";
-
 /// Tamano maximo de la cache que se lee (la que escribe `tc2` ocupa menos
 /// de 400 bytes).
 pub const LIMITE_CACHE: u64 = 64 * 1024;
@@ -206,7 +203,7 @@ struct GuardadaCruda {
     verificado: u64,
 }
 
-/// Cache local de la identidad (`<git-common-dir>/tc2/identidad.yml`).
+/// Cache local de la identidad (`<git-common-dir>/taskcodetwo/identidad.yml`).
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Cache {
     ruta: PathBuf,
@@ -216,7 +213,7 @@ impl Cache {
     /// La cache del repo cuya carpeta comun es `comun`.
     pub fn en(comun: &Path) -> Self {
         Self {
-            ruta: comun.join(CARPETA_CACHE).join(FICHERO_CACHE),
+            ruta: crate::huella::en_comun(comun).join(crate::huella::FICHERO_IDENTIDAD),
         }
     }
 
@@ -571,6 +568,30 @@ mod tests {
 
     const AHORA: u64 = 1_800_000_000;
 
+    /// C2 de TASK-116: la cache se escribe en `<comun>/taskcodetwo/` y no
+    /// deja nada suelto en la carpeta comun.
+    #[test]
+    fn la_cache_vive_en_la_carpeta_propia() {
+        let d = carpeta("huella");
+        let cache = Cache::en(&d);
+        assert_eq!(cache.ruta(), d.join("taskcodetwo").join("identidad.yml"));
+        let ana = Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: "ana".to_owned(),
+        };
+        assert_eq!(cache.guardar(&ana, AHORA), Ok(()));
+        assert!(cache.ruta().is_file());
+        let sueltos: Vec<_> = std::fs::read_dir(&d)
+            .into_iter()
+            .flatten()
+            .flatten()
+            .map(|e| e.file_name())
+            .collect();
+        assert_eq!(sueltos, ["taskcodetwo"]);
+        let _ = std::fs::remove_dir_all(&d);
+    }
+
     /// C6: una cuenta `Bot` (o `web-flow`, o fuera de la lista) nunca acaba
     /// en `asignado_a`: error tipado. Sin terminal, `Quien` es `Agente` y la
     /// transicion lleva `decidido_por: agente`; con terminal, `persona`.
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index a44624b..485290e 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -2,14 +2,15 @@
 //!
 //! Modulos: [`contrato`] (salida de toda operacion, ADR F1-4), [`git`]
 //! (capa de operaciones Git deterministas, F2-2) y [`estado`] (rama de
-//! estado `taskcode/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
+//! estado `taskcodetwo/tareas`, F2-3), [`alta`] (alta de tareas con reserva de
 //! ID, F2-4) [`tarea`] (modelo de tarea y frontmatter YAML, F2-5) y
 //! [`plataforma`] (puerto Plataforma y adaptadores GitHub y GitLab, F2-6 y
 //! F2-9), [`carril`] (tabla de suelo de F1-10) y [`config`] (configuracion
 //! con esquema validado, F2-7) e [`i18n`] (mensajes en espanol e ingles,
 //! F2-8), [`version`] (`version --json` y `compatible`, F2-12) y
 //! [`actualizar_cli`] (`actualizar --cli`, F2-12) e [`identidad`] (quien
-//! ejecuta y `tc2 identidad`, F2-13). Las capas
+//! ejecuta y `tc2 identidad`, F2-13) y [`huella`] (los nombres que la 2.0
+//! deja en un repo, F1-12 D1a). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -20,6 +21,7 @@ pub mod contrato;
 pub mod descarga;
 pub mod estado;
 pub mod git;
+pub mod huella;
 pub mod i18n;
 pub mod identidad;
 pub mod plataforma;
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-116/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-116/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-116/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (11 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/alta.rs" ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/convivencia_016.rs" ":(literal)crates/tc2/tests/estado_bordes.rs" ":(literal)crates/tc2/tests/estado_casos.rs" ":(literal)crates/tc2/tests/estado_concurrencia.rs" ":(literal)crates/tc2/tests/git_binario.rs" ":(literal)crates/tc2/tests/git_casos_limite.rs" ":(literal)crates/tc2/tests/git_ciclo.rs" ":(literal)crates/tc2/tests/git_conflicto.rs" ":(literal)crates/tc2/tests/git_push.rs"
````

````
crates/tc2/tests/alta.rs                |  11 ++++++--
 crates/tc2/tests/comun/mod.rs           |   3 ++-
 crates/tc2/tests/convivencia_016.rs     | 209 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/estado_bordes.rs       |  23 +++++++++--------
 crates/tc2/tests/estado_casos.rs        |  36 ++++++++++++++++-----------
 crates/tc2/tests/estado_concurrencia.rs |   4 +--
 crates/tc2/tests/git_binario.rs         |   2 +-
 crates/tc2/tests/git_casos_limite.rs    |   2 +-
 crates/tc2/tests/git_ciclo.rs           |   6 ++---
 crates/tc2/tests/git_conflicto.rs       |   2 +-
 crates/tc2/tests/git_push.rs            |   2 +-
 11 files changed, 263 insertions(+), 37 deletions(-)
````
