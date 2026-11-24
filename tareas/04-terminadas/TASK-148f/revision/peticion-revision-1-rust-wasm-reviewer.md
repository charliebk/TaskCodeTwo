# Peticion de revision — TASK-148f (ronda 1)

- Tarea: TASK-148f — dobles de Plataforma con Parcial y Doble (A8-05, absorbe A2-06) (parte f de TASK-148)
- Rama revisada: fix/task-148-auditoria-4d-soporte-de-tests-compartido
- Rama base: develop
- Commit revisado (HEAD): fd776e6274ef35d6a465d6b19761918159920678
- Fecha: 2026-11-24
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-148f (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-148f/tarea.md
- tareas/03-en-revision/TASK-148f/planificacion/
- crates/tc2/src/identidad.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/plataforma/prueba.rs
- crates/tc2/src/revision/mod.rs
- crates/tc2/src/tablero.rs
- crates/tc2/tests/comun/mod.rs
- crates/tc2/tests/comun/plataforma.rs
- crates/tc2/tests/revision.rs
- crates/tc2/tests/soporte.rs
- crates/tc2/tests/tablero.rs

## Commits a revisar (git log develop..HEAD)

````
fd776e6 docs(TASK-148f): resultado, criterios y autorrevision
e7cb0e8 fix(TASK-148f): comun::plataforma con Parcial y Doble y migracion de los dobles de tests (A8-05, A2-06)
50656db fix(TASK-148f): plataforma::prueba con Parcial y Doble y migracion de los dobles de src (A8-05)
be63f07 chore(TASK-148f): tarea en curso
afec6e1 merge(TASK-148f): develop en fix/task-148-auditoria-4d-soporte-de-tests-compartido para seguir con la parte
````

## Diff de tu dominio (10 fichero(s) de 10; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index ce77c31..3854eda 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -624,7 +624,7 @@ pub fn resolver(e: Entorno<'_>) -> Paso<Quien> {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::plataforma::{Firma, ModeloPuertas, ReglasLeidas};
+    use crate::plataforma::prueba::{Doble, Parcial};
     use std::cell::Cell;
 
     /// Plataforma de prueba: solo `identidad`, contando las llamadas.
@@ -634,44 +634,19 @@ mod tests {
     }
 
     impl Falsa {
-        fn de(login: &str, id: u64, tipo: &str) -> Self {
-            Self {
+        fn de(login: &str, id: u64, tipo: &str) -> Doble<Self> {
+            Doble(Self {
                 usuario: Usuario {
                     login: login.to_owned(),
                     id,
                     tipo: tipo.to_owned(),
                 },
                 llamadas: Cell::new(0),
-            }
+            })
         }
     }
 
-    fn sin_uso(op: &str) -> Informe {
-        Informe::nuevo(op, Resultado::Error)
-    }
-
-    impl Plataforma for Falsa {
-        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<crate::plataforma::PrAbierto>> {
-            Err(sin_uso("prs-abiertos"))
-        }
-        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
-            sin_uso("abrir-pr")
-        }
-        fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
-            sin_uso("estado-puertas")
-        }
-        fn merge_pr(&self, _: u64, _: &str) -> Informe {
-            sin_uso("merge-pr")
-        }
-        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
-            Err(sin_uso("leer-puertas"))
-        }
-        fn leer_permisos_ci(&self) -> Informe {
-            sin_uso("leer-permisos-ci")
-        }
-        fn firma_commit(&self, _: &str) -> Paso<Firma> {
-            Err(sin_uso("firma-commit"))
-        }
+    impl Parcial for Falsa {
         fn identidad(&self, _: Rol) -> Paso<Usuario> {
             self.llamadas.set(self.llamadas.get() + 1);
             Ok(self.usuario.clone())
@@ -766,11 +741,11 @@ mod tests {
             login: "ana".to_owned(),
         };
         assert_eq!(cache.guardar(&cuenta, AHORA), Ok(()));
-        let antes = ana.llamadas.get();
+        let antes = ana.0.llamadas.get();
         let q = quien_ejecuta(&ana, gh, &personas, &cache, false, AHORA + 60);
         assert_eq!(q.as_ref().map(Quien::decidido_por), Ok("agente"));
         assert_eq!(q.map(|q| q.cuenta().clone()), Ok(cuenta));
-        assert_eq!(ana.llamadas.get(), antes, "sin terminal no hay llamadas");
+        assert_eq!(ana.0.llamadas.get(), antes, "sin terminal no hay llamadas");
         let _ = std::fs::remove_dir_all(carpeta("bot"));
     }
 
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index af0f062..9e9d0d6 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -21,6 +21,8 @@ pub mod github;
 pub mod gitlab;
 pub mod http;
 pub mod personas;
+#[cfg(test)]
+pub(crate) mod prueba;
 pub mod puertas;
 
 pub use app_github::CredencialesApp;
@@ -425,7 +427,7 @@ pub trait Plataforma {
     /// entorno para que `git` empuje a `url_remoto` con el token del agente
     /// ([`entorno_git::entorno_push`]: cabecera limitada a esa URL, sin
     /// autor). El token no sale del adaptador mas que en esa cabecera. Por
-    /// defecto (adaptadores de prueba), `rechazada motivo=no-soportada`.
+    /// defecto, `rechazada motivo=no-soportada`.
     fn entorno_push(
         &self,
         url_remoto: &str,
@@ -443,8 +445,7 @@ pub trait Plataforma {
     /// check `revision-ia` que publico `origen` en la cabeza del PR `pr`, ya
     /// terminado, con los aceptados de sus anotaciones. Si aun no esta,
     /// `esperando motivo=sin-check`. Por defecto (GitLab, donde la revision
-    /// en CI no existe, y los adaptadores de prueba), `rechazada
-    /// motivo=no-soportada`.
+    /// en CI no existe), `rechazada motivo=no-soportada`.
     fn leer_revision(&self, rol: Rol, pr: u64, origen: u64) -> Paso<RevisionPublicada> {
         let _ = (rol, pr, origen);
         Err(
diff --git a/crates/tc2/src/plataforma/prueba.rs b/crates/tc2/src/plataforma/prueba.rs
new file mode 100644
index 0000000..a33a81c
--- /dev/null
+++ b/crates/tc2/src/plataforma/prueba.rs
@@ -0,0 +1,108 @@
+//! Soporte comun de los dobles de [`Plataforma`] de los tests unitarios
+//! (A8-05): cada doble implementa [`Parcial`] solo con lo que usa y se pasa
+//! envuelto en [`Doble`]. Una operacion que el test no espera hace `panic!`
+//! con su nombre del contrato, en vez de devolver un error en silencio. Los
+//! tests de integracion tienen el suyo en `tests/comun/plataforma.rs` (otro
+//! crate).
+
+#![allow(clippy::panic)]
+
+use std::ffi::OsString;
+
+use super::{
+    Firma, ModeloPuertas, Paso, Plataforma, PrAbierto, ReglasLeidas, RevisionPublicada, Rol,
+    Usuario,
+};
+use crate::contrato::Informe;
+
+/// Las operaciones de [`Plataforma`] que un doble puede dar. Por defecto,
+/// todas hacen `panic!` con el nombre de la operacion.
+pub(crate) trait Parcial {
+    fn prs_abiertos(&self, _rol: Rol) -> Paso<Vec<PrAbierto>> {
+        inesperada("prs-abiertos")
+    }
+    fn abrir_pr(&self, _rama: &str, _base: &str, _titulo: &str, _cuerpo: &str) -> Informe {
+        inesperada("abrir-pr")
+    }
+    fn entorno_push(&self, _url_remoto: &str) -> Paso<Vec<(OsString, OsString)>> {
+        inesperada("entorno-push")
+    }
+    fn leer_revision(&self, _rol: Rol, _pr: u64, _origen: u64) -> Paso<RevisionPublicada> {
+        inesperada("leer-revision")
+    }
+    fn estado_puertas(&self, _rol: Rol, _pr: u64, _modelo: &ModeloPuertas) -> Informe {
+        inesperada("estado-puertas")
+    }
+    fn merge_pr(&self, _pr: u64, _sha: &str) -> Informe {
+        inesperada("merge-pr")
+    }
+    fn leer_puertas(&self, _rol: Rol, _rama: &str) -> Paso<ReglasLeidas> {
+        inesperada("leer-puertas")
+    }
+    fn leer_permisos_ci(&self) -> Informe {
+        inesperada("leer-permisos-ci")
+    }
+    fn firma_commit(&self, _sha: &str) -> Paso<Firma> {
+        inesperada("firma-commit")
+    }
+    fn identidad(&self, _rol: Rol) -> Paso<Usuario> {
+        inesperada("identidad")
+    }
+}
+
+fn inesperada(op: &str) -> ! {
+    panic!("doble de Plataforma: {op} no se esperaba")
+}
+
+/// Envoltorio que da [`Plataforma`] a un [`Parcial`]. Las operaciones que
+/// [`Parcial`] no recoge siguen con el defecto del trait.
+pub(crate) struct Doble<P>(pub P);
+
+impl<P: Parcial> Plataforma for Doble<P> {
+    fn prs_abiertos(&self, rol: Rol) -> Paso<Vec<PrAbierto>> {
+        self.0.prs_abiertos(rol)
+    }
+    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
+        self.0.abrir_pr(rama, base, titulo, cuerpo)
+    }
+    fn entorno_push(&self, url_remoto: &str) -> Paso<Vec<(OsString, OsString)>> {
+        self.0.entorno_push(url_remoto)
+    }
+    fn leer_revision(&self, rol: Rol, pr: u64, origen: u64) -> Paso<RevisionPublicada> {
+        self.0.leer_revision(rol, pr, origen)
+    }
+    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
+        self.0.estado_puertas(rol, pr, modelo)
+    }
+    fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
+        self.0.merge_pr(pr, sha)
+    }
+    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
+        self.0.leer_puertas(rol, rama)
+    }
+    fn leer_permisos_ci(&self) -> Informe {
+        self.0.leer_permisos_ci()
+    }
+    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
+        self.0.firma_commit(sha)
+    }
+    fn identidad(&self, rol: Rol) -> Paso<Usuario> {
+        self.0.identidad(rol)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    struct Vacio;
+    impl Parcial for Vacio {}
+
+    /// A8-05: una operacion que el doble no espera hace `panic!` con su
+    /// nombre del contrato.
+    #[test]
+    #[should_panic(expected = "merge-pr")]
+    fn un_doble_vacio_no_admite_merge_pr() {
+        let _ = Doble(Vacio).merge_pr(1, "a");
+    }
+}
diff --git a/crates/tc2/src/revision/mod.rs b/crates/tc2/src/revision/mod.rs
index 9951003..5bfd383 100644
--- a/crates/tc2/src/revision/mod.rs
+++ b/crates/tc2/src/revision/mod.rs
@@ -524,7 +524,7 @@ pub fn seleccion_de(
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::plataforma::{Firma, PrAbierto, ReglasLeidas, Usuario};
+    use crate::plataforma::prueba::{Doble, Parcial};
     use std::cell::RefCell;
 
     /// Plataforma de prueba: solo `estado-puertas`, con el informe fijo y
@@ -534,44 +534,19 @@ mod tests {
         modelo: RefCell<Option<ModeloPuertas>>,
     }
 
-    fn sin_uso(op: &str) -> Informe {
-        Informe::nuevo(op, Resultado::Error)
-    }
-
-    impl Plataforma for Puertas {
-        fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
-            Err(sin_uso("prs-abiertos"))
-        }
-        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
-            sin_uso("abrir-pr")
-        }
+    impl Parcial for Puertas {
         fn estado_puertas(&self, rol: Rol, _: u64, modelo: &ModeloPuertas) -> Informe {
             assert_eq!(rol, Rol::Agente);
             *self.modelo.borrow_mut() = Some(modelo.clone());
             self.informe.clone()
         }
-        fn merge_pr(&self, _: u64, _: &str) -> Informe {
-            sin_uso("merge-pr")
-        }
-        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
-            Err(sin_uso("leer-puertas"))
-        }
-        fn leer_permisos_ci(&self) -> Informe {
-            sin_uso("leer-permisos-ci")
-        }
-        fn firma_commit(&self, _: &str) -> Paso<Firma> {
-            Err(sin_uso("firma-commit"))
-        }
-        fn identidad(&self, _: Rol) -> Paso<Usuario> {
-            Err(sin_uso("identidad"))
-        }
     }
 
-    fn con(informe: Informe) -> Puertas {
-        Puertas {
+    fn con(informe: Informe) -> Doble<Puertas> {
+        Doble(Puertas {
             informe,
             modelo: RefCell::new(None),
-        }
+        })
     }
 
     /// Criterio «check `revision-ia`» (unitario): sin `revisor_id` no se
@@ -582,10 +557,10 @@ mod tests {
         let p = con(Informe::nuevo("estado-puertas", Resultado::Ok));
         assert_eq!(revision_ia(&p, 7, None), "sin-revisor");
         assert_eq!(revision_ia(&p, 7, Some(0)), "sin-revisor");
-        assert!(p.modelo.borrow().is_none());
+        assert!(p.0.modelo.borrow().is_none());
         assert_eq!(revision_ia(&p, 7, Some(99)), "ok");
         assert_eq!(
-            p.modelo.borrow().as_ref().map(|m| m.checks.clone()),
+            p.0.modelo.borrow().as_ref().map(|m| m.checks.clone()),
             Some(vec![("revision-ia".to_owned(), 99)])
         );
         let esperando = |falta: &[&str]| {
diff --git a/crates/tc2/src/tablero.rs b/crates/tc2/src/tablero.rs
index ce11251..f8ac354 100644
--- a/crates/tc2/src/tablero.rs
+++ b/crates/tc2/src/tablero.rs
@@ -826,7 +826,8 @@ pub fn carpeta_actual(op: &str) -> Result<std::path::PathBuf, Informe> {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::plataforma::{Firma, ModeloPuertas, Paso, Plataforma, ReglasLeidas, Rol, Usuario};
+    use crate::plataforma::prueba::{Doble, Parcial};
+    use crate::plataforma::{Paso, Plataforma, Rol};
     use crate::roadmap::tests::{estado_de_referencia, personas};
 
     const COMMIT: &str = "0123456789ab";
@@ -997,31 +998,10 @@ mod tests {
     /// Plataforma falsa: solo `prs-abiertos`, con la respuesta dada.
     struct Falsa(Paso<Vec<PrAbierto>>);
 
-    impl Plataforma for Falsa {
+    impl Parcial for Falsa {
         fn prs_abiertos(&self, _: Rol) -> Paso<Vec<PrAbierto>> {
             self.0.clone()
         }
-        fn abrir_pr(&self, _: &str, _: &str, _: &str, _: &str) -> Informe {
-            Informe::nuevo("abrir-pr", Resultado::Error)
-        }
-        fn estado_puertas(&self, _: Rol, _: u64, _: &ModeloPuertas) -> Informe {
-            Informe::nuevo("estado-puertas", Resultado::Error)
-        }
-        fn merge_pr(&self, _: u64, _: &str) -> Informe {
-            Informe::nuevo("merge-pr", Resultado::Error)
-        }
-        fn leer_puertas(&self, _: Rol, _: &str) -> Paso<ReglasLeidas> {
-            Err(Informe::nuevo("leer-puertas", Resultado::Error))
-        }
-        fn leer_permisos_ci(&self) -> Informe {
-            Informe::nuevo("leer-permisos-ci", Resultado::Error)
-        }
-        fn firma_commit(&self, _: &str) -> Paso<Firma> {
-            Err(Informe::nuevo("firma-commit", Resultado::Error))
-        }
-        fn identidad(&self, _: Rol) -> Paso<Usuario> {
-            Err(Informe::nuevo("identidad", Resultado::Error))
-        }
     }
 
     fn pr(numero: u64, rama: &str, borrador: bool) -> PrAbierto {
@@ -1156,14 +1136,14 @@ mod tests {
     /// `pr-sin-revision=` (PR listo de una tarea que no esta en revision).
     #[test]
     fn board_cruza_los_pr_y_avisa() {
-        let falsa = Falsa(Ok(vec![
+        let falsa = Doble(Falsa(Ok(vec![
             pr(3, "feature/c", false),
             pr(5, "feature/b", true),
             pr(7, "feature/a", false),
             pr(9, "feature/e", true),
             pr(11, "feature/c", false),
             pr(12, "feature/otra", false),
-        ]));
+        ])));
         let consulta = match falsa.prs_abiertos(Rol::Agente) {
             Ok(v) => Consulta::Hecha(v),
             Err(i) => Consulta::de_fallo(&i),
@@ -1226,7 +1206,7 @@ mod tests {
                 "error",
             ),
         ] {
-            let falsa = Falsa(Err(fallo));
+            let falsa = Doble(Falsa(Err(fallo)));
             let consulta = match falsa.prs_abiertos(Rol::Agente) {
                 Ok(v) => Consulta::Hecha(v),
                 Err(i) => Consulta::de_fallo(&i),
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/01-en-diseno/TASK-148f/tarea.md                                                     | 47 -----------------------------------------------
 tareas/{01-en-diseno => 02-en-curso}/TASK-148f/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-148f/planificacion/plan-final.md                 |  0
 tareas/02-en-curso/TASK-148f/tarea.md                                                      | 70 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 70 insertions(+), 47 deletions(-)
````

## Tests del diff (5 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/mod.rs" ":(literal)crates/tc2/tests/comun/plataforma.rs" ":(literal)crates/tc2/tests/revision.rs" ":(literal)crates/tc2/tests/soporte.rs" ":(literal)crates/tc2/tests/tablero.rs"
````

````
crates/tc2/tests/comun/mod.rs        |  1 +
 crates/tc2/tests/comun/plataforma.rs | 89 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/revision.rs         | 73 ++++++++++++++++++++++++++++---------------------------------------------
 crates/tc2/tests/soporte.rs          | 14 ++++++++++++++
 crates/tc2/tests/tablero.rs          | 32 +++++---------------------------
 5 files changed, 137 insertions(+), 72 deletions(-)
````
## Candidatos 4D

Pistas generadas desde el diff, no hallazgos: comprueba cada una (Regla 4D de `revision.md`).

### Posible duplicacion con lo que ya existia

Definiciones nuevas con nombre igual o parecido a otra de la rama base. Mira si hacen lo mismo con pequenas diferencias (duplicacion real = IMPORTANTE).

- `prs_abiertos` (crates/tc2/src/plataforma/prueba.rs:21) ~ `prs_abiertos` (crates/tc2/src/identidad.rs:654), `prs_abiertos` (crates/tc2/src/plataforma/github.rs:402), `prs_abiertos` (crates/tc2/src/plataforma/gitlab.rs:528) y 5 mas
- `abrir_pr` (crates/tc2/src/plataforma/prueba.rs:24) ~ `abrir_pr` (crates/tc2/src/identidad.rs:657), `abrir_pr` (crates/tc2/src/plataforma/github.rs:433), `abrir_pr` (crates/tc2/src/plataforma/gitlab.rs:560) y 5 mas
- `entorno_push` (crates/tc2/src/plataforma/prueba.rs:27) ~ `entorno_ssh` (crates/tc2/tests/comun/remoto.rs:34), `entorno_push` (crates/tc2/src/plataforma/entorno_git.rs:109), `entorno_push` (crates/tc2/src/plataforma/github.rs:832) y 3 mas
- `leer_revision` (crates/tc2/src/plataforma/prueba.rs:30) ~ `leer_decision` (crates/tc2/src/hallazgos/orden.rs:694), `leer_revision` (crates/tc2/src/plataforma/github.rs:607), `leer_revision` (crates/tc2/src/plataforma/mod.rs:448) y 1 mas
- `estado_puertas` (crates/tc2/src/plataforma/prueba.rs:33) ~ `EstadoPuerta` (crates/tc2/src/plataforma/mod.rs:313), `estado_puertas` (crates/tc2/src/identidad.rs:660), `estado_puertas` (crates/tc2/src/plataforma/github.rs:483) y 6 mas
- `merge_pr` (crates/tc2/src/plataforma/prueba.rs:36) ~ `merge_pr` (crates/tc2/src/identidad.rs:663), `merge_pr` (crates/tc2/src/plataforma/github.rs:673), `merge_pr` (crates/tc2/src/plataforma/gitlab.rs:776) y 5 mas
- `leer_puertas` (crates/tc2/src/plataforma/prueba.rs:39) ~ `leer_puertas` (crates/tc2/src/identidad.rs:666), `leer_puertas` (crates/tc2/src/plataforma/github.rs:734), `leer_puertas` (crates/tc2/src/plataforma/gitlab.rs:858) y 8 mas
- `leer_permisos_ci` (crates/tc2/src/plataforma/prueba.rs:42) ~ `leer_permisos_ci` (crates/tc2/src/identidad.rs:669), `leer_permisos_ci` (crates/tc2/src/plataforma/github.rs:771), `leer_permisos_ci` (crates/tc2/src/plataforma/gitlab.rs:892) y 5 mas
- `firma_commit` (crates/tc2/src/plataforma/prueba.rs:45) ~ `firma_commit` (crates/tc2/src/identidad.rs:672), `firma_commit` (crates/tc2/src/plataforma/github.rs:797), `firma_commit` (crates/tc2/src/plataforma/gitlab.rs:903) y 5 mas
- `identidad` (crates/tc2/src/plataforma/prueba.rs:48) ~ `identidad` (crates/tc2/src/identidad.rs:675), `identidad` (crates/tc2/src/plataforma/conexion.rs:226), `identidad` (crates/tc2/src/plataforma/github.rs:815) y 6 mas
- `Doble` (crates/tc2/src/plataforma/prueba.rs:59) ~ `Doble` (crates/tc2/tests/revision.rs:51)
- `Vacio` (crates/tc2/src/plataforma/prueba.rs:98) ~ `vacia` (crates/tc2/src/estado/lectura.rs:68), `vacios` (crates/tc2/src/estimacion/historico.rs:137)
- `con` (crates/tc2/src/revision/mod.rs:545) ~ `con` (crates/tc2/src/contrato.rs:115), `con` (crates/tc2/src/doctor/mod.rs:118), `con` (crates/tc2/src/i18n/mod.rs:128) y 5 mas

