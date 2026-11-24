# Peticion de revision — TASK-148e (ronda 1)

- Tarea: TASK-148e — soporte comun de los tests unitarios (A8-18, A8-20 unitarios) (parte e de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): b784ddac5325664edac24005c42ed909e40e3a2a
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148e (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-148e/tarea.md
- tareas/03-en-revision/TASK-148e/planificacion/
- crates/tc2/src/ciclo/liberar.rs
- crates/tc2/src/ciclo/mod.rs
- crates/tc2/src/ciclo/operacion.rs
- crates/tc2/src/estado/mod.rs
- crates/tc2/src/estado/montaje.rs
- crates/tc2/src/estado/prueba.rs
- crates/tc2/src/git/estado.rs
- crates/tc2/src/git/mod.rs
- crates/tc2/src/git/prueba.rs
- crates/tc2/src/plan/aprobar.rs
- crates/tc2/src/plan/guardar.rs
- crates/tc2/src/plan/operacion.rs
- crates/tc2/src/revision/operacion.rs
- crates/tc2/src/roadmap/mod.rs
- crates/tc2/src/siguiente.rs
- crates/tc2/src/toma/operacion.rs

## Commits a revisar (git log develop..HEAD)

````
b784dda docs(TASK-148e): resultado, criterios y autorrevision
771e187 fix(TASK-148e): estado::prueba con EnMemoria, estado, motivo y texto para los unitarios (A8-18)
e7abcab fix(TASK-148e): git::prueba con CONFIG, entorno, aislar y sh para los unitarios (A8-20)
dc9d70a chore(TASK-148e): tarea en curso
fa7646a merge(TASK-148e): develop en fix/task-148-auditoria-4d-soporte-de-tests-compartido para seguir con la parte
````

## Diff de tu dominio (16 fichero(s) de 16; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/ciclo/liberar.rs b/crates/tc2/src/ciclo/liberar.rs
index 7d97493..77d1c75 100644
--- a/crates/tc2/src/ciclo/liberar.rs
+++ b/crates/tc2/src/ciclo/liberar.rs
@@ -387,11 +387,10 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::{EnMemoria, estado, motivo, texto};
     use crate::grafo::DESCARTADA;
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
-    use std::collections::BTreeMap;
 
     /// 2026-11-17T10:20:30Z.
     const AHORA: u64 = 1_794_910_830;
@@ -436,29 +435,6 @@ mod tests {
         t.escribir(&Cuerpo::plantilla()) + &fila
     }
 
-    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
-        EnMemoria(
-            tareas
-                .iter()
-                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
-                .collect::<BTreeMap<_, _>>(),
-        )
-    }
-
-    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
-        match r {
-            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
-            _ => None,
-        }
-    }
-
-    fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
-        match r.as_deref() {
-            Ok([c]) => c.contenido.clone(),
-            otro => format!("{otro:?}"),
-        }
-    }
-
     fn fila(fase: &str, por: &str) -> String {
         format!(
             "| 2026-11-17T10:20:30Z | {fase} | manual | {por} | {} |\n",
diff --git a/crates/tc2/src/ciclo/mod.rs b/crates/tc2/src/ciclo/mod.rs
index a7182b3..ef0ff96 100644
--- a/crates/tc2/src/ciclo/mod.rs
+++ b/crates/tc2/src/ciclo/mod.rs
@@ -434,16 +434,10 @@ fn de_la_orden(mut i: Informe, nombre: &str, id: &str) -> Informe {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::git::prueba::{R, aislar, entorno, sh};
     use crate::identidad::{Cuenta, Quien};
     use crate::plataforma::Proveedor;
-    use std::ffi::OsString;
     use std::path::PathBuf;
-    use std::process::Command;
-
-    type R<T> = Result<T, Box<dyn std::error::Error>>;
-
-    /// Configuracion de git aislada de la del usuario.
-    const CONFIG: &str = "[user]\n\tname = Prueba\n\temail = prueba@example.com\n[core]\n\tautocrlf = false\n[init]\n\tdefaultBranch = develop\n[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n";
 
     const PERSONAS: &str =
         "---\npersonas:\n  - {id: 1001, login: \"ana\"}\n  - {id: 1002, login: \"beto\"}\n";
@@ -459,31 +453,6 @@ mod tests {
         }
     }
 
-    fn entorno(raiz: &Path) -> Vec<(OsString, OsString)> {
-        vec![
-            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
-            (
-                "GIT_CONFIG_GLOBAL".into(),
-                raiz.join("gitconfig").into_os_string(),
-            ),
-        ]
-    }
-
-    /// git de sistema para preparar el escenario; devuelve stdout recortado.
-    fn sh(raiz: &Path, dir: &Path, args: &[&str]) -> R<String> {
-        let s = Command::new("git")
-            .current_dir(dir)
-            .args(args)
-            .envs(entorno(raiz))
-            .env("LC_ALL", "C")
-            .env("GIT_TERMINAL_PROMPT", "0")
-            .output()?;
-        if !s.status.success() {
-            return Err(format!("git {args:?}: {}", String::from_utf8_lossy(&s.stderr)).into());
-        }
-        Ok(String::from_utf8_lossy(&s.stdout).trim().to_owned())
-    }
-
     fn persona(id: u64, login: &str) -> Quien {
         Quien::Persona(Cuenta {
             proveedor: Proveedor::GitHub,
@@ -508,7 +477,7 @@ mod tests {
         let _ = std::fs::remove_dir_all(&raiz);
         std::fs::create_dir_all(&raiz)?;
         let _borrar = Carpeta(raiz.clone());
-        std::fs::write(raiz.join("gitconfig"), CONFIG)?;
+        aislar(&raiz)?;
         sh(
             &raiz,
             &raiz,
@@ -573,7 +542,7 @@ mod tests {
         let _ = std::fs::remove_dir_all(&raiz);
         std::fs::create_dir_all(&raiz)?;
         let _borrar = Carpeta(raiz.clone());
-        std::fs::write(raiz.join("gitconfig"), CONFIG)?;
+        aislar(&raiz)?;
         sh(
             &raiz,
             &raiz,
diff --git a/crates/tc2/src/ciclo/operacion.rs b/crates/tc2/src/ciclo/operacion.rs
index c446c4d..cede29d 100644
--- a/crates/tc2/src/ciclo/operacion.rs
+++ b/crates/tc2/src/ciclo/operacion.rs
@@ -267,10 +267,9 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::{estado, motivo, texto};
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
-    use std::collections::BTreeMap;
 
     /// 2026-11-17T10:20:30Z.
     const AHORA: u64 = 1_794_910_830;
@@ -294,29 +293,6 @@ mod tests {
         t.escribir(&Cuerpo::plantilla())
     }
 
-    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
-        EnMemoria(
-            tareas
-                .iter()
-                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
-                .collect::<BTreeMap<_, _>>(),
-        )
-    }
-
-    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
-        match r {
-            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
-            _ => None,
-        }
-    }
-
-    fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
-        match r.as_deref() {
-            Ok([c]) => c.contenido.clone(),
-            otro => format!("{otro:?}"),
-        }
-    }
-
     fn fila(fase: &str) -> String {
         format!(
             "| 2026-11-17T10:20:30Z | {fase} | manual | agente | {} |\n",
diff --git a/crates/tc2/src/estado/mod.rs b/crates/tc2/src/estado/mod.rs
index 88c6b8a..d11752c 100644
--- a/crates/tc2/src/estado/mod.rs
+++ b/crates/tc2/src/estado/mod.rs
@@ -29,6 +29,8 @@ mod frescura;
 mod lectura;
 mod montaje;
 mod pendientes;
+#[cfg(test)]
+pub(crate) mod prueba;
 mod tareas;
 mod transaccion;
 
diff --git a/crates/tc2/src/estado/montaje.rs b/crates/tc2/src/estado/montaje.rs
index 21f915a..290209a 100644
--- a/crates/tc2/src/estado/montaje.rs
+++ b/crates/tc2/src/estado/montaje.rs
@@ -315,6 +315,7 @@ fn denegada(f: FalloRemoto, detalle: &str) -> Informe {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::git::prueba::{aislar, sh};
     use std::process::{Command, Stdio};
 
     type R<T> = Result<T, Box<dyn std::error::Error>>;
@@ -332,35 +333,12 @@ mod tests {
         d
     }
 
-    /// git de sistema para preparar el escenario; devuelve stdout recortado.
-    fn sh(dir: &Path, args: &[&str]) -> R<String> {
-        let s = Command::new("git")
-            .current_dir(dir)
-            .args([
-                "-c",
-                "user.name=Prueba",
-                "-c",
-                "user.email=prueba@example.com",
-                "-c",
-                "commit.gpgsign=false",
-                "-c",
-                "init.defaultBranch=main",
-            ])
-            .args(args)
-            .env("GIT_TERMINAL_PROMPT", "0")
-            .output()?;
-        if !s.status.success() {
-            return Err(format!("git {args:?}: {}", String::from_utf8_lossy(&s.stderr)).into());
-        }
-        Ok(String::from_utf8_lossy(&s.stdout).trim().to_owned())
-    }
-
     /// Commit en el bare con un fichero `f` de contenido `texto` y los padres
     /// dados (ninguno: huerfano).
-    fn commit_en_bare(bare: &Path, texto: &str, padres: &[&str]) -> R<String> {
+    fn commit_en_bare(raiz: &Path, bare: &Path, texto: &str, padres: &[&str]) -> R<String> {
         let tmp = bare.join("blob-temporal");
         std::fs::write(&tmp, texto)?;
-        let blob = sh(bare, &["hash-object", "-w", utf8(&tmp)?])?;
+        let blob = sh(raiz, bare, &["hash-object", "-w", utf8(&tmp)?])?;
         std::fs::remove_file(&tmp)?;
         let mut c = Command::new("git")
             .current_dir(bare)
@@ -383,7 +361,7 @@ mod tests {
             args.push("-p");
             args.push(p);
         }
-        sh(bare, &args)
+        sh(raiz, bare, &args)
     }
 
     /// Un bare con `main` y la rama de estado, y un clon que solo tiene
@@ -394,18 +372,21 @@ mod tests {
         let bare = raiz.join("remoto.git");
         let clon = raiz.join("clon");
         std::fs::create_dir_all(&bare)?;
-        sh(&bare, &["init", "-q", "--bare"])?;
-        let c_main = commit_en_bare(&bare, "main", &[])?;
-        sh(&bare, &["update-ref", "refs/heads/main", &c_main])?;
-        sh(&bare, &["symbolic-ref", "HEAD", "refs/heads/main"])?;
-        let c_estado = commit_en_bare(&bare, "estado", &[])?;
+        aislar(&raiz)?;
+        sh(&raiz, &bare, &["init", "-q", "--bare"])?;
+        let c_main = commit_en_bare(&raiz, &bare, "main", &[])?;
+        sh(&raiz, &bare, &["update-ref", "refs/heads/main", &c_main])?;
+        sh(&raiz, &bare, &["symbolic-ref", "HEAD", "refs/heads/main"])?;
+        let c_estado = commit_en_bare(&raiz, &bare, "estado", &[])?;
         sh(
+            &raiz,
             &bare,
             &["update-ref", &format!("refs/heads/{RAMA}"), &c_estado],
         )?;
-        sh(&raiz, &["clone", "-q", utf8(&bare)?, utf8(&clon)?])?;
+        sh(&raiz, &raiz, &["clone", "-q", utf8(&bare)?, utf8(&clon)?])?;
         assert_eq!(
             sh(
+                &raiz,
                 &clon,
                 &["rev-parse", &format!("refs/remotes/{REMOTO}/{RAMA}")]
             )?,
@@ -468,8 +449,9 @@ mod tests {
     #[test]
     fn montar_con_rama_reescrita_es_error() -> R<()> {
         let (raiz, bare, clon, _) = escenario("reescrita")?;
-        let huerfano = commit_en_bare(&bare, "reescrita", &[])?;
+        let huerfano = commit_en_bare(&raiz, &bare, "reescrita", &[])?;
         sh(
+            &raiz,
             &bare,
             &["update-ref", &format!("refs/heads/{RAMA}"), &huerfano],
         )?;
@@ -477,7 +459,7 @@ mod tests {
         montar_falla(&clon, "rama-reescrita")?;
         // El bare sigue con el huerfano: no se ha publicado nada.
         assert_eq!(
-            sh(&bare, &["rev-parse", &format!("refs/heads/{RAMA}")])?,
+            sh(&raiz, &bare, &["rev-parse", &format!("refs/heads/{RAMA}")])?,
             huerfano
         );
         let _ = std::fs::remove_dir_all(&raiz);
@@ -491,8 +473,9 @@ mod tests {
         let (raiz, bare, clon, c_estado) = escenario("bloqueada")?;
         // El remoto avanza (fast-forward) para que el `fetch` tenga que
         // escribir la ref remota.
-        let siguiente = commit_en_bare(&bare, "siguiente", &[&c_estado])?;
+        let siguiente = commit_en_bare(&raiz, &bare, "siguiente", &[&c_estado])?;
         sh(
+            &raiz,
             &bare,
             &["update-ref", &format!("refs/heads/{RAMA}"), &siguiente],
         )?;
@@ -508,7 +491,7 @@ mod tests {
         montar_falla(&clon, "error-local")?;
         assert!(lock.exists(), "el lock ajeno no se toca");
         assert_eq!(
-            sh(&bare, &["rev-parse", &format!("refs/heads/{RAMA}")])?,
+            sh(&raiz, &bare, &["rev-parse", &format!("refs/heads/{RAMA}")])?,
             siguiente
         );
         let _ = std::fs::remove_dir_all(&raiz);
diff --git a/crates/tc2/src/estado/prueba.rs b/crates/tc2/src/estado/prueba.rs
new file mode 100644
index 0000000..267ca84
--- /dev/null
+++ b/crates/tc2/src/estado/prueba.rs
@@ -0,0 +1,57 @@
+//! Soporte comun de los tests unitarios de las operaciones sobre la rama de
+//! estado (A8-18): el doble [`EnMemoria`] y los ayudantes para montarlo y
+//! leer lo que devuelve una operacion.
+
+use std::collections::BTreeMap;
+
+use super::{Cambio, Estado, Fallo};
+
+/// Estado en memoria: ruta -> contenido.
+pub(crate) struct EnMemoria(pub(crate) BTreeMap<String, String>);
+
+impl EnMemoria {
+    /// Estado con los ficheros dados por su ruta.
+    pub(crate) fn de_rutas(ficheros: &[(&str, String)]) -> Self {
+        EnMemoria(
+            ficheros
+                .iter()
+                .map(|(r, t)| ((*r).to_owned(), t.clone()))
+                .collect(),
+        )
+    }
+}
+
+impl Estado for EnMemoria {
+    fn rutas(&self) -> Vec<String> {
+        self.0.keys().cloned().collect()
+    }
+    fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
+        Ok(self.0.get(ruta).cloned())
+    }
+}
+
+/// Estado con cada tarea guardada en `{id}/tarea.md`.
+pub(crate) fn estado(tareas: &[(&str, String)]) -> EnMemoria {
+    EnMemoria(
+        tareas
+            .iter()
+            .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
+            .collect(),
+    )
+}
+
+/// Motivo del rechazo, si la operacion se rechazo.
+pub(crate) fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
+    match r {
+        Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
+        _ => None,
+    }
+}
+
+/// Contenido del unico cambio, o el resultado en `Debug` si no es uno.
+pub(crate) fn texto(r: &Result<Vec<Cambio>, Fallo>) -> String {
+    match r.as_deref() {
+        Ok([c]) => c.contenido.clone(),
+        otro => format!("{otro:?}"),
+    }
+}
diff --git a/crates/tc2/src/git/estado.rs b/crates/tc2/src/git/estado.rs
index 759d623..71c4375 100644
--- a/crates/tc2/src/git/estado.rs
+++ b/crates/tc2/src/git/estado.rs
@@ -548,6 +548,7 @@ mod tests {
 #[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
 mod tests_stdin {
     use super::*;
+    use crate::git::prueba::{aislar, entorno, sh};
     use crate::i18n::Idioma;
 
     /// Repo vacio de verdad en una carpeta temporal propia.
@@ -555,13 +556,8 @@ mod tests_stdin {
         let d = std::env::temp_dir().join(format!("tc2-estado-{nombre}-{}", std::process::id()));
         let _ = std::fs::remove_dir_all(&d);
         std::fs::create_dir_all(&d).unwrap();
-        let ok = std::process::Command::new("git")
-            .current_dir(&d)
-            .args(["init", "-q"])
-            .status()
-            .unwrap()
-            .success();
-        assert!(ok);
+        aislar(&d).unwrap();
+        sh(&d, &d, &["init", "-q"]).unwrap();
         d
     }
 
@@ -673,10 +669,7 @@ mod tests_stdin {
             ),
         )
         .unwrap();
-        let git = Git::nuevo().con_entorno(vec![
-            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
-            ("GIT_CONFIG_GLOBAL".into(), config.into_os_string()),
-        ]);
+        let git = Git::nuevo().con_entorno(entorno(&d));
         let base = construir_commit(
             &git,
             &d,
diff --git a/crates/tc2/src/git/mod.rs b/crates/tc2/src/git/mod.rs
index 01da255..e6ff3f8 100644
--- a/crates/tc2/src/git/mod.rs
+++ b/crates/tc2/src/git/mod.rs
@@ -12,6 +12,8 @@
 mod diff;
 pub(crate) mod estado;
 mod guardar;
+#[cfg(test)]
+pub(crate) mod prueba;
 mod puerto;
 pub(crate) mod rama;
 pub(crate) mod remoto;
diff --git a/crates/tc2/src/git/prueba.rs b/crates/tc2/src/git/prueba.rs
new file mode 100644
index 0000000..eb0d0a5
--- /dev/null
+++ b/crates/tc2/src/git/prueba.rs
@@ -0,0 +1,65 @@
+//! Soporte comun de los tests unitarios que lanzan git de sistema (A8-20):
+//! la configuracion aislada de la del usuario y el git con el que se
+//! preparan los escenarios. Los tests de integracion tienen el suyo en
+//! `tests/comun` (otro crate).
+
+use std::ffi::OsString;
+use std::path::Path;
+use std::process::Command;
+
+pub(crate) type R<T> = Result<T, Box<dyn std::error::Error>>;
+
+/// Configuracion de git aislada de la del usuario.
+pub(crate) const CONFIG: &str = "[user]\n\tname = Prueba\n\temail = prueba@example.com\n[core]\n\tautocrlf = false\n[init]\n\tdefaultBranch = develop\n[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n";
+
+/// Entorno que hace que git lea solo `raiz/gitconfig`.
+pub(crate) fn entorno(raiz: &Path) -> Vec<(OsString, OsString)> {
+    vec![
+        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
+        (
+            "GIT_CONFIG_GLOBAL".into(),
+            raiz.join("gitconfig").into_os_string(),
+        ),
+    ]
+}
+
+/// Escribe [`CONFIG`] en `raiz/gitconfig`, la que lee [`entorno`].
+pub(crate) fn aislar(raiz: &Path) -> R<()> {
+    std::fs::write(raiz.join("gitconfig"), CONFIG)?;
+    Ok(())
+}
+
+/// git de sistema para preparar el escenario; devuelve stdout recortado.
+pub(crate) fn sh(raiz: &Path, dir: &Path, args: &[&str]) -> R<String> {
+    let s = Command::new("git")
+        .current_dir(dir)
+        .args(args)
+        .envs(entorno(raiz))
+        .env("LC_ALL", "C")
+        .env("GIT_TERMINAL_PROMPT", "0")
+        .output()?;
+    if !s.status.success() {
+        return Err(format!("git {args:?}: {}", String::from_utf8_lossy(&s.stderr)).into());
+    }
+    Ok(String::from_utf8_lossy(&s.stdout).trim().to_owned())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// `sh` lee la configuracion que deja `aislar`, no la del usuario.
+    #[test]
+    fn sh_lee_la_configuracion_aislada() -> R<()> {
+        let raiz = std::env::temp_dir()
+            .join("tc2-tests")
+            .join(format!("git-prueba-{}", std::process::id()));
+        let _ = std::fs::remove_dir_all(&raiz);
+        std::fs::create_dir_all(&raiz)?;
+        aislar(&raiz)?;
+        let email = sh(&raiz, &raiz, &["config", "user.email"]);
+        let _ = std::fs::remove_dir_all(&raiz);
+        assert_eq!(email?, "prueba@example.com");
+        Ok(())
+    }
+}
diff --git a/crates/tc2/src/plan/aprobar.rs b/crates/tc2/src/plan/aprobar.rs
index df123d4..78319bb 100644
--- a/crates/tc2/src/plan/aprobar.rs
+++ b/crates/tc2/src/plan/aprobar.rs
@@ -562,9 +562,9 @@ impl Operacion for PedirCambios {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::{EnMemoria, motivo};
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
     use crate::tarea::Cuerpo;
     use std::collections::BTreeMap;
 
@@ -597,13 +597,6 @@ mod tests {
         t.escribir(&c)
     }
 
-    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
-        match r {
-            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
-            _ => None,
-        }
-    }
-
     /// Criterio 1: el agente no puede construir un `Aprobar` (RR-04).
     #[test]
     fn solo_una_persona_aprueba() {
diff --git a/crates/tc2/src/plan/guardar.rs b/crates/tc2/src/plan/guardar.rs
index 6edb865..5e8e142 100644
--- a/crates/tc2/src/plan/guardar.rs
+++ b/crates/tc2/src/plan/guardar.rs
@@ -151,9 +151,9 @@ impl Operacion for Guardar {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::EnMemoria;
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
     use crate::tarea::{Cuerpo, Tarea};
     use std::collections::BTreeMap;
 
diff --git a/crates/tc2/src/plan/operacion.rs b/crates/tc2/src/plan/operacion.rs
index 8f65a4d..e6b3adc 100644
--- a/crates/tc2/src/plan/operacion.rs
+++ b/crates/tc2/src/plan/operacion.rs
@@ -516,11 +516,10 @@ impl Operacion for Planificar {
 mod tests {
     use super::*;
     use crate::config::ModoFlujo;
+    use crate::estado::prueba::{EnMemoria, motivo};
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
     use crate::tarea::Tarea;
-    use std::collections::BTreeMap;
 
     /// 2026-11-17T10:20:30Z.
     const AHORA: u64 = 1_794_910_830;
@@ -541,34 +540,18 @@ mod tests {
         t.escribir(&Cuerpo::con_enunciado("Que haga algo", &c))
     }
 
-    fn estado(ficheros: &[(&str, String)]) -> EnMemoria {
-        EnMemoria(
-            ficheros
-                .iter()
-                .map(|(r, t)| ((*r).to_owned(), t.clone()))
-                .collect::<BTreeMap<_, _>>(),
-        )
-    }
-
     fn rutas(r: &Result<Vec<Cambio>, Fallo>) -> Vec<String> {
         r.as_ref()
             .map(|c| c.iter().map(|x| x.ruta.clone()).collect())
             .unwrap_or_default()
     }
 
-    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
-        match r {
-            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
-            _ => None,
-        }
-    }
-
     /// Criterio 2 (unitario): uno en fix (solo la peticion de redaccion),
     /// los cuatro y el unificador con `--carril riesgo-alto`; el carril y
     /// el modo de la configuracion quedan en la tarea.
     #[test]
     fn uno_en_fix_y_todos_en_riesgo_alto() {
-        let e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
+        let e = EnMemoria::de_rutas(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
         let mut c = Config::suelo();
         c.modo_flujo = ModoFlujo::Automatico;
         let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
@@ -629,7 +612,7 @@ mod tests {
     #[test]
     fn rondas_y_carril() {
         let c = Config::suelo();
-        let mut e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
+        let mut e = EnMemoria::de_rutas(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
         let aplicar = |e: &mut EnMemoria, carril: Option<Carril>, c: &Config| {
             let op = Planificar::nueva("TASK-001", &ana(), carril, c, AHORA);
             let r = op.aplicar(e);
@@ -681,7 +664,7 @@ mod tests {
     #[test]
     fn cambios_pedidos_cierran_la_ronda() {
         let c = Config::suelo();
-        let mut e = estado(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
+        let mut e = EnMemoria::de_rutas(&[("TASK-001/tarea.md", md("fix", "planificada", &["a"]))]);
         let aplicar = |e: &mut EnMemoria| {
             let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
             let r = op.aplicar(e);
@@ -720,7 +703,7 @@ mod tests {
         sin_objetivo.estado = "planificada".into();
         let otra = md("feature", "planificada", &["a"])
             .replace("asignado_a: null", "asignado_a: {id: 9, login: \"beto\"}");
-        let e = estado(&[
+        let e = EnMemoria::de_rutas(&[
             ("TASK-001/tarea.md", md("feature", "en-curso", &["a"])),
             (
                 "TASK-002/tarea.md",
@@ -788,7 +771,7 @@ mod tests {
         }
         let md = md("hotfix", "planificada", &["a"])
             .replace("complejidad: null", "complejidad: \"critica\"");
-        let e = estado(&[("TASK-001/tarea.md", md)]);
+        let e = EnMemoria::de_rutas(&[("TASK-001/tarea.md", md)]);
         let op = Planificar::nueva("TASK-001", &ana(), None, &c, AHORA);
         assert!(op.aplicar(&e).is_ok());
         assert_eq!(op.resumen().map(|s| s.roles.len()), Some(3));
diff --git a/crates/tc2/src/revision/operacion.rs b/crates/tc2/src/revision/operacion.rs
index b3d92a0..a664d9d 100644
--- a/crates/tc2/src/revision/operacion.rs
+++ b/crates/tc2/src/revision/operacion.rs
@@ -198,11 +198,10 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::{estado, motivo};
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
     use crate::tarea::{Cuerpo, Tarea};
-    use std::collections::BTreeMap;
 
     /// 2026-11-17T10:20:30Z.
     const AHORA: u64 = 1_794_910_830;
@@ -223,24 +222,13 @@ mod tests {
         t.escribir(&Cuerpo::plantilla())
     }
 
-    fn estado(t: String) -> EnMemoria {
-        EnMemoria(BTreeMap::from([("TASK-001/tarea.md".to_owned(), t)]))
-    }
-
-    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
-        match r {
-            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
-            _ => None,
-        }
-    }
-
     /// `en-curso` pasa a `en-revision` con su fila; ya en revision, nada;
     /// otro estado o de otra persona, rechazada.
     #[test]
     fn de_en_curso_a_en_revision() {
         let antes = md("en-curso", ana().asignado().into());
         let op = Revisar::nueva("TASK-001", &ana(), AHORA);
-        let r = op.aplicar(&estado(antes.clone()));
+        let r = op.aplicar(&estado(&[("TASK-001", antes.clone())]));
         let texto = r
             .as_ref()
             .ok()
@@ -256,10 +244,13 @@ mod tests {
             );
         assert_eq!(texto, esperado, "{r:?}");
         let ya = md("en-revision", ana().asignado().into());
-        assert_eq!(op.aplicar(&estado(ya)), Ok(Vec::new()));
-        let r = op.aplicar(&estado(md("planificada", None)));
+        assert_eq!(op.aplicar(&estado(&[("TASK-001", ya)])), Ok(Vec::new()));
+        let r = op.aplicar(&estado(&[("TASK-001", md("planificada", None))]));
         assert_eq!(motivo(&r), Some("estado-no-valido"));
-        let r = op.aplicar(&estado(md("en-curso", Asignado::persona(9, "beto"))));
+        let r = op.aplicar(&estado(&[(
+            "TASK-001",
+            md("en-curso", Asignado::persona(9, "beto")),
+        )]));
         assert_eq!(motivo(&r), Some("de-otra-persona"));
         assert_eq!(op.dueno().as_deref(), Some("beto"));
         assert!(op.exige_red());
@@ -279,7 +270,7 @@ mod tests {
         };
         let antes = md("en-curso", ana().asignado().into());
         let op = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:rust"]));
-        let texto = match op.aplicar(&estado(antes)) {
+        let texto = match op.aplicar(&estado(&[("TASK-001", antes)])) {
             Ok(c) => c.first().map(|c| c.contenido.clone()).unwrap_or_default(),
             Err(_) => String::new(),
         };
@@ -288,9 +279,12 @@ mod tests {
             "{texto}"
         );
         let ya = texto.replace("estado: \"en-curso\"", "estado: \"en-revision\"");
-        assert_eq!(op.aplicar(&estado(ya.clone())), Ok(Vec::new()));
+        assert_eq!(
+            op.aplicar(&estado(&[("TASK-001", ya.clone())])),
+            Ok(Vec::new())
+        );
         let otra = Revisar::nueva("TASK-001", &ana(), AHORA).con_seleccion(sel(&["p:vue"]));
-        let r = otra.aplicar(&estado(ya.clone()));
+        let r = otra.aplicar(&estado(&[("TASK-001", ya.clone())]));
         let nuevo = r
             .ok()
             .and_then(|c| c.first().map(|c| c.contenido.clone()))
diff --git a/crates/tc2/src/roadmap/mod.rs b/crates/tc2/src/roadmap/mod.rs
index aa6cb31..cd09e72 100644
--- a/crates/tc2/src/roadmap/mod.rs
+++ b/crates/tc2/src/roadmap/mod.rs
@@ -571,22 +571,11 @@ fn adelantar_editada(g: &Git, carpeta: &Path, commit: &str) -> bool {
 #[cfg(test)]
 pub(crate) mod tests {
     use super::*;
+    use crate::estado::prueba::EnMemoria;
     use std::collections::BTreeMap;
 
     use crate::tarea::{Asignado, Cuerpo, Tarea};
 
-    /// Estado en memoria.
-    pub(crate) struct EnMemoria(pub(crate) BTreeMap<String, String>);
-
-    impl Estado for EnMemoria {
-        fn rutas(&self) -> Vec<String> {
-            self.0.keys().cloned().collect()
-        }
-        fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
-            Ok(self.0.get(ruta).cloned())
-        }
-    }
-
     fn v2(
         id: &str,
         titulo: &str,
diff --git a/crates/tc2/src/siguiente.rs b/crates/tc2/src/siguiente.rs
index 7922c14..99934e4 100644
--- a/crates/tc2/src/siguiente.rs
+++ b/crates/tc2/src/siguiente.rs
@@ -134,20 +134,9 @@ fn calcular(estado: &dyn Estado) -> Result<Calculo, Fallo> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::EnMemoria;
     use std::collections::BTreeMap;
 
-    /// Estado en memoria.
-    struct EnMemoria(BTreeMap<String, String>);
-
-    impl Estado for EnMemoria {
-        fn rutas(&self) -> Vec<String> {
-            self.0.keys().cloned().collect()
-        }
-        fn leer(&self, ruta: &str) -> Result<Option<String>, Fallo> {
-            Ok(self.0.get(ruta).cloned())
-        }
-    }
-
     fn tarea(id: &str, sprint: u32, estado: &str, deps: &[&str]) -> String {
         let mut t = crate::tarea::Tarea::nueva(id, "t", "2026-01-01");
         t.sprint = sprint;
diff --git a/crates/tc2/src/toma/operacion.rs b/crates/tc2/src/toma/operacion.rs
index 23205f1..f241d61 100644
--- a/crates/tc2/src/toma/operacion.rs
+++ b/crates/tc2/src/toma/operacion.rs
@@ -284,11 +284,10 @@ pub fn desde_anotacion(a: &str) -> Option<Box<dyn Operacion>> {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::estado::prueba::{estado, motivo};
     use crate::identidad::Cuenta;
     use crate::plataforma::Proveedor;
-    use crate::roadmap::tests::EnMemoria;
     use crate::tarea::{Cuerpo, Documento, Tarea};
-    use std::collections::BTreeMap;
 
     /// 2026-11-17T10:20:30Z.
     const AHORA: u64 = 1_794_910_830;
@@ -317,22 +316,6 @@ mod tests {
         t.escribir(&Cuerpo::plantilla())
     }
 
-    fn estado(tareas: &[(&str, String)]) -> EnMemoria {
-        EnMemoria(
-            tareas
-                .iter()
-                .map(|(id, t)| (format!("{id}/tarea.md"), t.clone()))
-                .collect::<BTreeMap<_, _>>(),
-        )
-    }
-
-    fn motivo(r: &Result<Vec<Cambio>, Fallo>) -> Option<&str> {
-        match r {
-            Err(Fallo::Rechazada { motivo, .. }) => Some(motivo),
-            _ => None,
-        }
-    }
-
     #[test]
     fn instante_en_utc() {
         assert_eq!(instante(0), "1970-01-01T00:00:00Z");
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-148e/tarea.md                                                     | 47 -----------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-148e/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148e/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-148e/tarea.md                                                      | 69 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 69 insertions(+), 47 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `EnMemoria` (crates/tc2/src/estado/prueba.rs:10) ~ `EnMemoria` (crates/tc2/src/roadmap/mod.rs:579), `EnMemoria` (crates/tc2/src/siguiente.rs:140), `EnMemoria` (crates/tc2/tests/roadmap.rs:556)
- `rutas` (crates/tc2/src/estado/prueba.rs:25) ~ `rutas` (crates/tc2/src/alta.rs:704), `rutas` (crates/tc2/src/estado/lectura.rs:19), `rutas` (crates/tc2/src/estado/lectura.rs:105) y 6 mas
- `leer` (crates/tc2/src/estado/prueba.rs:28) ~ `leer` (crates/tc2/src/config/mod.rs:437), `leer` (crates/tc2/src/desinstalar/mod.rs:373), `leer` (crates/tc2/src/estado/frescura.rs:160) y 26 mas
- `estado` (crates/tc2/src/estado/prueba.rs:34) ~ `estado` (crates/tc2/src/ciclo/liberar.rs:439), `estado` (crates/tc2/src/ciclo/operacion.rs:297), `Estado` (crates/tc2/src/confianza/mod.rs:50) y 12 mas
- `motivo` (crates/tc2/src/estado/prueba.rs:44) ~ `motivo` (crates/tc2/src/ciclo/liberar.rs:448), `motivo` (crates/tc2/src/ciclo/operacion.rs:306), `Motivo` (crates/tc2/src/config/mod.rs:321) y 16 mas
- `texto` (crates/tc2/src/estado/prueba.rs:52) ~ `texto` (crates/tc2/src/actualizar_cli.rs:450), `texto` (crates/tc2/src/ciclo/liberar.rs:455), `texto` (crates/tc2/src/ciclo/operacion.rs:313) y 36 mas
- `entorno` (crates/tc2/src/git/prueba.rs:16) ~ `entorno` (crates/tc2/src/ciclo/mod.rs:462), `Entorno` (crates/tc2/src/doctor/mod.rs:364), `Entorno` (crates/tc2/src/estimacion/mod.rs:172) y 14 mas

