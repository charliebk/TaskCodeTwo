# Peticion de revision — TASK-142 (ronda 1)

- Tarea: TASK-142 — Auditoria 4D: A4 git, rama de estado y toma (9 IMPORTANTE)
- Rama revisada: fix/task-142-auditoria-4d-a4-git-rama-de-estado-y-tom
- Rama base: develop
- Commit revisado (HEAD): d472e5db9a48898501273db538aa655aaf828c5f
- Fecha: 2026-11-25
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-142 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-142/tarea.md
- tareas/03-en-revision/TASK-142/planificacion/
- crates/tc2/src/estado/transaccion.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/guardar.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/git/rama.rs
- crates/tc2/src/git/remoto.rs
- crates/tc2/src/git/tag.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/comun/ops.rs
- crates/tc2/tests/convivencia_016.rs
- crates/tc2/tests/estado_bordes.rs
- crates/tc2/tests/estado_casos.rs
- crates/tc2/tests/estado_concurrencia.rs
- crates/tc2/tests/frescura.rs
- crates/tc2/tests/git_casos_limite.rs
- crates/tc2/tests/git_push.rs
- crates/tc2/tests/roadmap.rs

## Commits a revisar (git log develop..HEAD)

````
d472e5d docs(TASK-142): resultado, criterios y autorrevision
deffddc fix(TASK-142): A4-10 es_prohibido despliega las opciones cortas agrupadas
c06d185 fix(TASK-142): A4-10 test en rojo: opciones cortas agrupadas con una prohibida
b52ed8d fix(TASK-142): A4-08 ayudantes de los tests de estado en tests/comun
6b682dd fix(TASK-142): A4-06 DestinoWt comparte arranque y carpeta ocupada de crear y reanudar
41a5bc5 fix(TASK-142): A4-04 un solo informe de publicar fallido para push, tag y actualizar-rama
204f3f9 fix(TASK-142): A4-03 aplicar_en para intento y sin_red de la transaccion
68bd853 fix(TASK-142): A4-07 un solo analizador de git status --porcelain -z
bb0afbf fix(TASK-142): A4-05 hay_remoto es url_origin(..)?.is_some()
6360f38 fix(TASK-142): A4-02 un solo envoltorio para lanzar git y traducir el fallo
53bc8bf fix(TASK-142): A4-01 una sola consulta de ramas de tarea y una sola clasificacion del fallo del remoto
a48092c chore(TASK-142): tarea en curso
````

## Diff de tu dominio (18 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/estado/transaccion.rs b/crates/tc2/src/estado/transaccion.rs
index 7ea6a95..e43d469 100644
--- a/crates/tc2/src/estado/transaccion.rs
+++ b/crates/tc2/src/estado/transaccion.rs
@@ -350,24 +350,8 @@ fn intento(
             }
         }
     }
