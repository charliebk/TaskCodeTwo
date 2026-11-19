# Peticion de revision — TASK-037 (ronda 1)

- Tarea: TASK-037 — F4-2 approve con la identidad de la persona
- Rama revisada: feature/task-037-f4-2-approve-con-la-identidad-de-la-pers
- Rama base: develop
- Commit revisado (HEAD): b9a6a55f86af56fe615da1af20a1480ea2d9ae0a
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-037 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-037/tarea.md
- tareas/03-en-revision/TASK-037/planificacion/
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/cli.rs
- crates/tc2/src/i18n/catalogo/git.rs
- crates/tc2/src/i18n/catalogo/plan.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/mod.rs
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/plan/plantillas.rs
- crates/tc2/tests/aprobar.rs

## Commits a revisar (git log develop..HEAD)

````
b9a6a55 chore(TASK-037): autorrevision
7b98d98 docs(TASK-037): adendas de approve en F1-6, F1-4 y F1-10
4256e92 feat(TASK-037): approve firmado con la identidad de la persona
42083d1 chore(TASK-037): tarea en curso
````

## Diff de tu dominio (14 fichero(s) de 17; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index bb2aac7..88c6b8a 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -32,7 +32,7 @@ mod pendientes;
 mod tareas;
 mod transaccion;
 
-pub use crate::git::Cambio;
+pub use crate::git::{Cambio, Sello};
 pub use cerrojo::Cerrojo;
 pub use frescura::{Frescura, Marca, Remoto, hace, sincronizado_texto};
 pub(crate) use lectura::referencia_de_lectura;
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 42af1a0..b1c30af 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -18,7 +18,9 @@ use super::lectura::{Estado, Vista};
 use super::pendientes::{self, ANOTACION, Destino};
 use super::{CARPETA, OP, RAMA};
 use crate::contrato::{Informe, Resultado, corto};
-use crate::git::estado::{construir_commit, dir_comun, hay_remoto};
+use crate::git::estado::{
+    Sello, construir_commit, construir_commit_sellado, dir_comun, hay_remoto,
+};
 use crate::git::remoto::publicar;
 use crate::git::{
     Cambio, FalloRemoto, Git, REMOTO, Subcomando, commit_de, correr, es_ancestro, exigir,
@@ -65,6 +67,12 @@ pub trait Operacion {
     fn exige_red(&self) -> bool;
     /// Los cambios sobre `estado`, o por que no procede.
     fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo>;
+    /// Como se sella su commit (F4-2): [`Sello::Normal`] salvo `approve`,
+    /// que firma. Con [`Sello::Firmado`] las pendientes no entran en ese
+    /// commit (D4a de TASK-037): se publican con la siguiente operacion.
+    fn sello(&self) -> Sello {
+        Sello::Normal
+    }
 }
 
 /// Reconstruye una operacion desde su anotacion (`None`: desconocida).
@@ -312,7 +320,9 @@ fn intento(
     let mut vista = Vista::nueva(git, principal, base.clone())?;
     let mut anotaciones = Vec::new();
     let mut destinos = Vec::new();
-    if remoto {
+    let sello = op.sello();
+    // Un commit firmado lleva solo lo que firma la persona (D4a).
+    if remoto && sello == Sello::Normal {
         for p in pendientes::listar(git, principal)? {
             let reconstruida = p.anotacion.as_deref().and_then(|a| (c.catalogo)(a));
             let Some(o) = reconstruida else {
@@ -367,13 +377,14 @@ fn intento(
     }
 
     let mensaje = mensaje(op.nombre(), &anotaciones);
-    let Some(sha) = construir_commit(
+    let Some(sha) = construir_commit_sellado(
         git,
         principal,
         OP,
         base.as_deref(),
         &vista.cambios(),
         &mensaje,
+        &sello,
     )?
     else {
         let (publicadas, rechazadas) = pendientes::resolver(git, principal, &destinos)?;
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index a69f629..ebe19a1 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -314,6 +314,22 @@ fn hashear(git: &Git, dir: &Path, op: &str, rutas: &str, n: usize) -> Paso<Vec<S
     }
 }
 
+/// Como se sella el commit de la rama de estado (F4-2).
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub enum Sello {
+    /// Sin firma: el de todas las operaciones salvo `approve`.
+    #[default]
+    Normal,
+    /// Firmado con la clave de quien lo lanza (`commit-tree -S`, con
+    /// `gpg.format` y `user.signingkey` de su configuracion) y con un
+    /// trailer `<clave>: <blob>` por cada `(clave, ruta)`, con el blob de
+    /// esa ruta en el arbol del propio commit (`Plan-Blob`, F1-6 D3).
+    Firmado {
+        /// `(clave del trailer, ruta en la rama de estado)`.
+        trailers: Vec<(&'static str, String)>,
+    },
+}
+
 /// Construye un commit con el arbol de `base` mas `cambios` y `base` como
 /// padre (sin padre si `base` es `None`: la raiz de la rama huerfana). No
 /// mueve ninguna rama. `None` si el arbol queda igual que el de `base`.
@@ -324,6 +340,21 @@ pub(crate) fn construir_commit(
     base: Option<&str>,
     cambios: &[Cambio],
     mensaje: &str,
+) -> Paso<Option<String>> {
+    construir_commit_sellado(git, dir, op, base, cambios, mensaje, &Sello::Normal)
+}
+
+/// Como [`construir_commit`], con el `sello`. Con [`Sello::Firmado`], si el
+/// arbol no cambia no se firma nada (`None` antes de `commit-tree`); si la
+/// firma falla, `rechazada motivo=firma` (no hay commit que publicar).
+pub(crate) fn construir_commit_sellado(
+    git: &Git,
+    dir: &Path,
+    op: &str,
+    base: Option<&str>,
+    cambios: &[Cambio],
+    mensaje: &str,
+    sello: &Sello,
 ) -> Paso<Option<String>> {
     if let Some(malo) = cambios.iter().find(|c| !ruta_valida(&c.ruta)) {
         return Err(Informe::nuevo(op, Resultado::Error)
@@ -384,9 +415,35 @@ pub(crate) fn construir_commit(
     if let Some(b) = base {
         args.extend(["-p", b]);
     }
-    args.extend(["-m", mensaje]);
-    let sha = exigir(git, dir, op, Subcomando::CommitTree, &args)?;
-    Ok(Some(sha.trim().to_owned()))
+    let Sello::Firmado { trailers } = sello else {
+        args.extend(["-m", mensaje]);
+        let sha = exigir(git, dir, op, Subcomando::CommitTree, &args)?;
+        return Ok(Some(sha.trim().to_owned()));
+    };
+    let mut sellado = mensaje.to_owned();
+    for (clave, ruta) in trailers {
+        // El blob del arbol de este mismo commit, no el de la base.
+        let blob = exigir(
+            git,
+            dir,
+            op,
+            Subcomando::RevParse,
+            &["--verify", "--end-of-options", &format!("{arbol}:{ruta}")],
+        )?;
+        sellado.push_str(&format!("{clave}: {}\n", blob.trim()));
+    }
+    args.extend(["-m", &sellado]);
+    let s = git.ejecutar_firmado(dir, &args).map_err(|e| {
+        Informe::nuevo(op, Resultado::Error)
+            .con("git", Subcomando::CommitTree.nombre())
+            .diagnostico(e.mensaje())
+    })?;
+    if !s.exito() {
+        return Err(Informe::nuevo(op, Resultado::Rechazada)
+            .con("motivo", "firma")
+            .diagnostico(Mensaje::nuevo(&txt::FIRMA_FALLO).con("error", s.primera_linea_error())));
+    }
+    Ok(Some(s.stdout.trim().to_owned()))
 }
 
 /// Si `origin` esta configurado. `ls-remote --get-url` no habla con el
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 5eb37a6..01da255 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -19,7 +19,7 @@ mod tag;
 mod worktree;
 
 pub use diff::{diff_tarea, ficheros_de_rama};
-pub use estado::Cambio;
+pub use estado::{Cambio, Sello};
 pub use guardar::guardar;
 pub use puerto::{ErrorGit, Git, PREFIJO_REFS_PROPIAS, SalidaCruda, SalidaGit, Subcomando};
 pub use rama::{
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 7b401a1..5af62de 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -346,6 +346,32 @@ impl Git {
         self.lanzar_con_entrada(dir, sub, args, Some(indice), entrada)
     }
 
+    /// `git commit-tree -S <args>` en `dir`: la unica forma firmada (F4-2,
+    /// `approve`). `args` son los de un `commit-tree` normal (sin `-S`, que
+    /// pone esta funcion): fuera de aqui `-S` y `--gpg-sign` estan
+    /// prohibidos. La entrada estandar se hereda para que `ssh-keygen` (o
+    /// `gpg`) pueda pedir la frase de paso por el terminal de la persona;
+    /// stdout (el sha) y stderr (el fallo de la firma) se capturan.
+    pub fn ejecutar_firmado(&self, dir: &Path, args: &[&str]) -> Result<SalidaGit, ErrorGit> {
+        comprobar_argumentos(Subcomando::CommitTree, args)?;
+        let mut firmados = Vec::with_capacity(args.len() + 1);
+        firmados.push(FIRMA);
+        firmados.extend_from_slice(args);
+        let mut cmd = self.comando(dir, Subcomando::CommitTree, &firmados, None);
+        cmd.stdin(Stdio::inherit())
+            .stdout(Stdio::piped())
+            .stderr(Stdio::piped());
+        let salida = cmd
+            .spawn()
+            .and_then(std::process::Child::wait_with_output)
+            .map_err(|e| ErrorGit::Lanzar(e.to_string()))?;
+        Ok(SalidaGit {
+            codigo: salida.status.code().unwrap_or(-1),
+            stdout: String::from_utf8_lossy(&salida.stdout).into_owned(),
+            stderr: String::from_utf8_lossy(&salida.stderr).into_owned(),
+        })
+    }
+
     fn lanzar_con_entrada(
         &self,
         dir: &Path,
@@ -464,6 +490,10 @@ pub const HASH_OBJECT_STDIN: &[&str] = &["-w", "--no-filters", "--stdin-paths"];
 /// `<modo> <sha>\t<ruta>` terminadas en NUL, siempre contra un indice temporal.
 pub const UPDATE_INDEX_STDIN: &[&str] = &["--add", "-z", "--index-info"];
 
+/// La opcion de firma de `commit-tree`: solo la pone
+/// [`Git::ejecutar_firmado`].
+const FIRMA: &str = "-S";
+
 /// Argumentos prohibidos dentro de los subcomandos permitidos (F1-4,
 /// decision 3). Se mira cada argumento entero, no subcadenas (H9). No se
 /// miran el texto de un mensaje (`-m <texto>` en `commit` y `tag`) ni las
@@ -480,7 +510,10 @@ fn comprobar_argumentos(sub: Subcomando, args: &[&str]) -> Result<(), ErrorGit>
         sub,
         Subcomando::Add | Subcomando::Commit | Subcomando::Status | Subcomando::Diff
     );
-    let con_mensaje = matches!(sub, Subcomando::Commit | Subcomando::Tag);
+    let con_mensaje = matches!(
+        sub,
+        Subcomando::Commit | Subcomando::Tag | Subcomando::CommitTree
+    );
     let mut iter = args.iter().copied();
     while let Some(a) = iter.next() {
         if a == "--" && con_rutas {
@@ -539,6 +572,9 @@ fn es_prohibido(sub: Subcomando, a: &str) -> bool {
         Subcomando::Tag => matches!(a, "-d" | "-f" | "--delete"),
         Subcomando::Merge => a == "--abort",
         Subcomando::Commit => a == "--amend",
+        // La firma solo por `ejecutar_firmado` (F4-2): ni `-S`, ni
+        // `-S<clave>`, ni `--gpg-sign[=<clave>]` en un `commit-tree` normal.
+        Subcomando::CommitTree => a.starts_with(FIRMA) || a.starts_with("--gpg-sign"),
         Subcomando::ReadTree => matches!(a, "-u" | "-m" | "--reset") || a.starts_with("--prefix"),
         _ => false,
     }
@@ -593,6 +629,13 @@ mod tests {
             (Subcomando::Tag, &["--force", "v1"]),
             (Subcomando::Merge, &["--abort"]),
             (Subcomando::Commit, &["--amend", "-m", "x"]),
+            (Subcomando::CommitTree, &["-S", "abc", "-m", "x"]),
+            (Subcomando::CommitTree, &["abc", "-Sclave", "-m", "x"]),
+            (Subcomando::CommitTree, &["--gpg-sign", "abc", "-m", "x"]),
+            (
+                Subcomando::CommitTree,
+                &["--gpg-sign=clave", "abc", "-m", "x"],
+            ),
             (Subcomando::Pull, &["origin", "develop"]),
             (Subcomando::UpdateRef, &["refs/heads/develop", "abc"]),
             (Subcomando::UpdateRef, &["refs/remotes/origin/x", "abc"]),
@@ -753,4 +796,29 @@ mod tests {
         let r = git.ejecutar(Path::new("."), Subcomando::Push, &["origin", "--", "+r:r"]);
         assert!(matches!(r, Err(ErrorGit::Prohibido(_))));
     }
+
+    /// F4-2: la unica forma firmada es `ejecutar_firmado`, que pone el `-S`
+    /// ella misma; un `-S` o `--gpg-sign` de mas se rechaza sin lanzar git.
+    /// El texto del mensaje no es una opcion.
+    #[test]
+    fn la_firma_solo_por_ejecutar_firmado() {
+        let git = git_sin_binario();
+        for malo in [
+            &["-S", "abc", "-m", "x"][..],
+            &["abc", "-Sclave", "-m", "x"][..],
+            &["--gpg-sign=clave", "abc", "-m", "x"][..],
+        ] {
+            let r = git.ejecutar_firmado(Path::new("."), malo);
+            assert!(matches!(r, Err(ErrorGit::Prohibido(_))), "{malo:?}: {r:?}");
+        }
+        assert!(git.registro().is_empty());
+        let r = git.ejecutar_firmado(Path::new("."), &["abc", "-p", "def", "-m", "-S de texto"]);
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
+        assert_eq!(
+            git.registro(),
+            ["git commit-tree -S abc -p def -m -S de texto"]
+        );
+        let r = git.ejecutar(Path::new("."), Subcomando::CommitTree, &["abc", "-m", "-S"]);
+        assert!(matches!(r, Err(ErrorGit::Lanzar(_))), "{r:?}");
+    }
 }
diff --git a/crates/tc2/src/i18n/catalogo/cli.rs b/crates/tc2/src/i18n/catalogo/cli.rs
index 5bc161e..c78f094 100644
--- a/crates/tc2/src/i18n/catalogo/cli.rs
+++ b/crates/tc2/src/i18n/catalogo/cli.rs
@@ -10,8 +10,8 @@ crate::i18n::textos! {
         en: "tc2: could not write the output: {error}",
     }
     USO {
-        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
-        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
+        es: "Uso: tc2 --version | tc2 guardar [--json] \"<mensaje>\" <rutas...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<titulo>\" [campos] | tc2 import [--json] [--simular] [--sprint <N>] <fichero> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<motivo>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <carril>] | tc2 plan guardar [--json] TASK-NNN <destino> <fichero> | tc2 approve [--json] TASK-NNN [--pedir-cambios <fichero>] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <rama>] [--json]",
+        en: "Usage: tc2 --version | tc2 guardar [--json] \"<message>\" <paths...> | tc2 actualizar-rama [--json] | tc2 new [--json] \"<title>\" [fields] | tc2 import [--json] [--simular] [--sprint <N>] <file> | tc2 next [--json] | tc2 roadmap [--comparar | --escribir] [--json] | tc2 status [--json] | tc2 board [--sin-plataforma] [--json] | tc2 start [--json] TASK-NNN | tc2 descartar [--json] TASK-NNN \"<reason>\" | tc2 pausa [--json] TASK-NNN | tc2 reanudar [--json] TASK-NNN | tc2 liberar [--json] TASK-NNN | tc2 reasignar [--json] TASK-NNN <login> | tc2 plan [--json] TASK-NNN [--carril <lane>] | tc2 plan guardar [--json] TASK-NNN <destination> <file> | tc2 approve [--json] TASK-NNN [--pedir-cambios <file>] | tc2 estimar [--json] [TASK-NNN | --error TASK-NNN] | tc2 version [--json] | tc2 compatible [--json] --protocolo <A-B> | tc2 actualizar --cli [--comprobar] [--version <X.Y.Z>] [--json] | tc2 identidad [--json] | tc2 init [--simular] [--json] | tc2 doctor [--json] | tc2 desinstalar (--maquina | --proyecto) [--simular] [--config] [--borrar-tareas <branch>] [--json]",
     }
     FALTA_LA_ORDEN {
         es: "tc2: falta la orden. {uso}",
diff --git a/crates/tc2/src/i18n/catalogo/git.rs b/crates/tc2/src/i18n/catalogo/git.rs
index 141e563..691a4c5 100644
--- a/crates/tc2/src/i18n/catalogo/git.rs
+++ b/crates/tc2/src/i18n/catalogo/git.rs
@@ -50,6 +50,10 @@ crate::i18n::textos! {
         es: "git hash-object no devolvio un hash por fichero",
         en: "git hash-object did not return one hash per file",
     }
+    FIRMA_FALLO {
+        es: "git no pudo firmar el commit ({error}): configura gpg.format y user.signingkey con tu clave de firma y vuelve a lanzarlo desde tu terminal",
+        en: "git could not sign the commit ({error}): set gpg.format and user.signingkey to your signing key and run it again from your terminal",
+    }
 
     // Donde se lanza.
     LANZALO_EN_EL_WORKTREE {
diff --git a/crates/tc2/src/i18n/catalogo/plan.rs b/crates/tc2/src/i18n/catalogo/plan.rs
index c31d838..1ac01c7 100644
--- a/crates/tc2/src/i18n/catalogo/plan.rs
+++ b/crates/tc2/src/i18n/catalogo/plan.rs
@@ -1,4 +1,5 @@
-//! Mensajes de `tc2 plan` y `tc2 plan guardar` (F4-1).
+//! Mensajes de `tc2 plan` y `tc2 plan guardar` (F4-1) y de `tc2 approve`
+//! (F4-2).
 
 crate::i18n::textos! {
     USO_PLAN {
@@ -61,4 +62,40 @@ crate::i18n::textos! {
         es: "{ruta} pasa de {max} bytes: no se publica",
         en: "{ruta} is larger than {max} bytes: it is not published",
     }
+    USO_APPROVE {
+        es: "uso: approve [--json] TASK-NNN [--pedir-cambios <fichero>]",
+        en: "usage: approve [--json] TASK-NNN [--pedir-cambios <file>]",
+    }
+    DECIDIDO_POR {
+        es: "approve no admite --decidido-por: aprueba siempre una persona con su firma, en cualquier modo (ADR F1-10, decision 6)",
+        en: "approve does not accept --decidido-por: a person always approves with their signature, in any mode (ADR F1-10, decision 6)",
+    }
+    APROBAR_SIN_PERSONA {
+        es: "approve solo lo lanza una persona desde su terminal, con TC2_TOKEN_PERSONA y su clave de firma: el agente no aprueba (RR-04)",
+        en: "approve is only run by a person from their terminal, with TC2_TOKEN_PERSONA and their signing key: the agent does not approve (RR-04)",
+    }
+    APROBAR_DE_OTRA_PERSONA {
+        es: "la tarea {id} la lleva {login}: solo ella aprueba su plan (o pide que te la reasigne)",
+        en: "task {id} is taken by {login}: only they approve its plan (or ask to have it reassigned)",
+    }
+    APROBAR_PAUSADA {
+        es: "la tarea {id} esta en pausa: reanudala con tc2 reanudar {id} antes de aprobar su plan",
+        en: "task {id} is paused: resume it with tc2 reanudar {id} before approving its plan",
+    }
+    SIN_PLAN {
+        es: "la tarea {id} no tiene plan-final.md: lanza tc2 plan {id} y publica el plan con tc2 plan guardar",
+        en: "task {id} has no plan-final.md: run tc2 plan {id} and publish the plan with tc2 plan guardar",
+    }
+    PLAN_ESQUELETO {
+        es: "el plan-final.md de la tarea {id} es el esqueleto sin rellenar: publica el plan con tc2 plan guardar antes de aprobarlo",
+        en: "the plan-final.md of task {id} is the unfilled skeleton: publish the plan with tc2 plan guardar before approving it",
+    }
+    SIN_RONDA {
+        es: "la tarea {id} no tiene ninguna ronda de planificacion: lanza tc2 plan {id} antes de pedir cambios",
+        en: "task {id} has no planning round: run tc2 plan {id} before requesting changes",
+    }
+    RONDA_CERRADA {
+        es: "la ronda {ronda} de la tarea {id} ya tiene cambios pedidos: lanza tc2 plan {id} para abrir la siguiente",
+        en: "round {ronda} of task {id} already has requested changes: run tc2 plan {id} to open the next one",
+    }
 }
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index 30d6811..a507f24 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -19,7 +19,8 @@
 //! (`board` y `status`, F3-5) y [`toma`] (`start`, F3-6) y [`solape`]
 //! (aviso de solape de `start`, F3-11) y [`ciclo`]
 //! (`descartar`, `pausa` y `reanudar`, F3-7; `liberar` y `reasignar`,
-//! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1). Las capas
+//! F3-10) y [`doctor`] (F3-8) y [`plan`] (`plan` y `plan guardar`, F4-1;
+//! `approve`, F4-2). Las capas
 //! que vienen entran como modulos de esta biblioteca.
 
 pub mod actualizar_cli;
@@ -203,6 +204,12 @@ where
             let (json, resto) = con_json(resto);
             de_una_tarea(plan::NOMBRE, (json, idioma), |e| plan::orden(&resto, e))
         }
+        [orden, resto @ ..] if orden == plan::aprobar::NOMBRE => {
+            let (json, resto) = con_json(resto);
+            de_una_tarea(plan::aprobar::NOMBRE, (json, idioma), |e| {
+                plan::orden_approve(&resto, e)
+            })
+        }
         [orden, resto @ ..] if orden == estimacion::NOMBRE => {
             let (json, resto) = con_json(resto);
             orden_estimar(&resto, json, idioma)
@@ -353,8 +360,8 @@ fn orden_identidad(args: &[String]) -> Informe {
 }
 
 /// Una orden sobre una tarea (`start`, F3-6; `descartar`, `pausa` y
-/// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10; `plan`, F4-1) en la
-/// carpeta actual:
+/// `reanudar`, F3-7; `liberar` y `reasignar`, F3-10; `plan`, F4-1;
+/// `approve`, F4-2) en la carpeta actual:
 /// quien la lanza sale de
 /// [`identidad::resolver`] con el terminal y los tokens del proceso.
 ///
@@ -1012,6 +1019,57 @@ mod tests {
         }
     }
 
+    /// TASK-037: `approve` mal llamado es `uso` (2) sin leer nada;
+    /// `--decidido-por automatico` no existe en la 2.0 (F1-10 decision 6).
+    #[test]
+    fn approve_mal_llamado_es_uso() {
+        for (args, motivo) in [
+            (vec!["approve"], "sin-id"),
+            (vec!["approve", "TASK-1"], "id-invalido"),
+            (vec!["approve", "TASK-001", "x"], "argumentos-de-mas"),
+            (
+                vec!["approve", "TASK-001", "--pedir-cambios"],
+                "sin-fichero",
+            ),
+            (
+                vec![
+                    "approve",
+                    "TASK-001",
+                    "--pedir-cambios",
+                    "a",
+                    "--pedir-cambios",
+                    "b",
+                ],
+                "argumentos-de-mas",
+            ),
+            (
+                vec!["approve", "TASK-001", "--decidido-por", "automatico"],
+                "decidido-por",
+            ),
+            (
+                vec!["approve", "TASK-001", "--decidido-por=automatico"],
+                "decidido-por",
+            ),
+        ] {
+            let es = ejecutar_en(Idioma::Es, &args);
+            let en = ejecutar_en(Idioma::En, &args);
+            assert_eq!((es.codigo, en.codigo), (2, 2), "{args:?}");
+            assert_eq!(
+                es.stdout,
+                format!("approve uso motivo={motivo}\n"),
+                "{args:?}"
+            );
+            assert_eq!(es.stdout, en.stdout);
+            assert_ne!(es.stderr, en.stderr, "{args:?}");
+            assert_eq!(es.stderr.lines().count(), 1, "{args:?}");
+        }
+        let s = ejecutar_en(
+            Idioma::Es,
+            ["approve", "TASK-001", "--decidido-por", "automatico"],
+        );
+        assert!(s.stderr.contains("decision 6"), "{}", s.stderr);
+    }
+
     #[test]
     fn sin_orden_falla() {
         assert_eq!(ejecutar(Vec::<String>::new()).codigo, 2);
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
new file mode 100644
index 0000000..6c9a7f4
--- /dev/null
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -0,0 +1,523 @@
+//! `tc2 approve` como operaciones de la rama de estado (F4-2, RR-04).
+//!
+//! - [`Aprobar`]: solo existe para una persona ([`Aprobar::nueva`] con
+//!   [`Quien::Agente`] es `None`). Exige la tarea `en-diseno`, sin pausa, con
+//!   un `plan-final.md` que no sea el esqueleto; la de otra persona se
+//!   rechaza (`de-otra-persona`). Escribe `plan_aprobado: true`,
+//!   `actualizado` y la fila `approve | <modo congelado> | persona`. Con el
+//!   plan ya aprobado no cambia nada (`sin-cambios`, sin firmar otra vez).
+//!   Su commit va firmado y con el trailer `Plan-Blob` del `plan-final.md`
+//!   de ese mismo commit ([`Sello::Firmado`], F1-6 decision 3).
+//! - [`PedirCambios`]: publica `brainstorm/cambios-pedidos-<n>.md` en la
+//!   ronda mayor, que asi queda cerrada: el siguiente `plan` abre la `n+1`
+//!   (D1a). No concede nada: la puede lanzar tambien el agente. No toca
+//!   `tarea.md` (como `plan guardar`, no es una fase).
+//!
+//! Las dos exigen red y nunca quedan pendientes (no tienen catalogo).
+
+use std::cell::RefCell;
+
+use super::operacion::{EN_DISENO, modo_congelado, ronda_mayor};
+use super::plantillas as p;
+use super::roles::{self, Rol};
+use crate::config::Config;
+use crate::estado::{
+    Cambio, Estado, Fallo, Operacion, Sello, de_otra_persona, por_estado, tarea_de,
+};
+use crate::i18n::Mensaje;
+use crate::i18n::catalogo::plan as txt;
+use crate::identidad::Quien;
+use crate::tarea::{Asignado, Tarea, Transicion};
+use crate::toma::instante;
+
+/// Nombre de la orden, de la operacion y de su fila.
+pub const NOMBRE: &str = "approve";
+
+/// Nombre de la operacion de `--pedir-cambios` (en el commit).
+pub const NOMBRE_CAMBIOS: &str = "approve-pedir-cambios";
+
+/// Clave del trailer que ata la aprobacion al plan (F1-5, F1-6).
+pub const PLAN_BLOB: &str = "Plan-Blob";
+
+/// Fase de la fila que se escribe: la orden.
+const FASE: &str = NOMBRE;
+
+/// Decidido por: solo una persona aprueba.
+const PERSONA: &str = "persona";
+
+/// Aprobar el plan de la tarea `id`.
+#[derive(Debug)]
+pub struct Aprobar {
+    id: String,
+    asignado: Asignado,
+    ahora: u64,
+    modo: &'static str,
+    dueno: RefCell<Option<String>>,
+}
+
+impl Aprobar {
+    /// `approve` de `id` por `quien` en `ahora`; `None` si no es una
+    /// persona (RR-04: el agente no aprueba nunca). El modo de la fila es el
+    /// congelado por `plan`; sin el, el `modo_flujo` del suelo hasta que se
+    /// de la configuracion con [`Aprobar::con_config`].
+    pub fn nueva(id: &str, quien: &Quien, ahora: u64) -> Option<Self> {
+        let Quien::Persona(_) = quien else {
+            return None;
+        };
+        Some(Self {
+            id: id.to_owned(),
+            asignado: quien.asignado(),
+            ahora,
+            modo: Config::suelo().modo_flujo.nombre(),
+            dueno: RefCell::new(None),
+        })
+    }
+
+    /// El `modo_flujo` de la configuracion de la base, para la fila si la
+    /// tarea no tiene ninguna `plan`.
+    #[must_use]
+    pub fn con_config(mut self, config: &Config) -> Self {
+        self.modo = config.modo_flujo.nombre();
+        self
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+
+    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
+        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
+    }
+}
+
+/// Si `plan` es el esqueleto de `plan-final.md` que dejo `plan`: se compara
+/// con el de cada juego de roles posible (de 0 a 5 roles, con las etiquetas
+/// de la tarea y sin ellas) y con el de los roles de cada ronda con
+/// unificador, como `planEsPlantilla` de la 0.16.
+fn es_esqueleto(plan: &str, t: &Tarea, rutas: &[String]) -> bool {
+    let plan = plan.replace("\r\n", "\n");
+    let mut juegos: Vec<Vec<Rol>> = Vec::new();
+    for n in 0..=5u8 {
+        juegos.push(roles::seleccionar(n, &t.etiquetas));
+        juegos.push(roles::seleccionar(n, &[]));
+    }
+    juegos.extend(juegos_de_rondas(rutas, &t.id));
+    juegos
+        .iter()
+        .any(|r| p::plan_final(&t.id, &t.titulo, r) == plan)
+}
+
+/// Los roles de cada ronda, por los nombres de sus peticiones
+/// (`peticion-brainstorm-<rol>-<n>.md`), en el orden de [`roles::todos`].
+fn juegos_de_rondas(rutas: &[String], id: &str) -> Vec<Vec<Rol>> {
+    let prefijo = p::ruta_brainstorm(id, "");
+    let mut por_ronda: std::collections::BTreeMap<&str, Vec<&str>> =
+        std::collections::BTreeMap::new();
+    for nombre in rutas.iter().filter_map(|r| r.strip_prefix(&prefijo)) {
+        let Some(resto) = nombre
+            .strip_prefix("peticion-")
+            .and_then(|n| n.strip_suffix(".md"))
+        else {
+            continue;
+        };
+        if let Some((rol, n)) = resto.rsplit_once('-') {
+            por_ronda.entry(n).or_default().push(rol);
+        }
+    }
+    por_ronda
+        .values()
+        .map(|ids| {
+            roles::todos()
+                .filter(|r| ids.contains(&r.id))
+                .copied()
+                .collect()
+        })
+        .collect()
+}
+
+impl Operacion for Aprobar {
+    fn nombre(&self) -> &str {
+        NOMBRE
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default()
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // La firma es de ahora y de la persona: nunca queda pendiente.
+        true
+    }
+
+    fn sello(&self) -> Sello {
+        Sello::Firmado {
+            trailers: vec![(PLAN_BLOB, p::ruta_plan_final(&self.id))],
+        }
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.dueno.borrow_mut() = None;
+        let (mut t, mut cuerpo) = tarea_de(estado, &self.id)?;
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            NOMBRE,
+            ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
+        }
+        // Ya aprobado: nada que escribir ni que firmar otra vez.
+        if t.plan_aprobado {
+            return Ok(Vec::new());
+        }
+        if t.estado != EN_DISENO {
+            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
+        }
+        if cuerpo.en_pausa() {
+            return Err(self.rechazo("pausada", Mensaje::nuevo(&txt::APROBAR_PAUSADA)));
+        }
+        let ruta = p::ruta_plan_final(&self.id);
+        let Some(plan) = estado.leer(&ruta)? else {
+            return Err(self.rechazo("sin-plan", Mensaje::nuevo(&txt::SIN_PLAN)));
+        };
+        if es_esqueleto(&plan, &t, &estado.rutas()) {
+            return Err(self.rechazo("plan-esqueleto", Mensaje::nuevo(&txt::PLAN_ESQUELETO)));
+        }
+        let modo = modo_congelado(&cuerpo).unwrap_or_else(|| self.modo.to_owned());
+        let instante = instante(self.ahora);
+        t.plan_aprobado = true;
+        instante
+            .get(..10)
+            .unwrap_or_default()
+            .clone_into(&mut t.actualizado);
+        cuerpo.anadir_transicion(&Transicion {
+            fecha: &instante,
+            fase: FASE,
+            modo: &modo,
+            decidido_por: PERSONA,
+            version: crate::VERSION,
+        });
+        Ok(vec![Cambio::nuevo(
+            format!("{}/tarea.md", self.id),
+            t.escribir(&cuerpo),
+        )])
+    }
+}
+
+/// `approve --pedir-cambios`: cerrar la ronda mayor con lo que hay que
+/// cambiar del plan.
+#[derive(Debug)]
+pub struct PedirCambios {
+    id: String,
+    asignado: Asignado,
+    contenido: String,
+    ronda: RefCell<Option<u32>>,
+    dueno: RefCell<Option<String>>,
+}
+
+impl PedirCambios {
+    /// Los cambios de `contenido` (CRLF pasa a LF) para la tarea `id`, por
+    /// `quien` (persona o agente).
+    pub fn nueva(id: &str, quien: &Quien, contenido: &str) -> Self {
+        Self {
+            id: id.to_owned(),
+            asignado: quien.asignado(),
+            contenido: contenido.replace("\r\n", "\n"),
+            ronda: RefCell::new(None),
+            dueno: RefCell::new(None),
+        }
+    }
+
+    /// La ronda que cerro (o ya estaba cerrada con lo mismo) el ultimo
+    /// intento.
+    pub fn ronda(&self) -> Option<u32> {
+        *self.ronda.borrow()
+    }
+
+    /// Login de quien lleva la tarea si el ultimo intento la rechazo por
+    /// `de-otra-persona`.
+    pub fn dueno(&self) -> Option<String> {
+        self.dueno.borrow().clone()
+    }
+
+    fn rechazo(&self, motivo: &str, m: Mensaje) -> Fallo {
+        Fallo::rechazada(motivo, m.con("id", self.id.as_str()))
+    }
+}
+
+impl Operacion for PedirCambios {
+    fn nombre(&self) -> &str {
+        NOMBRE_CAMBIOS
+    }
+
+    fn anotacion(&self) -> String {
+        format!(
+            "{NOMBRE_CAMBIOS}|{}|{}|{}",
+            self.id,
+            self.asignado.id().unwrap_or_default(),
+            self.asignado.login().unwrap_or_default()
+        )
+    }
+
+    fn exige_red(&self) -> bool {
+        // El contenido no cabe en la anotacion (como `plan guardar`).
+        true
+    }
+
+    fn aplicar(&self, estado: &dyn Estado) -> Result<Vec<Cambio>, Fallo> {
+        *self.ronda.borrow_mut() = None;
+        *self.dueno.borrow_mut() = None;
+        let (t, _) = tarea_de(estado, &self.id)?;
+        if let Some((fallo, dueno)) = de_otra_persona(
+            &self.id,
+            t.asignado_a.as_ref(),
+            &self.asignado,
+            NOMBRE,
+            ("de-otra-persona", &txt::APROBAR_DE_OTRA_PERSONA),
+        ) {
+            *self.dueno.borrow_mut() = dueno;
+            return Err(fallo);
+        }
+        if t.estado != EN_DISENO {
+            return Err(por_estado(&self.id, &t.estado, NOMBRE, &[EN_DISENO]));
+        }
+        if t.plan_aprobado {
+            return Err(self.rechazo("plan-aprobado", Mensaje::nuevo(&txt::PLAN_APROBADO)));
+        }
+        let (ronda, _, cerrada) = ronda_mayor(&estado.rutas(), &self.id);
+        if ronda == 0 {
+            return Err(self.rechazo("sin-ronda", Mensaje::nuevo(&txt::SIN_RONDA)));
+        }
+        let ruta = p::ruta_brainstorm(&self.id, &p::nombre_cambios_pedidos(ronda));
+        if cerrada {
+            // Lo mismo otra vez: nada que escribir; otra cosa, a la
+            // siguiente ronda.
+            if estado.leer(&ruta)?.as_deref() == Some(self.contenido.as_str()) {
+                *self.ronda.borrow_mut() = Some(ronda);
+                return Ok(Vec::new());
+            }
+            return Err(self.rechazo(
+                "ronda-cerrada",
+                Mensaje::nuevo(&txt::RONDA_CERRADA).con("ronda", ronda.to_string()),
+            ));
+        }
+        *self.ronda.borrow_mut() = Some(ronda);
+        Ok(vec![Cambio::nuevo(ruta, self.contenido.clone())])
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::identidad::Cuenta;
+    use crate::plataforma::Proveedor;
+    use crate::roadmap::tests::EnMemoria;
+    use crate::tarea::Cuerpo;
+    use std::collections::BTreeMap;
+
+    /// 2026-11-19T10:20:30Z.
+    const AHORA: u64 = 1_795_083_630;
+
+    fn cuenta() -> Cuenta {
+        Cuenta {
+            proveedor: Proveedor::GitHub,
+            id: 7,
+            login: "ana".to_owned(),
+        }
+    }
+
+    fn persona() -> Quien {
+        Quien::Persona(cuenta())
+    }
+
+    fn md(id: &str, estado: &str) -> String {
+        let mut t = Tarea::nueva(id, "Uno", "2026-11-01");
+        t.estado = estado.to_owned();
+        let mut c = Cuerpo::con_enunciado("Que haga algo", &["a".to_owned()]);
+        c.anadir_transicion(&Transicion {
+            fecha: "2026-11-18T09:00:00Z",
+            fase: "plan",
+            modo: "semiautomatico",
+            decidido_por: "agente",
+            version: "0.1",
+        });
+        t.escribir(&c)
+    }
+
+    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+        match r {
+            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+            _ => None,
+        }
+    }
+
+    /// Criterio 1: el agente no puede construir un `Aprobar` (RR-04).
+    #[test]
+    fn solo_una_persona_aprueba() {
+        assert!(Aprobar::nueva("TASK-001", &Quien::Agente(cuenta()), AHORA).is_none());
+        assert!(Aprobar::nueva("TASK-001", &persona(), AHORA).is_some());
+    }
+
+    /// Criterio 3 (unitario): aprueba un plan redactado con la fila de la
+    /// persona y el modo congelado; el esqueleto, otra persona, el estado,
+    /// la pausa y la falta de plan se rechazan; ya aprobado no cambia nada.
+    #[test]
+    fn aplicar_aprueba_y_rechaza() {
+        let roles = roles::seleccionar(1, &[]);
+        let esqueleto = p::plan_final("TASK-002", "Uno", &roles);
+        let pf = |id: &str| format!("{id}/planificacion/plan-final.md");
+        let otra = md("TASK-003", "en-diseno")
+            .replace("asignado_a: null", "asignado_a: {id: 9, login: \"beto\"}");
+        let e = EnMemoria(BTreeMap::from([
+            ("TASK-001/tarea.md".to_owned(), md("TASK-001", "en-diseno")),
+            (pf("TASK-001"), "# Mi plan\n".to_owned()),
+            ("TASK-002/tarea.md".to_owned(), md("TASK-002", "en-diseno")),
+            (pf("TASK-002"), esqueleto.replace('\n', "\r\n")),
+            ("TASK-003/tarea.md".to_owned(), otra),
+            (pf("TASK-003"), "# Plan\n".to_owned()),
+            (
+                "TASK-004/tarea.md".to_owned(),
+                md("TASK-004", "planificada"),
+            ),
+            (pf("TASK-004"), "# Plan\n".to_owned()),
+            (
+                "TASK-005/tarea.md".to_owned(),
+                md("TASK-005", "en-diseno").replace("plan_aprobado: false", "plan_aprobado: true"),
+            ),
+            ("TASK-006/tarea.md".to_owned(), md("TASK-006", "en-diseno")),
+            (
+                "TASK-007/tarea.md".to_owned(),
+                md("TASK-007", "en-diseno")
+                    + "| 2026-11-18T10:00:00Z | pausa | manual | agente | 0.1 |\n",
+            ),
+            (pf("TASK-007"), "# Plan\n".to_owned()),
+        ]));
+        let aplicar = |id: &str| {
+            let op = Aprobar::nueva(id, &persona(), AHORA)
+                .unwrap_or_else(|| unreachable!())
+                .con_config(&Config::suelo());
+            (op.aplicar(&e), op.dueno())
+        };
+        let (r, _) = aplicar("TASK-001");
+        let rutas: Vec<String> = r
+            .as_ref()
+            .map(|c| c.iter().map(|x| x.ruta.clone()).collect())
+            .unwrap_or_default();
+        assert_eq!(rutas, ["TASK-001/tarea.md"], "{r:?}");
+        let tarea = r
+            .ok()
+            .and_then(|c| c.first().map(|x| x.contenido.clone()))
+            .unwrap_or_default();
+        assert!(tarea.contains("\nplan_aprobado: true\n"), "{tarea}");
+        assert!(tarea.contains("\nactualizado: \"2026-11-19\"\n"), "{tarea}");
+        assert!(
+            tarea.ends_with(&format!(
+                "| 2026-11-19T10:20:30Z | approve | semiautomatico | persona | {} |\n",
+                crate::VERSION
+            )),
+            "{tarea}"
+        );
+        assert_eq!(motivo(&aplicar("TASK-002").0), Some("plan-esqueleto"));
+        let (r, dueno) = aplicar("TASK-003");
+        assert_eq!(
+            (motivo(&r), dueno.as_deref()),
+            (Some("de-otra-persona"), Some("beto"))
+        );
+        assert_eq!(motivo(&aplicar("TASK-004").0), Some("estado-no-valido"));
+        assert_eq!(aplicar("TASK-005").0, Ok(Vec::new()));
+        assert_eq!(motivo(&aplicar("TASK-006").0), Some("sin-plan"));
+        assert_eq!(motivo(&aplicar("TASK-007").0), Some("pausada"));
+        assert_eq!(motivo(&aplicar("TASK-009").0), Some("no-existe"));
+    }
+
+    /// El esqueleto de una ronda con unificador (cuatro roles) tambien se
+    /// reconoce; un plan redactado, no.
+    #[test]
+    fn esqueleto_de_cualquier_juego_de_roles() {
+        let mut t = Tarea::nueva("TASK-001", "Uno", "2026-11-01");
+        t.etiquetas = vec!["wasm".to_owned()];
+        let cinco = roles::seleccionar(4, &t.etiquetas);
+        let cuatro = roles::seleccionar(4, &[]);
+        for r in [&cinco, &cuatro, &Vec::new()] {
+            assert!(es_esqueleto(&p::plan_final("TASK-001", "Uno", r), &t, &[]));
+        }
+        assert!(!es_esqueleto("# Plan\n", &t, &[]));
+        let b = "TASK-001/planificacion/brainstorm/";
+        let rutas = vec![
+            format!("{b}peticion-brainstorm-riesgos-1.md"),
+            format!("{b}peticion-brainstorm-arquitectura-1.md"),
+            format!("{b}peticion-unificador-1.md"),
+        ];
+        let dos: Vec<Rol> = roles::todos()
+            .filter(|r| ["brainstorm-arquitectura", "brainstorm-riesgos"].contains(&r.id))
+            .copied()
+            .collect();
+        assert_eq!(juegos_de_rondas(&rutas, "TASK-001"), vec![dos]);
+    }
+
+    /// `exige_red` y el sello firmado con `Plan-Blob` de `plan-final.md`.
+    #[test]
+    fn exige_red_y_firma_con_plan_blob() {
+        let op = Aprobar::nueva("TASK-001", &persona(), AHORA).unwrap_or_else(|| unreachable!());
+        assert!(op.exige_red());
+        assert_eq!(
+            op.sello(),
+            Sello::Firmado {
+                trailers: vec![(
+                    "Plan-Blob",
+                    "TASK-001/planificacion/plan-final.md".to_owned()
+                )]
+            }
+        );
+        assert_eq!(op.anotacion(), "approve|TASK-001|7|ana");
+        let c = PedirCambios::nueva("TASK-001", &persona(), "x");
+        assert!(c.exige_red());
+        assert_eq!(c.sello(), Sello::Normal);
+    }
+
+    /// D1a: los cambios pedidos van a la ronda mayor; repetirlos no cambia
+    /// nada; otros, con la ronda ya cerrada, se rechazan; sin ronda o con el
+    /// plan aprobado, tambien. El agente puede pedirlos.
+    #[test]
+    fn pedir_cambios_cierra_la_ronda_mayor() {
+        let b = "TASK-001/planificacion/brainstorm/";
+        let mut e = EnMemoria(BTreeMap::from([
+            ("TASK-001/tarea.md".to_owned(), md("TASK-001", "en-diseno")),
+            (format!("{b}peticion-plan-1.md"), "p".to_owned()),
+            (format!("{b}peticion-unificador-2.md"), "u".to_owned()),
+            ("TASK-002/tarea.md".to_owned(), md("TASK-002", "en-diseno")),
+            (
+                "TASK-003/tarea.md".to_owned(),
+                md("TASK-003", "en-diseno").replace("plan_aprobado: false", "plan_aprobado: true"),
+            ),
+        ]));
+        let agente = Quien::Agente(cuenta());
+        let op = PedirCambios::nueva("TASK-001", &agente, "cambia a\r\n");
+        let r = op.aplicar(&e);
+        assert_eq!(
+            r,
+            Ok(vec![Cambio::nuevo(
+                format!("{b}cambios-pedidos-2.md"),
+                "cambia a\n"
+            )])
+        );
+        assert_eq!(op.ronda(), Some(2));
+        e.0.insert(format!("{b}cambios-pedidos-2.md"), "cambia a\n".into());
+        assert_eq!(op.aplicar(&e), Ok(Vec::new()));
+        let otro = PedirCambios::nueva("TASK-001", &agente, "otra cosa\n");
+        assert_eq!(motivo(&otro.aplicar(&e)), Some("ronda-cerrada"));
+        let sin = PedirCambios::nueva("TASK-002", &agente, "x\n");
+        assert_eq!(motivo(&sin.aplicar(&e)), Some("sin-ronda"));
+        let ya = PedirCambios::nueva("TASK-003", &agente, "x\n");
+        assert_eq!(motivo(&ya.aplicar(&e)), Some("plan-aprobado"));
+    }
+}
diff --git a/crates/tc2/src/plan/mod.rs b/crates/tc2/src/plan/mod.rs
index 7b999d0..47d4073 100644
--- a/crates/tc2/src/plan/mod.rs
+++ b/crates/tc2/src/plan/mod.rs
@@ -17,10 +17,22 @@
 //!   rol ([`guardar`]). Destino y fichero se comprueban antes de leer el
 //!   estado (`uso`).
 //!
+//! - `approve [--json] TASK-NNN` (F4-2, RR-04): solo una persona (con
+//!   terminal y token verificado: [`crate::identidad::resolver`]); con el
+//!   agente o sin identidad, `rechazada motivo=identidad rol=persona` antes
+//!   de leer nada. Luego `fetch`, la configuracion y la operacion
+//!   [`Aprobar`], cuyo commit va firmado con la clave de la persona y con el
+//!   trailer `Plan-Blob`; si la firma falla, `rechazada motivo=firma` sin
+//!   publicar nada. `--decidido-por` no existe (F1-10 decision 6): `uso`.
+//! - `approve [--json] TASK-NNN --pedir-cambios <fichero>`: publica
+//!   `brainstorm/cambios-pedidos-<n>.md` en la ronda mayor y la cierra
+//!   ([`PedirCambios`], D1a); la puede lanzar tambien el agente.
+//!
 //! Lanzadas desde un worktree de tarea escriben el estado en el principal
 //! (lo localiza la transaccion). El entorno es el de `start`
 //! ([`crate::toma::Entorno`]).
 
+pub mod aprobar;
 pub mod guardar;
 pub mod heuristica;
 pub mod operacion;
@@ -39,6 +51,7 @@ use crate::i18n::catalogo::plan as txt;
 use crate::plataforma::errores::sanear;
 use crate::toma::Entorno;
 
+pub use aprobar::{Aprobar, PedirCambios};
 pub use guardar::{Guardar, destino_valido};
 pub use operacion::{Forma, Planificar, Resumen};
 
@@ -151,7 +164,120 @@ pub fn plan_en(e: &Entorno<'_>, id: &str, carril: Option<Carril>) -> Informe {
     let i = match paso_plan(e, id, carril) {
         Ok(i) | Err(i) => i,
     };
-    de_la_orden(i, id)
+    de_la_orden(NOMBRE, i, id)
+}
+
+fn uso_approve(motivo: &str) -> Informe {
+    Informe::nuevo(aprobar::NOMBRE, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(&txt::USO_APPROVE)
+}
+
+/// `approve TASK-NNN [--pedir-cambios <fichero>]` (sin `--json`, que quita
+/// `lib.rs`). Nada se lee ni se escribe antes de comprobar los argumentos.
+pub fn orden_approve(args: &[String], e: &Entorno<'_>) -> Informe {
+    let Some((id, resto)) = args.split_first() else {
+        return uso_approve("sin-id");
+    };
+    if !git::es_id(id) {
+        return uso_approve("id-invalido");
+    }
+    let mut fichero = None;
+    let mut it = resto.iter();
+    while let Some(a) = it.next() {
+        match a.as_str() {
+            // F1-10 decision 6: la parada `approve` firmado es fija.
+            x if x == "--decidido-por" || x.starts_with("--decidido-por=") => {
+                return Informe::nuevo(aprobar::NOMBRE, Resultado::Uso)
+                    .con("motivo", "decidido-por")
+                    .diagnostico(&txt::DECIDIDO_POR);
+            }
+            "--pedir-cambios" if fichero.is_none() => match it.next() {
+                Some(f) => fichero = Some(f),
+                None => return uso_approve("sin-fichero"),
+            },
+            _ => return uso_approve("argumentos-de-mas"),
+        }
+    }
+    match fichero {
+        None => aprobar_en(e, id),
+        Some(f) => match leer_fichero(&e.dir.join(f)) {
+            Ok(c) => pedir_cambios_en(e, id, &c),
+            Err(mut i) => {
+                aprobar::NOMBRE.clone_into(&mut i.comando);
+                i
+            }
+        },
+    }
+}
+
+/// `approve` de la tarea `id` en el repo de `e`.
+pub fn aprobar_en(e: &Entorno<'_>, id: &str) -> Informe {
+    let i = match paso_aprobar(e, id) {
+        Ok(i) | Err(i) => i,
+    };
+    de_la_orden(aprobar::NOMBRE, i, id)
+}
+
+/// `approve --pedir-cambios` de `contenido` en la tarea `id` del repo de
+/// `e`.
+pub fn pedir_cambios_en(e: &Entorno<'_>, id: &str, contenido: &str) -> Informe {
+    let i = match paso_pedir_cambios(e, id, contenido) {
+        Ok(i) | Err(i) => i,
+    };
+    de_la_orden(aprobar::NOMBRE, i, id)
+}
+
+/// `rechazada motivo=identidad rol=persona` (RR-04), con lo que diga el
+/// rechazo de la identidad si lo hubo (`cache=`, `cuenta=`...).
+fn sin_persona(rechazo: Option<Informe>) -> Informe {
+    let mut i = Informe::nuevo(aprobar::NOMBRE, Resultado::Rechazada)
+        .con("motivo", "identidad")
+        .con("rol", "persona")
+        .diagnostico(&txt::APROBAR_SIN_PERSONA);
+    for (k, v) in rechazo.map(|r| r.claves).unwrap_or_default() {
+        if k != "motivo" && k != "rol" {
+            i.claves.push((k, v));
+        }
+    }
+    i
+}
+
+fn paso_aprobar(e: &Entorno<'_>, id: &str) -> Result<Informe, Informe> {
+    // Lo primero, quien: el agente (o sin identidad) no llega a leer ni a
+    // escribir nada.
+    let quien = match (e.quien)() {
+        Ok(q) => q,
+        Err(i) if i.resultado == Resultado::Rechazada => return Err(sin_persona(Some(i))),
+        Err(i) => return Err(i),
+    };
+    let Some(op) = Aprobar::nueva(id, &quien, e.ahora) else {
+        return Err(sin_persona(None));
+    };
+    traer(e.git, e.dir)?;
+    let op = op.con_config(&config_de_base(e.git, e.dir)?);
+    let t = escribir(e, &op);
+    match t.resultado {
+        Resultado::Ok | Resultado::SinCambios => Ok(t),
+        _ => Err(con_dueno(t, op.dueno())),
+    }
+}
+
+fn paso_pedir_cambios(e: &Entorno<'_>, id: &str, contenido: &str) -> Result<Informe, Informe> {
+    let quien = (e.quien)()?;
+    traer(e.git, e.dir)?;
+    let op = PedirCambios::nueva(id, &quien, contenido);
+    let t = escribir(e, &op);
+    match t.resultado {
+        Resultado::Ok | Resultado::SinCambios => {
+            let n = op.ronda().unwrap_or_default();
+            Ok(
+                t.con("cambios-pedidos", plantillas::nombre_cambios_pedidos(n))
+                    .con("ronda", n.to_string()),
+            )
+        }
+        _ => Err(con_dueno(t, op.dueno())),
+    }
 }
 
 /// `plan guardar` de `contenido` en `destino` (ya validado) de la tarea
@@ -160,7 +286,7 @@ pub fn guardar_en(e: &Entorno<'_>, id: &str, destino: &str, contenido: &str) ->
     let i = match paso_guardar(e, id, destino, contenido) {
         Ok(i) | Err(i) => i,
     };
-    let mut i = de_la_orden(i, id);
+    let mut i = de_la_orden(NOMBRE, i, id);
     i.claves
         .insert(1, ("destino".to_owned(), Valor::Texto(destino.to_owned())));
     i
@@ -260,11 +386,11 @@ fn con_dueno(mut i: Informe, dueno: Option<String>) -> Informe {
     i
 }
 
-/// El informe como de la orden: el comando, `id=` el primero y sin la clave
-/// `operacion` de la transaccion. Un `sinred` sin `pendiente` dice
+/// El informe como de la orden `nombre`: el comando, `id=` el primero y sin
+/// la clave `operacion` de la transaccion. Un `sinred` sin `pendiente` dice
 /// `pendiente=no` (exige red).
-fn de_la_orden(mut i: Informe, id: &str) -> Informe {
-    NOMBRE.clone_into(&mut i.comando);
+fn de_la_orden(nombre: &str, mut i: Informe, id: &str) -> Informe {
+    nombre.clone_into(&mut i.comando);
     i.claves.retain(|(k, _)| k != "operacion" && k != "id");
     i.claves
         .insert(0, ("id".to_owned(), Valor::Texto(id.to_owned())));
@@ -278,6 +404,7 @@ fn de_la_orden(mut i: Informe, id: &str) -> Informe {
 mod tests {
     use super::*;
     use crate::i18n::Idioma;
+    use crate::identidad::Quien;
 
     /// Carpeta temporal propia del test: se borra al acabar.
     struct Carpeta(std::path::PathBuf);
@@ -288,6 +415,53 @@ mod tests {
         }
     }
 
+    fn sin_catalogo(_: &str) -> Option<Box<dyn Operacion>> {
+        None
+    }
+
+    /// Criterio 1 de TASK-037 (unitario): con el agente o sin identidad,
+    /// `approve rechazada motivo=identidad rol=persona` y ni una llamada a
+    /// git (ni transaccion ni `commit-tree`); el rechazo de la identidad
+    /// conserva su `cache=`.
+    #[test]
+    fn approve_sin_persona_no_toca_nada() {
+        use crate::identidad::Cuenta;
+        use crate::plataforma::Proveedor;
+        let agente = || -> Result<Quien, Informe> {
+            Ok(Quien::Agente(Cuenta {
+                proveedor: Proveedor::GitHub,
+                id: 7,
+                login: "ana".to_owned(),
+            }))
+        };
+        let sin_cache = || -> Result<Quien, Informe> {
+            Err(Informe::nuevo("identidad", Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("cache", "falta"))
+        };
+        let quienes: [&dyn Fn() -> Result<Quien, Informe>; 2] = [&agente, &sin_cache];
+        for (quien, esperado) in quienes.into_iter().zip([
+            "approve rechazada id=TASK-001 motivo=identidad rol=persona",
+            "approve rechazada id=TASK-001 motivo=identidad rol=persona cache=falta",
+        ]) {
+            let g = Git::nuevo();
+            let dir = std::env::temp_dir();
+            let e = Entorno {
+                dir: &dir,
+                git: &g,
+                quien,
+                catalogo: &sin_catalogo,
+                ahora: 0,
+            };
+            let i = orden_approve(&["TASK-001".to_owned()], &e);
+            assert_eq!(i.linea(), esperado);
+            assert_eq!(i.resultado.codigo(), 3);
+            assert!(g.registro().is_empty(), "{:?}", g.registro());
+            let d = i.diagnostico_en(Idioma::Es).unwrap_or_default();
+            assert!(d.contains("RR-04"), "{d}");
+        }
+    }
+
     /// El uso no anuncia `--carril errata`: `plan` nunca la acepta (todo
     /// tipo declara fix o funcionalidad y el carril solo sube).
     #[test]
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
index c1b7b3c..cc7aba7 100644
--- a/crates/tc2/src/plan/operacion.rs
+++ b/crates/tc2/src/plan/operacion.rs
@@ -20,10 +20,11 @@
 //!   peticion por rol y `peticion-unificador-<n>.md`).
 //! - Ronda: la mayor con testigo (`peticion-unificador-<n>.md` o
 //!   `peticion-plan-<n>.md`) en `TASK-NNN/planificacion/brainstorm/`. Si sus
-//!   peticiones son exactamente las que tocan ahora, la ronda sigue abierta
-//!   y no se reescribe; si no (no hay ninguna, o cambio el juego de roles),
-//!   se abre la siguiente. Las rondas solo se anaden: la rama de estado no
-//!   borra ficheros.
+//!   peticiones son exactamente las que tocan ahora y no tiene cambios
+//!   pedidos (`cambios-pedidos-<n>.md`, de `approve --pedir-cambios`), la
+//!   ronda sigue abierta y no se reescribe; si no (no hay ninguna, cambio el
+//!   juego de roles o la persona pidio cambios), se abre la siguiente. Las
+//!   rondas solo se anaden: la rama de estado no borra ficheros.
 //! - `plan-final.md`: el esqueleto de F1-9 solo si no existe; uno que ya
 //!   existe no se pisa.
 //! - `tarea.md`: `estado: en-diseno`, `carril:`, `actualizado` y la fila
@@ -172,8 +173,10 @@ pub fn modo_congelado(cuerpo: &Cuerpo) -> Option<String> {
         .map(str::to_owned)
 }
 
-/// Rondas con testigo y nombres por ronda de `TASK-NNN/planificacion/brainstorm/`.
-fn ronda_mayor(rutas: &[String], id: &str) -> (u32, Vec<String>) {
+/// La mayor ronda con testigo de `TASK-NNN/planificacion/brainstorm/`, sus
+/// peticiones (nombres, en orden) y si esta cerrada por
+/// `approve --pedir-cambios` (`cambios-pedidos-<n>.md`, D1a de TASK-037).
+pub(super) fn ronda_mayor(rutas: &[String], id: &str) -> (u32, Vec<String>, bool) {
     let prefijo = p::ruta_brainstorm(id, "");
     let nombres: Vec<&str> = rutas
         .iter()
@@ -197,7 +200,8 @@ fn ronda_mayor(rutas: &[String], id: &str) -> (u32, Vec<String>) {
         .map(|n| (*n).to_owned())
         .collect();
     de_r.sort();
-    (r, de_r)
+    let cerrada = r >= 1 && nombres.contains(&p::nombre_cambios_pedidos(r).as_str());
+    (r, de_r, cerrada)
 }
 
 /// Las peticiones de una ronda: nombre y contenido, en orden de nombre.
@@ -316,7 +320,7 @@ impl Operacion for Planificar {
         let instante = instante(self.ahora);
         let fecha = instante.get(..10).unwrap_or_default().to_owned();
         let rutas = estado.rutas();
-        let (mayor, de_mayor) = ronda_mayor(&rutas, &self.id);
+        let (mayor, de_mayor, cerrada) = ronda_mayor(&rutas, &self.id);
         let datos = |ronda: u32| Datos {
             id: &self.id,
             titulo: &t.titulo,
@@ -334,7 +338,9 @@ impl Operacion for Planificar {
                 .map(|(n, _)| n)
                 .collect()
         };
-        let abierta = mayor >= 1 && nombres(mayor) == de_mayor;
+        // Con cambios pedidos, la ronda esta cerrada aunque los roles sean
+        // los mismos: se abre la siguiente (D1a de TASK-037).
+        let abierta = mayor >= 1 && !cerrada && nombres(mayor) == de_mayor;
         let ronda = if abierta { mayor } else { mayor + 1 };
         let mut cambios = Vec::new();
         if !abierta {
@@ -542,6 +548,44 @@ mod tests {
         );
     }
 
+    /// D1a de TASK-037: con `cambios-pedidos-1.md` la ronda 1 esta cerrada
+    /// y `plan` con los mismos roles abre la ronda 2 (sin pisar el plan);
+    /// sin el testigo, repetir no cambia nada. Un testigo de otra ronda no
+    /// cierra la mayor.
+    #[test]
+    fn cambios_pedidos_cierran_la_ronda() {
+        let c = Config::suelo();
+        let mut e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
+        let aplicar = |e: &mut EnMemoria| {
+            let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
+            let r = op.aplicar(e);
+            for x in r.as_ref().map(Vec::as_slice).unwrap_or_default() {
+                e.0.insert(x.ruta.clone(), x.contenido.clone());
+            }
+            (r, op.resumen())
+        };
+        assert_eq!(rutas(&aplicar(&mut e).0).len(), 3);
+        assert_eq!(aplicar(&mut e).0, Ok(Vec::new()));
+        let b = "TASK-001/planificacion/brainstorm/";
+        e.0.insert(format!("{b}cambios-pedidos-0.md"), "x\n".into());
+        assert_eq!(aplicar(&mut e).0, Ok(Vec::new()));
+        e.0.insert(format!("{b}cambios-pedidos-1.md"), "cambia esto\n".into());
+        let (r, s) = aplicar(&mut e);
+        assert_eq!(
+            rutas(&r),
+            [
+                format!("{b}peticion-plan-2.md"),
+                "TASK-001/tarea.md".to_owned()
+            ]
+        );
+        assert_eq!(
+            s.map(|s| (s.ronda, s.peticiones)),
+            Some((2, vec!["peticion-plan-2.md".to_owned()]))
+        );
+        // La ronda 2 sigue abierta mientras nadie pida cambios en ella.
+        assert_eq!(aplicar(&mut e).0, Ok(Vec::new()));
+    }
+
     /// Lo que no se planifica: enunciado que no vale, estado, otra persona.
     #[test]
     fn rechazos() {
diff --git a/crates/tc2/src/plan/plantillas.rs b/crates/tc2/src/plan/plantillas.rs
index 2adeef2..640392a 100644
--- a/crates/tc2/src/plan/plantillas.rs
+++ b/crates/tc2/src/plan/plantillas.rs
@@ -72,6 +72,14 @@ pub fn nombre_peticion_redaccion(ronda: u32) -> String {
     format!("peticion-plan-{ronda}.md")
 }
 
+/// `cambios-pedidos-<ronda>.md`: lo que la persona pide cambiar del plan
+/// de esa ronda (`tc2 approve --pedir-cambios`). Es el testigo de que la
+/// ronda esta cerrada: el siguiente `plan` abre la siguiente (D1a de
+/// TASK-037).
+pub fn nombre_cambios_pedidos(ronda: u32) -> String {
+    format!("cambios-pedidos-{ronda}.md")
+}
+
 /// `TASK-NNN/planificacion/plan-final.md`.
 pub fn ruta_plan_final(id: &str) -> String {
     format!("{id}/{PLANIFICACION}/{PLAN_FINAL}")
````

## Excluido del diff (3 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/{01-en-diseno => 02-en-curso}/TASK-037/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-037/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-037/tarea.md                                    | 8 +++++++-
 3 files changed, 7 insertions(+), 1 deletion(-)
````

## Tests del diff (1 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/aprobar.rs"
````

````
crates/tc2/tests/aprobar.rs | 373 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 373 insertions(+)
````