-    match op.aplicar(&vista) {
-        Ok(cambios) => {
-            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
-                return Ok(Intento::Hecho(i));
-            }
-            vista.superponer(cambios);
-        }
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => {
-            return Ok(Intento::Hecho(
-                informe(Resultado::Rechazada, op)
-                    .con("motivo", motivo)
-                    .diagnostico(diagnostico),
-            ));
-        }
-        Err(Fallo::Error(d)) => return Err(error(d)),
+    if let Some(i) = aplicar_en(&mut vista, op)? {
+        return Ok(i);
     }
     anotaciones.push(anotacion.to_owned());
     if let Some(generar) = c.roadmap {
@@ -454,6 +438,29 @@ fn choque(rutas: &[String], cambios: &[Cambio]) -> Option<(String, String)> {
     None
 }
 
+/// Aplica `op` sobre `vista` y superpone sus cambios. `Some` si la
+/// operacion termina ahi (no procede, o sus cambios chocan por mayusculas).
+fn aplicar_en(vista: &mut Vista<'_>, op: &dyn Operacion) -> Result<Option<Intento>, Informe> {
+    match op.aplicar(vista) {
+        Ok(cambios) => {
+            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
+                return Ok(Some(Intento::Hecho(i)));
+            }
+            vista.superponer(cambios);
+            Ok(None)
+        }
+        Err(Fallo::Rechazada {
+            motivo,
+            diagnostico,
+        }) => Ok(Some(Intento::Hecho(
+            informe(Resultado::Rechazada, op)
+                .con("motivo", motivo)
+                .diagnostico(diagnostico),
+        ))),
+        Err(Fallo::Error(d)) => Err(error(d)),
+    }
+}
+
 /// `rechazada motivo=ruta-duplicada` si los cambios de la operacion chocan.
 fn duplicada(rutas: &[String], cambios: &[Cambio], op: &dyn Operacion) -> Option<Informe> {
     choque(rutas, cambios).map(|(nueva, otra)| {
@@ -530,24 +537,8 @@ fn sin_red(
         None => commit_de(git, principal, OP, &rama_remota())?,
     };
     let mut vista = Vista::nueva(git, principal, conocida.clone())?;
-    match op.aplicar(&vista) {
-        Ok(cambios) => {
-            if let Some(i) = duplicada(&vista.rutas(), &cambios, op) {
-                return Ok(Intento::Hecho(i));
-            }
-            vista.superponer(cambios);
-        }
-        Err(Fallo::Rechazada {
-            motivo,
-            diagnostico,
-        }) => {
-            return Ok(Intento::Hecho(
-                informe(Resultado::Rechazada, op)
-                    .con("motivo", motivo)
-                    .diagnostico(diagnostico),
-            ));
-        }
-        Err(Fallo::Error(d)) => return Err(error(d)),
+    if let Some(i) = aplicar_en(&mut vista, op)? {
+        return Ok(i);
     }
     let mensaje = mensaje(op.nombre(), &[anotacion.to_owned()]);
     let Some(sha) = construir_commit(
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 71c4375..836a65f 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -10,7 +10,7 @@ use std::sync::atomic::{AtomicUsize, Ordering};
 
 use super::puerto::{HASH_OBJECT_STDIN, UPDATE_INDEX_STDIN};
 use super::rama::error_de_fichero;
-use super::{Git, Paso, Subcomando, correr, exigir, git_fallo};
+use super::{Git, Paso, Subcomando, correr, exigida, exigir, lanzado};
 use crate::contrato::{Informe, Resultado};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::git as txt;
@@ -132,22 +132,12 @@ pub(crate) fn leer_blobs(
         std::collections::HashMap::new();
     if !en_lote.is_empty() {
         let entrada: String = en_lote.iter().map(|r| format!("{commit}:{r}\n")).collect();
-        let s = git
-            .ejecutar_con_entrada(dir, Subcomando::CatFile, &["--batch"], entrada.as_bytes())
-            .map_err(|e| {
-                Informe::nuevo(op, Resultado::Error)
-                    .con("git", Subcomando::CatFile.nombre())
-                    .diagnostico(e.mensaje())
-            })?;
-        if s.codigo != 0 {
-            return Err(Informe::nuevo(op, Resultado::Error)
-                .con("git", Subcomando::CatFile.nombre())
-                .diagnostico(
-                    Mensaje::nuevo(&txt::GIT_FALLO)
-                        .con("subcomando", Subcomando::CatFile.nombre())
-                        .con("error", s.stderr.lines().next().unwrap_or_default()),
-                ));
-        }
+        let sub = Subcomando::CatFile;
+        let s = exigida(
+            op,
+            sub,
+            git.ejecutar_con_entrada(dir, sub, &["--batch"], entrada.as_bytes()),
+        )?;
         let contenidos = partir_batch(&s.stdout, en_lote.len()).ok_or_else(|| {
             Informe::nuevo(op, Resultado::Error).diagnostico(&txt::CAT_FILE_INCOMPLETO)
         })?;
@@ -248,20 +238,7 @@ fn con_indice(
     args: &[&str],
     indice: &Path,
 ) -> Paso<String> {
-    let s = git
-        .ejecutar_con_indice(dir, sub, args, indice)
-        .map_err(|e| {
-            Informe::nuevo(op, Resultado::Error)
-                .con("git", sub.nombre())
-                .diagnostico(e.mensaje())
-        })?;
-    if s.exito() {
-        Ok(s.stdout)
-    } else {
-        Err(Informe::nuevo(op, Resultado::Error)
-            .con("git", sub.nombre())
-            .diagnostico(git_fallo(sub, &s)))
-    }
+    Ok(exigida(op, sub, git.ejecutar_con_indice(dir, sub, args, indice))?.stdout)
 }
 
 /// `hash-object` (sin indice) o `update-index` (con `indice`) en su unica
@@ -278,23 +255,7 @@ fn con_entrada(
         None => git.ejecutar_con_entrada(dir, sub, HASH_OBJECT_STDIN, entrada),
         Some(i) => git.ejecutar_con_indice_y_entrada(dir, sub, UPDATE_INDEX_STDIN, i, entrada),
     };
-    let s = r.map_err(|e| {
-        Informe::nuevo(op, Resultado::Error)
-            .con("git", sub.nombre())
-            .diagnostico(e.mensaje())
-    })?;
-    if s.codigo == 0 {
-        Ok(s.stdout)
-    } else {
-        let s = super::SalidaGit {
-            codigo: s.codigo,
-            stdout: String::new(),
-            stderr: s.stderr,
-        };
-        Err(Informe::nuevo(op, Resultado::Error)
-            .con("git", sub.nombre())
-            .diagnostico(git_fallo(sub, &s)))
-    }
+    Ok(exigida(op, sub, r)?.stdout)
 }
 
 /// Escribe en el almacen de objetos los ficheros de `rutas` (una ruta por
@@ -433,11 +394,7 @@ pub(crate) fn construir_commit_sellado(
         sellado.push_str(&format!("{clave}: {}\n", blob.trim()));
     }
     args.extend(["-m", &sellado]);
-    let s = git.ejecutar_firmado(dir, &args).map_err(|e| {
-        Informe::nuevo(op, Resultado::Error)
-            .con("git", Subcomando::CommitTree.nombre())
-            .diagnostico(e.mensaje())
-    })?;
+    let s = lanzado(op, Subcomando::CommitTree, git.ejecutar_firmado(dir, &args))?;
     if !s.exito() {
         return Err(Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "firma")
@@ -446,20 +403,13 @@ pub(crate) fn construir_commit_sellado(
     Ok(Some(s.stdout.trim().to_owned()))
 }
 
-/// Si `origin` esta configurado. `ls-remote --get-url` no habla con el
-/// remoto: sin `origin`, devuelve el nombre tal cual.
+/// Si `origin` esta configurado (sin hablar con el remoto).
 pub(crate) fn hay_remoto(git: &Git, dir: &Path, op: &str) -> Paso<bool> {
-    let s = correr(
-        git,
-        dir,
-        op,
-        Subcomando::LsRemote,
-        &["--get-url", super::REMOTO],
-    )?;
-    Ok(s.exito() && s.stdout.trim() != super::REMOTO && !s.stdout.trim().is_empty())
+    Ok(url_origin(git, dir, op)?.is_some())
 }
 
-/// URL de `origin`, si esta configurado (sin hablar con el remoto).
+/// URL de `origin`, si esta configurado. `ls-remote --get-url` no habla con
+/// el remoto: sin `origin`, devuelve el nombre tal cual.
 pub(crate) fn url_origin(git: &Git, dir: &Path, op: &str) -> Paso<Option<String>> {
     let s = correr(
         git,
@@ -590,24 +540,18 @@ mod tests_stdin {
         let _ = std::fs::remove_dir_all(&d);
     }
 
-    /// Git falso que delega en el real salvo `hash-object`, al que responde
-    /// con un unico sha sea cual sea el numero de rutas. El subcomando es el
-    /// quinto argumento (`-c color.ui=never -c core.quotepath=false <sub>`).
-    fn git_que_pierde_shas(d: &Path) -> Git {
-        let sha = "0123456789012345678901234567890123456789";
+    /// Git falso que delega en el real salvo lo que intercepte su guion:
+    /// `cmd` en Windows, `sh` en el resto, que van antes de la llamada al
+    /// git real. El subcomando es el quinto argumento (`-c color.ui=never -c
+    /// core.quotepath=false <sub>`).
+    fn git_falso(d: &Path, cmd: &str, sh: &str) -> Git {
         if cfg!(windows) {
             let f = d.join("git-falso.cmd");
-            let guion = format!(
-                "@echo off\r\nif \"%5\"==\"hash-object\" (\r\n  echo {sha}\r\n  exit /b 0\r\n)\r\ngit %*\r\n"
-            );
-            std::fs::write(&f, guion).unwrap();
+            std::fs::write(&f, format!("@echo off\r\n{cmd}git %*\r\n")).unwrap();
             Git::con_binario(f)
         } else {
             let f = d.join("git-falso.sh");
-            let guion = format!(
-                "#!/bin/sh\nif [ \"$5\" = hash-object ]; then cat >/dev/null; echo {sha}; exit 0; fi\nexec git \"$@\"\n"
-            );
-            std::fs::write(&f, guion).unwrap();
+            std::fs::write(&f, format!("#!/bin/sh\n{sh}exec git \"$@\"\n")).unwrap();
             #[cfg(unix)]
             {
                 use std::os::unix::fs::PermissionsExt;
@@ -617,6 +561,36 @@ mod tests_stdin {
         }
     }
 
+    /// Git falso que responde a `hash-object` con un unico sha sea cual sea
+    /// el numero de rutas.
+    fn git_que_pierde_shas(d: &Path) -> Git {
+        let sha = "0123456789012345678901234567890123456789";
+        git_falso(
+            d,
+            &format!("if \"%5\"==\"hash-object\" (\r\n  echo {sha}\r\n  exit /b 0\r\n)\r\n"),
+            &format!("if [ \"$5\" = hash-object ]; then cat >/dev/null; echo {sha}; exit 0; fi\n"),
+        )
+    }
+
+    /// A4-02: un `cat-file --batch` que falla con stderr que empieza por una
+    /// linea en blanco da en el diagnostico la primera linea no vacia, como
+    /// cualquier otra llamada a git.
+    #[test]
+    fn leer_blobs_diagnostica_la_primera_linea_no_vacia() {
+        let d = repo("leer-blobs");
+        let git = git_falso(
+            &d,
+            "if \"%5\"==\"cat-file\" (\r\n  >&2 echo.\r\n  >&2 echo fatal: x\r\n  exit /b 1\r\n)\r\n",
+            "if [ \"$5\" = cat-file ]; then cat >/dev/null; printf '\\nfatal: x\\n' >&2; exit 1; fi\n",
+        );
+        let i = leer_blobs(&git, &d, "prueba", "HEAD", &["a.md"]).expect_err("cat-file falla");
+        assert_eq!(i.resultado, Resultado::Error);
+        assert!(i.linea().contains("git=cat-file"), "{}", i.linea());
+        let diag = i.diagnostico_en(Idioma::Es).unwrap_or_default();
+        assert!(diag.contains("fatal: x"), "{diag}");
+        let _ = std::fs::remove_dir_all(&d);
+    }
+
     /// TASK-127: si `hash-object` devuelve menos shas que ficheros,
     /// `construir_commit` falla con `HASH_OBJECT_INCOMPLETO` y no construye
     /// ningun commit (emparejar por orden daria a cada ruta otro contenido).
diff --git a/crates/tc2/src/git/guardar.rs b/crates/tc2/src/git/guardar.rs
index 7d7588d..6e68aa4 100644
--- a/crates/tc2/src/git/guardar.rs
+++ b/crates/tc2/src/git/guardar.rs
@@ -6,7 +6,7 @@
 use std::path::{Component, Path};
 
 use super::remoto::{merge_abierto, sin_resolver};
-use super::{Git, Paso, Subcomando, commit_de, entradas_status, exigir};
+use super::{Git, Paso, Subcomando, commit_de, entradas_status, exigir, registros_status};
 use crate::contrato::{Informe, Resultado, corto};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::git as txt;
@@ -154,21 +154,13 @@ fn repartir<'a>(
 }
 
 /// Como [`entradas_status`], pero un renombrado tambien aporta su ruta de
-/// origen, como borrada en el indice (`D `).
+/// origen, como borrada en el indice (`D `). La copia (`C`) tambien trae el
+/// origen, pero el origen sigue ahi.
 fn entradas_con_origen(salida: &str) -> Vec<(String, String)> {
     let mut v = Vec::new();
-    let mut campos = salida.split('\0').filter(|c| !c.is_empty());
-    while let Some(c) = campos.next() {
-        let xy = c.get(..2).unwrap_or(c).to_owned();
-        let ruta = c.get(3..).unwrap_or("").to_owned();
-        // La copia (`C`) tambien trae el origen, pero el origen sigue ahi.
-        let origen = if xy.starts_with('R') || xy.starts_with('C') {
-            campos.next()
-        } else {
-            None
-        };
+    for (xy, ruta, origen) in registros_status(salida) {
         if let (Some(o), true) = (origen, xy.starts_with('R')) {
-            v.push(("D ".to_owned(), o.to_owned()));
+            v.push(("D ".to_owned(), o));
         }
         v.push((xy, ruta));
     }
@@ -223,7 +215,23 @@ fn relativa_a_la_raiz(prefijo: &str, ruta: &str) -> Option<String> {
 
 #[cfg(test)]
 mod tests {
-    use super::relativa_a_la_raiz;
+    use super::{entradas_con_origen, relativa_a_la_raiz};
+
+    /// Un renombrado (`R`) aporta su origen como `D `; una copia (`C`), no.
+    #[test]
+    fn el_origen_cuenta_solo_en_los_renombrados() {
+        let s = "R  nuevo.txt\0viejo.txt\0C  copia.txt\0orig.txt\0 M a.txt\0";
+        let par = |xy: &str, r: &str| (xy.to_owned(), r.to_owned());
+        assert_eq!(
+            entradas_con_origen(s),
+            vec![
+                par("D ", "viejo.txt"),
+                par("R ", "nuevo.txt"),
+                par("C ", "copia.txt"),
+                par(" M", "a.txt"),
+            ]
+        );
+    }
 
     #[test]
     fn rutas_relativas_a_la_raiz() {
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index e6ff3f8..6545076 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -37,6 +37,8 @@ pub use worktree::{quitar_worktree, worktree_publicado};
 
 use std::path::{Component, Path, PathBuf};
 
+use puerto::{Salida, primera_linea};
+
 use crate::contrato::{Informe, Resultado};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::git as txt;
@@ -50,6 +52,28 @@ pub const CARPETA_WORKTREES: &str = crate::huella::CARPETA_WORKTREES;
 /// Resultado interno: el `Err` ya es el informe que se devuelve.
 pub(crate) type Paso<T> = Result<T, Informe>;
 
+/// La salida de una llamada a `git` (cualquier forma del puerto), con el
+/// fallo al lanzar convertido en un informe `error` con `git=<sub>`.
+pub(crate) fn lanzado<S>(op: &str, sub: Subcomando, r: Result<S, ErrorGit>) -> Paso<S> {
+    r.map_err(|e| {
+        Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(e.mensaje())
+    })
+}
+
+/// Como [`lanzado`], pero un codigo distinto de 0 tambien es `error`.
+pub(crate) fn exigida<S: Salida>(op: &str, sub: Subcomando, r: Result<S, ErrorGit>) -> Paso<S> {
+    let s = lanzado(op, sub, r)?;
+    if s.codigo() == 0 {
+        Ok(s)
+    } else {
+        Err(Informe::nuevo(op, Resultado::Error)
+            .con("git", sub.nombre())
+            .diagnostico(git_fallo(sub, &s)))
+    }
+}
+
 /// Lanza `git` y convierte el fallo al lanzar en un informe `error`.
 pub(crate) fn correr(
     git: &Git,
@@ -58,18 +82,14 @@ pub(crate) fn correr(
     sub: Subcomando,
     args: &[&str],
 ) -> Paso<SalidaGit> {
-    git.ejecutar(dir, sub, args).map_err(|e| {
-        Informe::nuevo(op, Resultado::Error)
-            .con("git", sub.nombre())
-            .diagnostico(e.mensaje())
-    })
+    lanzado(op, sub, git.ejecutar(dir, sub, args))
 }
 
 /// Diagnostico de un subcomando de git que termino con error.
-pub(crate) fn git_fallo(sub: Subcomando, s: &SalidaGit) -> Mensaje {
+pub(crate) fn git_fallo(sub: Subcomando, s: &impl Salida) -> Mensaje {
     Mensaje::nuevo(&txt::GIT_FALLO)
         .con("subcomando", sub.nombre())
-        .con("error", s.primera_linea_error())
+        .con("error", primera_linea(s.stderr()))
 }
 
 /// Como [`correr`], pero un codigo distinto de 0 tambien es `error`.
@@ -80,14 +100,7 @@ pub(crate) fn exigir(
     sub: Subcomando,
     args: &[&str],
 ) -> Paso<String> {
-    let s = correr(git, dir, op, sub, args)?;
-    if s.exito() {
-        Ok(s.stdout)
-    } else {
-        Err(Informe::nuevo(op, Resultado::Error)
-            .con("git", sub.nombre())
-            .diagnostico(git_fallo(sub, &s)))
-    }
+    Ok(exigida(op, sub, git.ejecutar(dir, sub, args))?.stdout)
 }
 
 /// Commit al que apunta `referencia`, o `None` si no existe.
@@ -213,22 +226,34 @@ fn normalizar(p: &Path) -> Vec<String> {
         .collect()
 }
 
-/// Entradas de `git status --porcelain -z`: (estado XY, ruta). Las de
-/// renombrado traen un segundo campo (la ruta de origen) que se salta.
-pub(crate) fn entradas_status(salida: &str) -> Vec<(String, String)> {
+/// Registros de `git status --porcelain -z`: (estado XY, ruta, origen). Las
+/// de renombrado (`R`) y copia (`C`) traen un segundo campo, la ruta de
+/// origen.
+pub(crate) fn registros_status(salida: &str) -> Vec<(String, String, Option<String>)> {
     let mut v = Vec::new();
     let mut campos = salida.split('\0').filter(|c| !c.is_empty());
     while let Some(c) = campos.next() {
         let xy = c.get(..2).unwrap_or(c).to_owned();
         let ruta = c.get(3..).unwrap_or("").to_owned();
-        if xy.starts_with('R') || xy.starts_with('C') {
-            campos.next();
-        }
-        v.push((xy, ruta));
+        let origen = if xy.starts_with('R') || xy.starts_with('C') {
+            campos.next().map(str::to_owned)
+        } else {
+            None
+        };
+        v.push((xy, ruta, origen));
     }
     v
 }
 
+/// Entradas de `git status --porcelain -z`: (estado XY, ruta), sin la ruta
+/// de origen de los renombrados y las copias.
+pub(crate) fn entradas_status(salida: &str) -> Vec<(String, String)> {
+    registros_status(salida)
+        .into_iter()
+        .map(|(xy, ruta, _)| (xy, ruta))
+        .collect()
+}
+
 /// `TASK-NNN` con al menos tres cifras.
 pub(crate) fn es_id(id: &str) -> bool {
     id.strip_prefix("TASK-")
@@ -251,16 +276,27 @@ mod tests {
 
     #[test]
     fn status_z_con_renombrado_y_espacios() {
-        let s = " M a b.txt\0R  nuevo.txt\0viejo.txt\0?? d/\0";
+        let s = " M a b.txt\0R  nuevo.txt\0viejo.txt\0C  copia.txt\0orig.txt\0?? d/\0";
         let e = entradas_status(s);
         assert_eq!(
             e,
             vec![
                 (" M".to_owned(), "a b.txt".to_owned()),
                 ("R ".to_owned(), "nuevo.txt".to_owned()),
+                ("C ".to_owned(), "copia.txt".to_owned()),
                 ("??".to_owned(), "d/".to_owned()),
             ]
         );
+        let origenes: Vec<Option<String>> = registros_status(s).into_iter().map(|r| r.2).collect();
+        assert_eq!(
+            origenes,
+            vec![
+                None,
+                Some("viejo.txt".to_owned()),
+                Some("orig.txt".to_owned()),
+                None
+            ]
+        );
     }
 
     #[test]
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index bdfa00d..2886973 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -149,12 +149,43 @@ impl SalidaGit {
 
     /// Primera linea no vacia de stderr, para un diagnostico.
     pub fn primera_linea_error(&self) -> String {
-        self.stderr
-            .lines()
-            .map(str::trim)
-            .find(|l| !l.is_empty())
-            .unwrap_or("sin mensaje")
-            .to_owned()
+        primera_linea(&self.stderr)
+    }
+}
+
+/// Primera linea no vacia de `stderr`, recortada, o `sin mensaje`.
+pub(crate) fn primera_linea(stderr: &str) -> String {
+    stderr
+        .lines()
+        .map(str::trim)
+        .find(|l| !l.is_empty())
+        .unwrap_or("sin mensaje")
+        .to_owned()
+}
+
+/// Lo comun de [`SalidaGit`] y [`SalidaCruda`] para traducir un fallo.
+pub(crate) trait Salida {
+    /// Codigo de salida.
+    fn codigo(&self) -> i32;
+    /// Salida de error, tal cual.
+    fn stderr(&self) -> &str;
+}
+
+impl Salida for SalidaGit {
+    fn codigo(&self) -> i32 {
+        self.codigo
+    }
+    fn stderr(&self) -> &str {
+        &self.stderr
+    }
+}
+
+impl Salida for SalidaCruda {
+    fn codigo(&self) -> i32 {
+        self.codigo
+    }
+    fn stderr(&self) -> &str {
+        &self.stderr
     }
 }
 
@@ -553,7 +584,8 @@ pub const UPDATE_INDEX_STDIN: &[&str] = &["--add", "-z", "--index-info"];
 const FIRMA: &str = "-S";
 
 /// Argumentos prohibidos dentro de los subcomandos permitidos (F1-4,
-/// decision 3). Se mira cada argumento entero, no subcadenas (H9). No se
+/// decision 3). Se mira cada argumento entero, no subcadenas (H9), y cada
+/// letra de un grupo de opciones cortas (`-qf` es `-q -f`). No se
 /// miran el texto de un mensaje (`-m <texto>` en `commit` y `tag`) ni las
 /// rutas tras `--` en los subcomandos que las aceptan (`add`, `commit`,
 /// `status`, `diff`): ahi una palabra como `--force` es dato, no opcion.
@@ -610,7 +642,22 @@ fn forma_de_update_ref(args: &[&str]) -> bool {
     }
 }
 
+/// Si `a` esta prohibido en `sub`, entero o, si es un grupo de opciones
+/// cortas (`-xyz`, un solo guion), por cualquiera de sus letras (`-x`, `-y`,
+/// `-z`): git acepta `push -qf` como `push -q -f`. Cierra ante la duda: un
+/// valor pegado (`-mfoo`) tambien se despliega.
 fn es_prohibido(sub: Subcomando, a: &str) -> bool {
+    let grupo = a
+        .strip_prefix('-')
+        .filter(|r| !r.starts_with('-') && r.chars().count() > 1)
+        .unwrap_or_default();
+    es_prohibido_solo(sub, a)
+        || grupo
+            .chars()
+            .any(|c| es_prohibido_solo(sub, &format!("-{c}")))
+}
+
+fn es_prohibido_solo(sub: Subcomando, a: &str) -> bool {
     if a.starts_with("--force") {
         return true;
     }
@@ -834,6 +881,11 @@ mod tests {
             (Subcomando::ReadTree, &["abc"]),
             (Subcomando::UpdateIndex, &["--add", "a"]),
             (Subcomando::WriteTree, &[]),
+            // Opciones cortas agrupadas que contienen una prohibida: git las
+            // acepta (A4-10 de la auditoria 4D).
+            (Subcomando::Push, &["-qf", "origin", "r"]),
+            (Subcomando::Branch, &["-qD", "x"]),
+            (Subcomando::Worktree, &["remove", "-ff", "w"]),
         ];
         for (sub, args) in casos {
             let r = git.ejecutar(Path::new("."), *sub, args);
@@ -850,6 +902,8 @@ mod tests {
         let git = git_sin_binario();
         let permitidos: &[(Subcomando, &[&str])] = &[
             (Subcomando::Push, &["--porcelain", "origin", "a:b"]),
+            // Agrupadas sin ninguna prohibida.
+            (Subcomando::Push, &["-qv", "--porcelain", "origin", "a:b"]),
             (
                 Subcomando::Commit,
                 &["-q", "-m", "--force no se usa", "--only", "--", "a.txt"],
diff --git a/crates/tc2/src/git/rama.rs b/crates/tc2/src/git/rama.rs
index d758bd2..6b63ee6 100644
--- a/crates/tc2/src/git/rama.rs
+++ b/crates/tc2/src/git/rama.rs
@@ -5,7 +5,7 @@ use std::collections::{BTreeMap, BTreeSet};
 use std::path::{Path, PathBuf};
 
 use super::estado::hay_remoto;
-use super::remoto::{FalloRemoto, clasificar_fallo_remoto};
+use super::remoto::consultar_remoto;
 use super::{
     CARPETA_WORKTREES, Git, Paso, REMOTO, Subcomando, commit_de, correr, es_id, exigir, misma_ruta,
     nombre_de_rama, worktree_relativo, worktrees,
@@ -51,8 +51,8 @@ pub fn validar_base(git: &Git, dir: &Path, base: &str) -> Informe {
     }
 }
 
-/// Mira si el ID ya tiene rama, en local o en el remoto (patron
-/// `*/task-nnn-*`). Ocupado → `rechazada motivo=ocupada`.
+/// Mira si el ID ya tiene rama, en local o, si hay `origin`, en el remoto
+/// (patron `*/task-nnn-*`). Ocupado → `rechazada motivo=ocupada`.
 pub fn comprobar_id(git: &Git, dir: &Path, id: &str) -> Informe {
     comprobar_id_salvo(git, dir, id, None)
 }
@@ -86,27 +86,26 @@ fn ramas_ajenas(mut ramas: Vec<String>, propia: Option<&str>) -> Vec<String> {
 }
 
 fn ramas_del_id(git: &Git, dir: &Path, op: &str, id: &str) -> Paso<Vec<String>> {
-    let patron = format!("*/{}-*", id.to_lowercase());
-    let remoto = correr(
-        git,
-        dir,
-        op,
-        Subcomando::LsRemote,
-        &["--heads", REMOTO, &patron],
-    )?;
-    if !remoto.exito() {
-        return Err(Informe::nuevo(op, Resultado::Sinred)
-            .con("remoto", REMOTO)
-            .diagnostico(super::remoto::sin_red_remoto(op)));
-    }
+    ramas_de_tarea(git, dir, op, &format!("*/{}-*", id.to_lowercase()))
+}
+
+/// Ramas que casan con `patron` en local y, si hay `origin`, en el remoto,
+/// ordenadas y sin repetir. Un fallo del remoto se clasifica como en
+/// `fetch`: `sinred` (4), `denegada` (8) o `error` local.
+fn ramas_de_tarea(git: &Git, dir: &Path, op: &str, patron: &str) -> Paso<Vec<String>> {
+    let remoto = if hay_remoto(git, dir, op)? {
+        consultar_remoto(git, dir, op, &["--heads", REMOTO, patron])?
+    } else {
+        String::new()
+    };
     let local = exigir(
         git,
         dir,
         op,
         Subcomando::Branch,
-        &["--list", "--format=%(refname:short)", &patron],
+        &["--list", "--format=%(refname:short)", patron],
     )?;
-    Ok(nombres(&remoto.stdout, &local))
+    Ok(nombres(&remoto, &local))
 }
 
 /// Nombres de rama de la salida de `ls-remote --heads` y de `branch --list
@@ -159,37 +158,7 @@ fn id_de_rama(rama: &str) -> Option<u32> {
 /// `denegada` (8) o `error` local.
 pub fn ids_con_rama(git: &Git, dir: &Path, op: &str) -> Paso<BTreeSet<u32>> {
     let principal = localizar_principal(git, dir)?;
-    let remoto = if hay_remoto(git, &principal, op)? {
-        let s = correr(
-            git,
-            &principal,
-            op,
-            Subcomando::LsRemote,
-            &["--heads", REMOTO, PATRON_TAREAS],
-        )?;
-        if !s.exito() {
-            return Err(match clasificar_fallo_remoto(&s.stderr) {
-                FalloRemoto::SinRed | FalloRemoto::RamaInexistente => {
-                    Informe::nuevo(op, Resultado::Sinred)
-                        .con("remoto", REMOTO)
-                        .diagnostico(super::remoto::sin_red_remoto(op))
-                }
-                FalloRemoto::Local => super::remoto::error_local(op, &s.primera_linea_error()),
-                otro => super::remoto::denegada(op, otro, &s.primera_linea_error()),
-            });
-        }
-        s.stdout
-    } else {
-        String::new()
-    };
-    let local = exigir(
-        git,
-        &principal,
-        op,
-        Subcomando::Branch,
-        &["--list", "--format=%(refname:short)", PATRON_TAREAS],
-    )?;
-    Ok(nombres(&remoto, &local)
+    Ok(ramas_de_tarea(git, &principal, op, PATRON_TAREAS)?
         .iter()
         .filter_map(|r| id_de_rama(r))
         .collect())
@@ -315,67 +284,105 @@ pub fn crear_rama_y_worktree(
     crear(git, dir, id, tipo, slug, base).unwrap_or_else(|i| i)
 }
 
+/// Rama y worktree de una tarea: lo que comparten [`crear_rama_y_worktree`]
+/// y [`reanudar`] (casos b y c de F1-3).
+struct DestinoWt {
+    op: &'static str,
+    rama: String,
+    principal: PathBuf,
+    wt: PathBuf,
+    rel: String,
+}
+
+impl DestinoWt {
+    /// El worktree principal y la carpeta del worktree de la tarea `id`,
+    /// con `rama` ya validada.
+    fn preparar(git: &Git, dir: &Path, op: &'static str, id: &str, rama: String) -> Paso<Self> {
+        let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, op))?;
+        Ok(Self {
+            op,
+            rama,
+            wt: principal.join(CARPETA_WORKTREES).join(id),
+            principal,
+            rel: worktree_relativo(id),
+        })
+    }
+
+    /// `rechazada rama=<rama> motivo=<motivo>`.
+    fn rechazo(&self, motivo: &str) -> Informe {
+        Informe::nuevo(self.op, Resultado::Rechazada)
+            .con("rama", self.rama.as_str())
+            .con("motivo", motivo)
+    }
+
+    /// El worktree registrado en la rama, si lo hay.
+    fn registrado(&self, git: &Git) -> Paso<Option<super::Worktree>> {
+        Ok(worktrees(git, &self.principal, self.op)?
+            .into_iter()
+            .find(|w| w.rama.as_deref() == Some(self.rama.as_str())))
+    }
+
+    /// `rechazada motivo=carpeta-ocupada` si la carpeta ya existe.
+    fn carpeta_ocupada(&self) -> Option<Informe> {
+        self.wt.exists().then(|| {
+            self.rechazo("carpeta-ocupada")
+                .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", self.rel.as_str()))
+        })
+    }
+
+    /// `worktree add -q <antes> <carpeta> <despues>`, con la carpeta de los
+    /// worktrees excluida del `status` del principal.
+    fn anadir(&self, git: &Git, antes: &[&str], despues: &[&str]) -> Paso<()> {
+        excluir_worktrees(git, &self.principal, self.op)?;
+        let wt_txt = ruta_texto(self.op, &self.wt)?;
+        let mut args = vec!["add", "-q"];
+        args.extend_from_slice(antes);
+        args.push(&wt_txt);
+        args.extend_from_slice(despues);
+        exigir(git, &self.principal, self.op, Subcomando::Worktree, &args)?;
+        Ok(())
+    }
+}
+
 fn crear(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str, base: &str) -> Paso<Informe> {
     const OP: &str = "crear-rama-y-worktree";
     let rama = rama_valida(OP, id, tipo, slug)?;
     if base.is_empty() || base.starts_with('-') {
         return Err(Informe::nuevo(OP, Resultado::Uso).con("motivo", "base-invalida"));
     }
-    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
-    let wt = principal.join(CARPETA_WORKTREES).join(id);
-    let rel = worktree_relativo(id);
-    let rechazo = |motivo: &str| {
-        Informe::nuevo(OP, Resultado::Rechazada)
-            .con("rama", rama.as_str())
-            .con("motivo", motivo)
-    };
-    if let Some(w) = worktrees(git, &principal, OP)?
-        .into_iter()
-        .find(|w| w.rama.as_deref() == Some(rama.as_str()))
-    {
-        return Ok(if misma_ruta(&w.ruta, &wt) {
+    let d = DestinoWt::preparar(git, dir, OP, id, rama)?;
+    let (rama, principal) = (d.rama.as_str(), d.principal.as_path());
+    if let Some(w) = d.registrado(git)? {
+        return Ok(if misma_ruta(&w.ruta, &d.wt) {
             Informe::nuevo(OP, Resultado::SinCambios)
-                .con("rama", rama.as_str())
-                .con("worktree", rel)
+                .con("rama", rama)
+                .con("worktree", d.rel.as_str())
         } else {
-            rechazo("rama-en-otro-worktree").diagnostico(&txt::RAMA_EN_OTRO_WORKTREE)
+            d.rechazo("rama-en-otro-worktree")
+                .diagnostico(&txt::RAMA_EN_OTRO_WORKTREE)
         });
     }
-    if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_some() {
-        return Ok(rechazo("rama-existe").diagnostico(&txt::RAMA_YA_EXISTE));
+    if commit_de(git, principal, OP, &format!("refs/heads/{rama}"))?.is_some() {
+        return Ok(d.rechazo("rama-existe").diagnostico(&txt::RAMA_YA_EXISTE));
     }
-    if commit_de(
-        git,
-        &principal,
-        OP,
-        &format!("refs/remotes/{REMOTO}/{rama}"),
-    )?
-    .is_some()
-    {
-        return Ok(rechazo("rama-en-remoto").diagnostico(&txt::RAMA_YA_PUBLICADA));
+    if commit_de(git, principal, OP, &format!("refs/remotes/{REMOTO}/{rama}"))?.is_some() {
+        return Ok(d
+            .rechazo("rama-en-remoto")
+            .diagnostico(&txt::RAMA_YA_PUBLICADA));
     }
-    if wt.exists() {
-        return Ok(rechazo("carpeta-ocupada")
-            .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
+    if let Some(i) = d.carpeta_ocupada() {
+        return Ok(i);
     }
-    let Some(commit) = commit_de(git, &principal, OP, base)? else {
+    let Some(commit) = commit_de(git, principal, OP, base)? else {
         return Ok(Informe::nuevo(OP, Resultado::Rechazada)
             .con("base", base)
             .con("motivo", "base-no-existe")
             .diagnostico(Mensaje::nuevo(&txt::BASE_NO_EXISTE).con("base", base)));
     };
-    excluir_worktrees(git, &principal, OP)?;
-    let wt_txt = ruta_texto(OP, &wt)?;
-    exigir(
-        git,
-        &principal,
-        OP,
-        Subcomando::Worktree,
-        &["add", "-q", "--no-track", "-b", &rama, &wt_txt, &commit],
-    )?;
+    d.anadir(git, &["--no-track", "-b", rama], &[&commit])?;
     Ok(Informe::nuevo(OP, Resultado::Ok)
-        .con("rama", rama.as_str())
-        .con("worktree", rel)
+        .con("rama", rama)
+        .con("worktree", d.rel.as_str())
         .con("base", corto(&commit)))
 }
 
@@ -387,54 +394,39 @@ pub fn reanudar(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Info
 
 fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Paso<Informe> {
     const OP: &str = "reanudar";
-    let rama = rama_valida(OP, id, tipo, slug)?;
-    let principal = localizar_principal(git, dir).map_err(|i| renombrar(i, OP))?;
-    let wt = principal.join(CARPETA_WORKTREES).join(id);
-    let rel = worktree_relativo(id);
-    let rechazo = |motivo: &str| {
-        Informe::nuevo(OP, Resultado::Rechazada)
-            .con("rama", rama.as_str())
-            .con("motivo", motivo)
-    };
+    let d = DestinoWt::preparar(git, dir, OP, id, rama_valida(OP, id, tipo, slug)?)?;
+    let (rama, principal) = (d.rama.as_str(), d.principal.as_path());
     // Un worktree borrado a mano sigue registrado hasta que se poda.
-    exigir(git, &principal, OP, Subcomando::Worktree, &["prune"])?;
+    exigir(git, principal, OP, Subcomando::Worktree, &["prune"])?;
     let mut creado = Vec::new();
-    if commit_de(git, &principal, OP, &format!("refs/heads/{rama}"))?.is_none() {
+    if commit_de(git, principal, OP, &format!("refs/heads/{rama}"))?.is_none() {
         let remota = format!("refs/remotes/{REMOTO}/{rama}");
-        if commit_de(git, &principal, OP, &remota)?.is_none() {
-            return Ok(rechazo("rama-no-existe").diagnostico(&txt::TAREA_SIN_RAMA));
+        if commit_de(git, principal, OP, &remota)?.is_none() {
+            return Ok(d
+                .rechazo("rama-no-existe")
+                .diagnostico(&txt::TAREA_SIN_RAMA));
         }
         exigir(
             git,
-            &principal,
+            principal,
             OP,
             Subcomando::Branch,
-            &["-q", "--no-track", &rama, &remota],
+            &["-q", "--no-track", rama, &remota],
         )?;
         creado.push("rama".to_owned());
     }
-    match worktrees(git, &principal, OP)?
-        .into_iter()
-        .find(|w| w.rama.as_deref() == Some(rama.as_str()))
-    {
-        Some(w) if misma_ruta(&w.ruta, &wt) => {}
+    match d.registrado(git)? {
+        Some(w) if misma_ruta(&w.ruta, &d.wt) => {}
         Some(_) => {
-            return Ok(rechazo("rama-en-otro-worktree").diagnostico(&txt::RAMA_EN_OTRO_WORKTREE));
+            return Ok(d
+                .rechazo("rama-en-otro-worktree")
+                .diagnostico(&txt::RAMA_EN_OTRO_WORKTREE));
         }
         None => {
-            if wt.exists() {
-                return Ok(rechazo("carpeta-ocupada")
-                    .diagnostico(Mensaje::nuevo(&txt::CARPETA_OCUPADA).con("ruta", rel.as_str())));
+            if let Some(i) = d.carpeta_ocupada() {
+                return Ok(i);
             }
-            excluir_worktrees(git, &principal, OP)?;
-            let wt_txt = ruta_texto(OP, &wt)?;
-            exigir(
-                git,
-                &principal,
-                OP,
-                Subcomando::Worktree,
-                &["add", "-q", &wt_txt, &rama],
-            )?;
+            d.anadir(git, &[], &[rama])?;
             creado.push("worktree".to_owned());
         }
     }
@@ -446,8 +438,8 @@ fn reanudar_paso(git: &Git, dir: &Path, id: &str, tipo: &str, slug: &str) -> Pas
             Resultado::Ok
         },
     )
-    .con("rama", rama.as_str())
-    .con("worktree", rel);
+    .con("rama", rama)
+    .con("worktree", d.rel.as_str());
     Ok(if creado.is_empty() {
         i
     } else {
diff --git a/crates/tc2/src/git/remoto.rs b/crates/tc2/src/git/remoto.rs
index be44271..5362fd9 100644
--- a/crates/tc2/src/git/remoto.rs
+++ b/crates/tc2/src/git/remoto.rs
@@ -37,7 +37,7 @@ fn sinred(op: &str) -> Informe {
 }
 
 /// «`op`: no se pudo leer origin; reintenta con red».
-pub(crate) fn sin_red_remoto(op: &str) -> Mensaje {
+fn sin_red_remoto(op: &str) -> Mensaje {
     Mensaje::nuevo(&txt::SIN_RED_REMOTO)
         .con("op", op)
         .con("remoto", REMOTO)
@@ -158,11 +158,24 @@ pub(crate) fn que_hacer_al_publicar(r: Resultado) -> &'static Texto {
     }
 }
 
-/// «push: `ultimo`; que hacer».
-fn push_fallo(ultimo: String, r: Resultado) -> Mensaje {
-    Mensaje::nuevo(&txt::PUSH_FALLO)
-        .con("ultimo", ultimo)
-        .con_mensaje("que_hacer", que_hacer_al_publicar(r))
+/// Informe de un [`publicar`] que no salio: su resultado, `clave`, el
+/// `motivo` de una `denegada` o un `error` local, los intentos y el
+/// diagnostico `texto` con el ultimo fallo y que hacer.
+pub(crate) fn informe_publicacion(
+    op: &str,
+    p: Publicacion,
+    clave: (&str, &str),
+    texto: &'static Texto,
+) -> Informe {
+    let mut i = Informe::nuevo(op, p.resultado).con(clave.0, clave.1);
+    if matches!(p.resultado, Resultado::Denegada | Resultado::Error) {
+        i = i.con("motivo", p.motivo);
+    }
+    i.con("intentos", p.intentos.to_string()).diagnostico(
+        Mensaje::nuevo(texto)
+            .con("ultimo", p.ultimo)
+            .con_mensaje("que_hacer", que_hacer_al_publicar(p.resultado)),
+    )
 }
 
 /// Trae el remoto. Nada nuevo → `sin-cambios`; sin red → `sinred`; remoto
@@ -171,8 +184,9 @@ pub fn fetch(git: &Git, dir: &Path) -> Informe {
     traer(git, dir, "fetch").unwrap_or_else(|i| i)
 }
 
-/// Informe de un `fetch` que fallo: sin red, fallo local o denegado.
-fn fallo_fetch(op: &str, s: &SalidaGit) -> Informe {
+/// Informe de una llamada al remoto (`fetch`, `ls-remote`) que fallo: sin
+/// red, fallo local o denegado.
+pub(crate) fn fallo_del_remoto(op: &str, s: &SalidaGit) -> Informe {
     match clasificar_fallo_remoto(&s.stderr) {
         FalloRemoto::SinRed | FalloRemoto::RamaInexistente => sinred(op),
         FalloRemoto::Local => error_local(op, &s.primera_linea_error()),
@@ -180,6 +194,17 @@ fn fallo_fetch(op: &str, s: &SalidaGit) -> Informe {
     }
 }
 
+/// `ls-remote <args>` y su stdout; un fallo se clasifica con
+/// [`fallo_del_remoto`].
+pub(crate) fn consultar_remoto(git: &Git, dir: &Path, op: &str, args: &[&str]) -> Paso<String> {
+    let s = correr(git, dir, op, Subcomando::LsRemote, args)?;
+    if s.exito() {
+        Ok(s.stdout)
+    } else {
+        Err(fallo_del_remoto(op, &s))
+    }
+}
+
 /// Trae solo `rama` de `origin` (actualiza `origin/<rama>`), con la misma
 /// clasificacion de fallos que [`fetch`], salvo que `origin` no tenga la
 /// rama: eso no es falta de red sino `rechazada motivo=sin-rama-base
@@ -205,14 +230,14 @@ pub(crate) fn traer_rama(git: &Git, dir: &Path, op: &str, rama: &str) -> Paso<()
                     .con("remoto", REMOTO),
             ));
     }
-    Err(fallo_fetch(op, &s))
+    Err(fallo_del_remoto(op, &s))
 }
 
 fn traer(git: &Git, dir: &Path, op: &str) -> Paso<Informe> {
     let antes = refs_remotas(git, dir, op)?;
     let s = correr(git, dir, op, Subcomando::Fetch, &["-q", REMOTO])?;
     if !s.exito() {
-        return Err(fallo_fetch(op, &s));
+        return Err(fallo_del_remoto(op, &s));
     }
     let despues = refs_remotas(git, dir, op)?;
     let r = if antes == despues {
@@ -366,23 +391,16 @@ fn push_paso(git: &Git, dir: &Path, op: &str, esperas: &[Duration]) -> Paso<Info
     let head = commit_de(git, dir, op, "HEAD")?.unwrap_or_default();
     let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
     let p = publicar(git, dir, op, &refspec, esperas)?;
-    let (r, intentos, ultimo) = (p.resultado, p.intentos, p.ultimo);
-    let i = Informe::nuevo(op, r).con("rama", rama.as_str());
-    Ok(match r {
+    let i = Informe::nuevo(op, p.resultado).con("rama", rama.as_str());
+    Ok(match p.resultado {
         Resultado::Ok => i
             .con("commit", corto(&head))
-            .con("intentos", intentos.to_string()),
+            .con("intentos", p.intentos.to_string()),
         Resultado::SinCambios => i.con("commit", corto(&head)),
         Resultado::Rechazada => i
             .con("motivo", "no-fast-forward")
             .diagnostico(&txt::LANZA_ACTUALIZAR_RAMA),
-        Resultado::Denegada | Resultado::Error => i
-            .con("motivo", p.motivo)
-            .con("intentos", intentos.to_string())
-            .diagnostico(push_fallo(ultimo, r)),
-        _ => i
-            .con("intentos", intentos.to_string())
-            .diagnostico(push_fallo(ultimo, r)),
+        _ => informe_publicacion(op, p, ("rama", &rama), &txt::PUSH_FALLO),
     })
 }
 
@@ -504,8 +522,7 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
     if hay_remota && !es_ancestro(git, dir, OP, "HEAD", &remota)? {
         let refspec = format!("refs/heads/{rama}:refs/heads/{rama}");
         let p = publicar(git, dir, OP, &refspec, esperas)?;
-        let (r, intentos, ultimo) = (p.resultado, p.intentos, p.ultimo);
-        match r {
+        match p.resultado {
             Resultado::Ok | Resultado::SinCambios => publicada = true,
             Resultado::Rechazada => {
                 return Ok(Informe::nuevo(OP, Resultado::Rechazada)
@@ -513,16 +530,12 @@ fn actualizar(git: &Git, dir: &Path, esperas: &[Duration]) -> Paso<Informe> {
                     .con("motivo", "no-fast-forward")
                     .diagnostico(&txt::PUBLICARON_ENTRE_MEDIAS));
             }
-            otro => {
-                let que_hacer = que_hacer_al_publicar(otro);
-                let mut i = Informe::nuevo(OP, otro).con("rama", rama.as_str());
-                if matches!(otro, Resultado::Denegada | Resultado::Error) {
-                    i = i.con("motivo", p.motivo);
-                }
-                return Ok(i.con("intentos", intentos.to_string()).diagnostico(
-                    Mensaje::nuevo(&txt::PUSH_FALLO_ACTUALIZADA_EN_LOCAL)
-                        .con("ultimo", ultimo)
-                        .con_mensaje("que_hacer", que_hacer),
+            _ => {
+                return Ok(informe_publicacion(
+                    OP,
+                    p,
+                    ("rama", &rama),
+                    &txt::PUSH_FALLO_ACTUALIZADA_EN_LOCAL,
                 ));
             }
         }
diff --git a/crates/tc2/src/git/tag.rs b/crates/tc2/src/git/tag.rs
index cb3c6ce..e4bb210 100644
--- a/crates/tc2/src/git/tag.rs
+++ b/crates/tc2/src/git/tag.rs
@@ -3,8 +3,8 @@
 use std::path::Path;
 use std::time::Duration;
 
-use super::remoto::publicar;
-use super::{Git, Paso, REMOTO, Subcomando, commit_de, correr, exigir};
+use super::remoto::{consultar_remoto, informe_publicacion, publicar};
+use super::{Git, Paso, REMOTO, Subcomando, commit_de, exigir};
 use crate::contrato::{Informe, Resultado, corto};
 use crate::i18n::Mensaje;
 use crate::i18n::catalogo::git as txt;
@@ -79,15 +79,11 @@ fn tag_paso(
         let refspec = format!("refs/tags/{tag}:refs/tags/{tag}");
         let p = publicar(git, dir, OP, &refspec, esperas)?;
         if !p.resultado.es_exito() {
-            let que_hacer = super::remoto::que_hacer_al_publicar(p.resultado);
-            let mut i = Informe::nuevo(OP, p.resultado).con("tag", tag);
-            if matches!(p.resultado, Resultado::Denegada | Resultado::Error) {
-                i = i.con("motivo", p.motivo);
-            }
-            return Ok(i.con("intentos", p.intentos.to_string()).diagnostico(
-                Mensaje::nuevo(&txt::TAG_SIN_PUBLICAR)
-                    .con("ultimo", p.ultimo)
-                    .con_mensaje("que_hacer", que_hacer),
+            return Ok(informe_publicacion(
+                OP,
+                p,
+                ("tag", tag),
+                &txt::TAG_SIN_PUBLICAR,
             ));
         }
     }
@@ -100,20 +96,9 @@ fn tag_paso(
 fn commit_remoto(git: &Git, dir: &Path, op: &str, tag: &str) -> Paso<Option<String>> {
     let r = format!("refs/tags/{tag}");
     let pelado = format!("{r}^{{}}");
-    let s = correr(
-        git,
-        dir,
-        op,
-        Subcomando::LsRemote,
-        &["--tags", REMOTO, &r, &pelado],
-    )?;
-    if !s.exito() {
-        return Err(Informe::nuevo(op, Resultado::Sinred)
-            .con("remoto", REMOTO)
-            .diagnostico(super::remoto::sin_red_remoto(op)));
-    }
+    let salida = consultar_remoto(git, dir, op, &["--tags", REMOTO, &r, &pelado])?;
     let mut directo = None;
-    for l in s.stdout.lines() {
+    for l in salida.lines() {
         let mut c = l.split('\t');
         let (Some(sha), Some(nombre)) = (c.next(), c.next()) else {
             continue;
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-142/tarea.md                                                     | 50 --------------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-142/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-142/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-142/tarea.md                                                      | 87 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 87 insertions(+), 50 deletions(-)
````

## Tests del diff (10 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/comun/ops.rs" ":(literal)crates/tc2/tests/convivencia_016.rs" ":(literal)crates/tc2/tests/estado_bordes.rs" ":(literal)crates/tc2/tests/estado_casos.rs" ":(literal)crates/tc2/tests/estado_concurrencia.rs" ":(literal)crates/tc2/tests/frescura.rs" ":(literal)crates/tc2/tests/git_casos_limite.rs" ":(literal)crates/tc2/tests/git_push.rs" ":(literal)crates/tc2/tests/roadmap.rs"
````

````
crates/tc2/tests/comun/mod.rs           | 22 ++++++++++++++++++++++
 crates/tc2/tests/comun/ops.rs           | 26 +++++++++++++++++++++++++-
 crates/tc2/tests/convivencia_016.rs     | 13 ++-----------
 crates/tc2/tests/estado_bordes.rs       | 65 ++++++++++++++---------------------------------------------------
 crates/tc2/tests/estado_casos.rs        | 60 ++++++++++++++----------------------------------------------
 crates/tc2/tests/estado_concurrencia.rs |  6 +-----
 crates/tc2/tests/frescura.rs            |  6 +-----
 crates/tc2/tests/git_casos_limite.rs    | 54 +++++++++++++++++++++++++++++++++++++++++++++++++++---
 crates/tc2/tests/git_push.rs            | 35 +++++++++++++++++++++++++++++++++++
 crates/tc2/tests/roadmap.rs             |  6 +-----
 10 files changed, 166 insertions(+), 127 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `git_falso` (crates/tc2/src/git/estado.rs:547) ~ `git_fallo` (crates/tc2/src/git/mod.rs:69)
- `lanzado` (crates/tc2/src/git/mod.rs:57) ~ `Lanzada` (crates/tc2/src/ciclo/operacion.rs:51)
- `exigida` (crates/tc2/src/git/mod.rs:66) ~ `exigidas` (crates/tc2/tests/plataforma_gitlab.rs:73)
- `codigo` (crates/tc2/src/git/puerto.rs:169) ~ `codigo` (crates/tc2/src/contrato.rs:59), `codigo` (crates/tc2/tests/doctor.rs:533)
- `ramas_de_tarea` (crates/tc2/src/git/rama.rs:95) ~ `rama_de_tarea` (crates/tc2/src/git/remoto.rs:390)
- `preparar` (crates/tc2/src/git/rama.rs:300) ~ `preparar` (crates/tc2/src/roadmap/mod.rs:467), `preparar` (crates/tc2/tests/comun/remoto.rs:80), `Preparar` (crates/tc2/tests/comun/remoto.rs:151)
- `rechazo` (crates/tc2/src/git/rama.rs:312) ~ `Rechazo` (crates/tc2/src/hallazgos/mod.rs:341), `rechazo` (crates/tc2/src/hallazgos/mod.rs:634), `rechazo` (crates/tc2/src/plan/aprobar.rs:334) y 7 mas
- `anadir` (crates/tc2/src/git/rama.rs:335) ~ `anadir` (crates/tc2/src/init/aplicar.rs:94)

### Posible duplicacion dentro de la tarea

Definiciones nuevas parecidas entre si: la misma tarea puede estar escribiendo dos veces lo mismo.

- `git_falso` (crates/tc2/src/git/estado.rs:547) ~ `git_fallo` (crates/tc2/src/git/mod.rs:89)

