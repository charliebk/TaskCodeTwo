# Peticion de revision — TASK-022 (ronda 1)

- Tarea: TASK-022 — F2-9 Adaptador GitLab
- Rama revisada: feature/task-022-f2-9-adaptador-gitlab
- Rama base: develop
- Commit revisado (HEAD): 304782b8e59e4293ddbdd512d739b3979a0ac971
- Fecha: 2026-11-15
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-022 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-022/tarea.md
- tareas/03-en-revision/TASK-022/planificacion/
- crates/tc2/src/plataforma/conexion.rs
- crates/tc2/src/plataforma/deteccion.rs
- crates/tc2/src/plataforma/errores.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/gitlab.rs
- crates/tc2/src/plataforma/http.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/src/plataforma/puertas.rs
- crates/tc2/tests/comun/http_mock.rs
- crates/tc2/tests/plataforma_gitlab.rs
- crates/tc2/tests/plataforma_gitlab_e2e.rs

## Commits a revisar (git log develop..HEAD)

````
304782b chore(TASK-022): constancia de autorrevision y evidencia E2E
904be48 refactor(TASK-022): autorrevision del adaptador GitLab
773c4c6 test(TASK-022): formas de GitLab observadas en el E2E
5161b73 fix(TASK-022): puertas de GitLab segun lo observado en el E2E
70ba7bd feat(TASK-022): adaptador GitLab con la bateria compartida de F2-6
f31f910 refactor(TASK-022): transporte comun en conexion y errores por plataforma
04aacde chore(TASK-022): tarea en curso
````

## Diff de tu dominio (11 fichero(s) de 18; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/src/plataforma/conexion.rs b/crates/tc2/src/plataforma/conexion.rs
new file mode 100644
index 0000000..4c6c49c
--- /dev/null
+++ b/crates/tc2/src/plataforma/conexion.rs
@@ -0,0 +1,275 @@
+//! Transporte comun de los adaptadores (F2-6 y F2-9): el token de cada rol,
+//! la llamada, la lectura tipada, la paginacion, la cache de identidades y
+//! la comprobacion de que la identidad de persona esta en la lista.
+//!
+//! Cada adaptador conserva su lista cerrada de endpoints y solo le pasa el
+//! metodo y la ruta ya construidos: asi los dos paginan, manejan el token y
+//! fallan cerrado igual.
+
+use std::cell::RefCell;
+use std::collections::BTreeMap;
+
+use serde::de::DeserializeOwned;
+
+use super::credenciales::{Credenciales, Rol, Secreto};
+use super::errores;
+use super::http::{Base, Cliente, Metodo, Respuesta};
+use super::{Paso, Personas, Proveedor, Usuario};
+use crate::contrato::{Informe, Resultado};
+
+/// Paginas como mucho al seguir `Link rel=next` (100 elementos cada una).
+pub const MAX_PAGINAS: usize = 10;
+
+/// Conexion de un adaptador con su plataforma.
+pub struct Conexion {
+    cliente: Cliente,
+    proveedor: Proveedor,
+    credenciales: Box<dyn Credenciales>,
+    personas: Personas,
+    identidades: RefCell<BTreeMap<Rol, Usuario>>,
+}
+
+impl std::fmt::Debug for Conexion {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("Conexion")
+            .field("cliente", &self.cliente)
+            .finish_non_exhaustive()
+    }
+}
+
+impl Conexion {
+    /// Conexion a `base` para `proveedor`.
+    pub fn nueva(
+        base: Base,
+        proveedor: Proveedor,
+        credenciales: Box<dyn Credenciales>,
+        personas: Personas,
+    ) -> Self {
+        Self {
+            cliente: Cliente::para(base, proveedor),
+            proveedor,
+            credenciales,
+            personas,
+            identidades: RefCell::new(BTreeMap::new()),
+        }
+    }
+
+    /// La plataforma.
+    pub fn proveedor(&self) -> Proveedor {
+        self.proveedor
+    }
+
+    /// La lista de personas.
+    pub fn personas(&self) -> &Personas {
+        &self.personas
+    }
+
+    /// Llamadas hechas, `METODO ruta`.
+    pub fn registro(&self) -> Vec<String> {
+        self.cliente.registro()
+    }
+
+    /// Token del rol, o el rechazo dirigido si falta (nunca se prueba con el
+    /// de otro rol).
+    pub fn token(&self, op: &str, rol: Rol) -> Paso<Secreto> {
+        self.credenciales.token(rol).ok_or_else(|| {
+            let extra = if rol == Rol::Persona {
+                " (el de persona solo se usa desde un terminal interactivo)"
+            } else {
+                ""
+            };
+            Informe::nuevo(op, Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("falta", rol.nombre())
+                .diagnostico(format!(
+                    "esta operacion corre con la identidad de {}: define {}{extra}",
+                    rol.nombre(),
+                    rol.variable()
+                ))
+        })
+    }
+
+    /// Llamada cruda: cualquier estado HTTP llega como respuesta.
+    pub fn pedir(
+        &self,
+        op: &str,
+        rol: Rol,
+        (metodo, ruta): (Metodo, String),
+        cuerpo: Option<&str>,
+    ) -> Paso<Respuesta> {
+        let token = self.token(op, rol)?;
+        self.cliente
+            .llamar(metodo, &ruta, Some(&token), cuerpo)
+            .map_err(|f| errores::fallo(op, &f))
+    }
+
+    /// Traduce una respuesta que no es 2xx.
+    pub fn traducir(&self, op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
+        errores::traducir(self.proveedor, op, rol, permiso, r)
+    }
+
+    /// Respuesta 2xx con una forma inesperada.
+    pub fn invalida(&self, op: &str, que: &str) -> Informe {
+        errores::respuesta_invalida(self.proveedor, op, que)
+    }
+
+    /// Lee un cuerpo con la forma `T`.
+    pub fn parsear<T: DeserializeOwned>(&self, op: &str, cuerpo: &str) -> Paso<T> {
+        serde_json::from_str(cuerpo)
+            .map_err(|e| self.invalida(op, &errores::sanear(&e.to_string())))
+    }
+
+    /// Lectura que exige 2xx y la forma `T`.
+    pub fn leer<T: DeserializeOwned>(
+        &self,
+        op: &str,
+        rol: Rol,
+        permiso: &str,
+        ep: (Metodo, String),
+    ) -> Paso<T> {
+        let r = self.pedir(op, rol, ep, None)?;
+        if !r.exito() {
+            return Err(self.traducir(op, rol, permiso, &r));
+        }
+        self.parsear(op, &r.cuerpo)
+    }
+
+    /// Lista paginada: sigue `Link rel=next` en la misma base, hasta
+    /// [`MAX_PAGINAS`]; con mas, falla cerrado.
+    pub fn leer_paginas<T, F>(
+        &self,
+        op: &str,
+        rol: Rol,
+        permiso: &str,
+        primera: String,
+        mut extraer: F,
+    ) -> Paso<Vec<T>>
+    where
+        F: FnMut(&str) -> Paso<Vec<T>>,
+    {
+        let token = self.token(op, rol)?;
+        let mut ruta = primera;
+        let mut todo = Vec::new();
+        for _ in 0..MAX_PAGINAS {
+            let r = self
+                .cliente
+                .llamar(Metodo::Get, &ruta, Some(&token), None)
+                .map_err(|f| errores::fallo(op, &f))?;
+            if !r.exito() {
+                return Err(self.traducir(op, rol, permiso, &r));
+            }
+            todo.extend(extraer(&r.cuerpo)?);
+            match r.siguiente() {
+                Some(url) => {
+                    ruta = self
+                        .cliente
+                        .ruta_de(url)
+                        .map_err(|f| errores::fallo(op, &f))?;
+                }
+                None => return Ok(todo),
+            }
+        }
+        Err(Informe::nuevo(op, Resultado::Error)
+            .con("motivo", "demasiadas-paginas")
+            .diagnostico(format!(
+                "la lista tiene mas de {} elementos: no se evalua a medias",
+                MAX_PAGINAS * 100
+            )))
+    }
+
+    /// Identidad del rol, de la cache o leida con `leer` (una vez por rol y
+    /// proceso).
+    pub fn identidad(&self, rol: Rol, leer: impl FnOnce() -> Paso<Usuario>) -> Paso<Usuario> {
+        if let Some(u) = self.identidades.borrow().get(&rol) {
+            return Ok(u.clone());
+        }
+        let u = leer()?;
+        self.identidades.borrow_mut().insert(rol, u.clone());
+        Ok(u)
+    }
+
+    /// La identidad de persona (`persona`) tiene que estar en la lista.
+    pub fn exigir_persona(&self, op: &str, persona: Paso<Usuario>) -> Paso<()> {
+        let u = persona.map_err(|mut i| {
+            op.clone_into(&mut i.comando);
+            i
+        })?;
+        if u.es_persona(&self.personas) {
+            Ok(())
+        } else {
+            Err(Informe::nuevo(op, Resultado::Rechazada)
+                .con("motivo", "identidad")
+                .con("rol", "persona")
+                .diagnostico(format!(
+                    "el token de persona es de «{}», que no esta en la lista de personas del proyecto",
+                    errores::sanear(&u.login)
+                )))
+        }
+    }
+}
+
+/// Rama aceptable en una ruta de la API (la misma forma que exige Git, mas
+/// estricta).
+pub fn rama_valida(r: &str) -> bool {
+    !r.is_empty()
+        && r.len() <= 200
+        && !r.starts_with(['-', '/', '.'])
+        && !r.ends_with(['/', '.'])
+        && !r.contains("..")
+        && !r.contains("//")
+        && !r.ends_with(".lock")
+        && r.bytes()
+            .all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
+}
+
+/// Sha completo (40 hex).
+pub fn sha_valido(s: &str) -> bool {
+    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
+}
+
+/// Informe de uso (2) con su motivo.
+pub fn uso(op: &str, motivo: &str, diag: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Uso)
+        .con("motivo", motivo)
+        .diagnostico(diag.to_owned())
+}
+
+/// `merge-pr` rechazado porque el head no es el evaluado.
+pub fn head_cambiado(pr: u64) -> Informe {
+    Informe::nuevo("merge-pr", Resultado::Rechazada)
+        .con("pr", pr.to_string())
+        .con("motivo", "head-cambiado")
+        .diagnostico(
+            "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas",
+        )
+}
+
+/// Rechazo de un rol que no corresponde a la operacion.
+pub fn rol_no_valido(op: &str, rol: Rol) -> Informe {
+    Informe::nuevo(op, Resultado::Rechazada)
+        .con("motivo", "identidad")
+        .con("rol", rol.nombre())
+        .diagnostico(format!(
+            "{op} corre con la identidad de agente o de persona"
+        ))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn ramas_y_shas() {
+        for r in ["develop", "feature/task-021-f2-6-adaptador-github", "a.b_c"] {
+            assert!(rama_valida(r), "{r}");
+        }
+        for r in [
+            "", "-x", "/x", "x/", "a..b", "a//b", "a b", "a%2e", "x.lock", ".x",
+        ] {
+            assert!(!rama_valida(r), "{r}");
+        }
+        assert!(sha_valido(&"a".repeat(40)));
+        assert!(!sha_valido(&"g".repeat(40)));
+        assert!(!sha_valido("abc"));
+    }
+}
diff --git a/crates/tc2/src/plataforma/deteccion.rs b/crates/tc2/src/plataforma/deteccion.rs
index 000097e..34366aa 100644
--- a/crates/tc2/src/plataforma/deteccion.rs
+++ b/crates/tc2/src/plataforma/deteccion.rs
@@ -12,24 +12,7 @@ use crate::contrato::{Informe, Resultado};
 /// Nombre de la operacion en los errores.
 const OP: &str = "detectar-plataforma";
 
-/// Plataforma soportada.
-#[derive(Debug, Clone, Copy, PartialEq, Eq)]
-pub enum Proveedor {
-    /// GitHub.
-    GitHub,
-    /// GitLab.
-    GitLab,
-}
-
-impl Proveedor {
-    /// Nombre en la configuracion y en la linea.
-    pub fn nombre(self) -> &'static str {
-        match self {
-            Self::GitHub => "github",
-            Self::GitLab => "gitlab",
-        }
-    }
-}
+pub use super::Proveedor;
 
 /// A donde habla el puerto.
 #[derive(Debug, Clone, PartialEq, Eq)]
diff --git a/crates/tc2/src/plataforma/errores.rs b/crates/tc2/src/plataforma/errores.rs
index d4f21dd..8a70cb1 100644
--- a/crates/tc2/src/plataforma/errores.rs
+++ b/crates/tc2/src/plataforma/errores.rs
@@ -1,11 +1,13 @@
-//! De HTTP a la linea de contrato, con el arreglo para la persona (F2-6:
-//! «errores dirigidos a la persona»). Es el unico sitio que conoce los
-//! codigos HTTP; el adaptador de GitLab (F2-9) lo reutiliza.
+//! De HTTP a la linea de contrato, con el arreglo para la persona (F2-6 y
+//! F2-9: «errores dirigidos a la persona»). Es el unico sitio que conoce los
+//! codigos HTTP, y lo comparten los adaptadores de GitHub y GitLab: el
+//! [`Proveedor`] pone el nombre de la plataforma en el diagnostico.
 //!
 //! El diagnostico dice que hacer. El mensaje de la API es dato no confiable
 //! (F1-8, Z2): se recorta y se quitan los caracteres de control antes de
 //! citarlo. Nunca lleva el token ni cabeceras.
 
+use super::Proveedor;
 use super::credenciales::Rol;
 use super::http::{FalloHttp, Respuesta};
 use crate::contrato::{Informe, Resultado};
@@ -13,12 +15,18 @@ use crate::contrato::{Informe, Resultado};
 /// Aviso de GitHub cuando el plan no da rulesets ni proteccion de rama.
 pub const AVISO_PLAN: &str = "Upgrade to GitHub Pro";
 
-/// Mensaje de la API (`message`), saneado: sin controles y como mucho 200
-/// caracteres.
+/// Mensaje de la API, saneado: sin controles y como mucho 200 caracteres.
+/// GitHub lo da en `message`; GitLab, en `message` (texto, objeto o lista)
+/// o en `error`.
 pub fn mensaje_api(r: &Respuesta) -> String {
-    let m = serde_json::from_str::<serde_json::Value>(&r.cuerpo)
-        .ok()
-        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(str::to_owned))
+    let v = serde_json::from_str::<serde_json::Value>(&r.cuerpo).ok();
+    let m = v
+        .as_ref()
+        .and_then(|v| v.get("message").or_else(|| v.get("error")))
+        .map(|m| match m.as_str() {
+            Some(s) => s.to_owned(),
+            None => m.to_string(),
+        })
         .unwrap_or_default();
     sanear(&m)
 }
@@ -41,7 +49,7 @@ pub fn es_aviso_de_plan(r: &Respuesta) -> bool {
 /// Si la respuesta es un limite de peticiones agotado.
 fn es_limite(r: &Respuesta) -> bool {
     // El limite secundario de GitHub puede llegar sin cabeceras: entonces lo
-    // dice el mensaje.
+    // dice el mensaje. GitLab da 429.
     (r.estado == 403 || r.estado == 429)
         && (r.restantes.as_deref() == Some("0")
             || r.reintentar.is_some()
@@ -70,12 +78,17 @@ fn es_numero(s: &str) -> bool {
 
 /// Traduce una respuesta que no es 2xx. `op` es la operacion, `rol` con que
 /// identidad corria y `permiso` el que necesita (para el 403 sin plan).
-pub fn traducir(op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
+pub fn traducir(p: Proveedor, op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
     let api = mensaje_api(r);
+    let nombre = p.titulo();
     let cita = if api.is_empty() {
         String::new()
     } else {
-        format!(" (GitHub: «{api}»)")
+        format!(" ({nombre}: «{api}»)")
+    };
+    let objeto = match p {
+        Proveedor::GitHub => "el repo, el PR o el commit",
+        Proveedor::GitLab => "el proyecto, el MR o el commit",
     };
     match r.estado {
         401 => Informe::nuevo(op, Resultado::Denegada)
@@ -95,11 +108,13 @@ pub fn traducir(op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
                     "limite de peticiones de la API agotado: espera {s} s y relanza{cita}"
                 ))
         }
-        403 if api.contains(AVISO_PLAN) => Informe::nuevo(op, Resultado::Rechazada)
-            .con("motivo", "plan")
-            .diagnostico(format!(
-                "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}"
-            )),
+        403 if p == Proveedor::GitHub && api.contains(AVISO_PLAN) => {
+            Informe::nuevo(op, Resultado::Rechazada)
+                .con("motivo", "plan")
+                .diagnostico(format!(
+                    "el plan de GitHub no da esta funcion en un repo privado: haz el repo publico o usa GitHub Pro, Team o Enterprise{cita}"
+                ))
+        }
         403 => Informe::nuevo(op, Resultado::Denegada)
             .con("motivo", "permisos")
             .con("rol", rol.nombre())
@@ -111,37 +126,49 @@ pub fn traducir(op: &str, rol: Rol, permiso: &str, r: &Respuesta) -> Informe {
             .con("motivo", "no-existe-o-sin-acceso")
             .con("rol", rol.nombre())
             .diagnostico(format!(
-                "no existe o el token de {} no lo ve: revisa el repo, el PR o el commit y el alcance del token (GitHub da 404 en un repo privado sin acceso){cita}",
+                "no existe o el token de {} no lo ve: revisa {objeto} y el alcance del token ({nombre} da 404 en un repo privado sin acceso){cita}",
                 rol.nombre()
             )),
-        405 => Informe::nuevo(op, Resultado::Rechazada)
-            .con("motivo", "no-mergeable")
-            .diagnostico(format!(
-                "GitHub no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}"
-            )),
+        // GitLab da 405 o 406 cuando el MR no se puede mergear.
+        405 => no_mergeable(op, nombre, &cita),
+        406 if p == Proveedor::GitLab => no_mergeable(op, nombre, &cita),
         409 => Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "head-cambiado")
             .diagnostico(format!(
                 "la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas{cita}"
             )),
-        // El detalle del 422 viene en `errors[].message`, no en `message`.
+        // El detalle del 422 de GitHub viene en `errors[].message`, no en
+        // `message`.
         422 if r.cuerpo.contains("No commits between") => Informe::nuevo(op, Resultado::Rechazada)
             .con("motivo", "sin-commits")
             .diagnostico(format!(
                 "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar{cita}"
             )),
-        422 => Informe::nuevo(op, Resultado::Error)
-            .con("motivo", "validacion")
-            .diagnostico(format!("GitHub rechazo los datos de la peticion{cita}")),
+        400 if p == Proveedor::GitLab => validacion(op, nombre, &cita),
+        422 => validacion(op, nombre, &cita),
         e => Informe::nuevo(op, Resultado::Error)
             .con("motivo", "http")
             .con("estado", e.to_string())
             .diagnostico(format!(
-                "respuesta inesperada de GitHub ({e}): reintenta mas tarde{cita}"
+                "respuesta inesperada de {nombre} ({e}): reintenta mas tarde{cita}"
             )),
     }
 }
 
+fn no_mergeable(op: &str, nombre: &str, cita: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Rechazada)
+        .con("motivo", "no-mergeable")
+        .diagnostico(format!(
+            "{nombre} no deja mergear: faltan puertas o hay conflictos; lanza estado-puertas{cita}"
+        ))
+}
+
+fn validacion(op: &str, nombre: &str, cita: &str) -> Informe {
+    Informe::nuevo(op, Resultado::Error)
+        .con("motivo", "validacion")
+        .diagnostico(format!("{nombre} rechazo los datos de la peticion{cita}"))
+}
+
 /// Traduce un fallo sin respuesta.
 pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
     match f {
@@ -179,10 +206,11 @@ pub fn fallo(op: &str, f: &FalloHttp) -> Informe {
 }
 
 /// Respuesta 2xx con un cuerpo que no tiene la forma esperada.
-pub fn respuesta_invalida(op: &str, que: &str) -> Informe {
+pub fn respuesta_invalida(p: Proveedor, op: &str, que: &str) -> Informe {
     Informe::nuevo(op, Resultado::Error)
         .con("motivo", "respuesta-invalida")
         .diagnostico(format!(
-            "la respuesta de GitHub no tiene la forma esperada ({que})"
+            "la respuesta de {} no tiene la forma esperada ({que})",
+            p.titulo()
         ))
 }
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index 9569247..ef77bc3 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -4,19 +4,19 @@
 //! Cada endpoint es una variante de [`Endpoint`], que construye el metodo y
 //! la ruta: no hay llamadas con una URL libre. Las respuestas son dato no
 //! confiable (F1-8, Z2): se leen con structs que solo tienen los campos que
-//! se usan y un campo ausente da `error motivo=respuesta-invalida`.
+//! se usan y un campo ausente da `error motivo=respuesta-invalida`. El
+//! transporte es el comun de [`super::conexion`].
 
-use std::cell::RefCell;
 use std::collections::BTreeMap;
 
 use serde::Deserialize;
-use serde::de::DeserializeOwned;
 
-use super::credenciales::{Credenciales, Rol, Secreto};
+use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_valido, uso};
+use super::credenciales::{Credenciales, Rol};
 use super::errores::{self, es_aviso_de_plan};
-use super::http::{Base, Cliente, Metodo, Respuesta, codificar};
+use super::http::{Base, Metodo, codificar};
 use super::puertas::{self, EstadoCheck, Lecturas};
-use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, ReglasLeidas, Usuario};
+use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario};
 use crate::contrato::{Informe, Resultado, Valor, corto};
 
 const ABRIR_PR: &str = "abrir-pr";
@@ -26,9 +26,6 @@ const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
 const FIRMA_COMMIT: &str = "firma-commit";
 const IDENTIDAD: &str = "identidad";
 
-/// Paginas como mucho al seguir `Link rel=next` (100 elementos cada una).
-const MAX_PAGINAS: usize = 10;
-
 /// Garantias que en GitHub dan los rulesets (tier «Free publico;
 /// Pro/Team/Enterprise» de la tabla de F1-13): sin ellos, el 403 de plan.
 const GARANTIAS_DE_RULESETS: [&str; 10] = [
@@ -190,17 +187,14 @@ struct ReglaApi {
 
 /// Adaptador de GitHub para un repo.
 pub struct GitHub {
-    cliente: Cliente,
+    con: Conexion,
     repo: String,
-    credenciales: Box<dyn Credenciales>,
-    personas: Personas,
-    identidades: RefCell<BTreeMap<Rol, Usuario>>,
 }
 
 impl std::fmt::Debug for GitHub {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         f.debug_struct("GitHub")
-            .field("cliente", &self.cliente)
+            .field("con", &self.con)
             .field("repo", &self.repo)
             .finish_non_exhaustive()
     }
@@ -229,141 +223,36 @@ impl GitHub {
                 .diagnostico("el repo de GitHub tiene que ser dueno/repo"));
         }
         Ok(Self {
-            cliente: Cliente::nuevo(base),
+            con: Conexion::nueva(base, Proveedor::GitHub, credenciales, personas),
             repo: repo.to_owned(),
-            credenciales,
-            personas,
-            identidades: RefCell::new(BTreeMap::new()),
         })
     }
 
     /// Llamadas hechas, `METODO ruta` (para medir el coste por operacion).
     pub fn registro(&self) -> Vec<String> {
-        self.cliente.registro()
-    }
-
-    fn token(&self, op: &str, rol: Rol) -> Paso<Secreto> {
-        self.credenciales.token(rol).ok_or_else(|| {
-            let extra = if rol == Rol::Persona {
-                " (el de persona solo se usa desde un terminal interactivo)"
-            } else {
-                ""
-            };
-            Informe::nuevo(op, Resultado::Rechazada)
-                .con("motivo", "identidad")
-                .con("falta", rol.nombre())
-                .diagnostico(format!(
-                    "esta operacion corre con la identidad de {}: define {}{extra}",
-                    rol.nombre(),
-                    rol.variable()
-                ))
-        })
-    }
-
-    /// Llamada cruda: cualquier estado HTTP llega como respuesta.
-    fn pedir(
-        &self,
-        op: &str,
-        rol: Rol,
-        ep: &Endpoint<'_>,
-        cuerpo: Option<&str>,
-    ) -> Paso<Respuesta> {
-        let token = self.token(op, rol)?;
-        let (metodo, ruta) = ep.metodo_y_ruta(&self.repo);
-        self.cliente
-            .llamar(metodo, &ruta, Some(&token), cuerpo)
-            .map_err(|f| errores::fallo(op, &f))
+        self.con.registro()
     }
 
-    /// Lectura que exige 2xx y la forma `T`.
-    fn leer<T: DeserializeOwned>(
-        &self,
-        op: &str,
-        rol: Rol,
-        permiso: &str,
-        ep: &Endpoint<'_>,
-    ) -> Paso<T> {
-        let r = self.pedir(op, rol, ep, None)?;
-        if !r.exito() {
-            return Err(errores::traducir(op, rol, permiso, &r));
-        }
-        parsear(op, &r.cuerpo)
-    }
-
-    /// Lista paginada: sigue `Link rel=next` en la misma base, hasta
-    /// [`MAX_PAGINAS`].
-    fn leer_paginas<T, F>(
-        &self,
-        op: &str,
-        rol: Rol,
-        permiso: &str,
-        ep: &Endpoint<'_>,
-        mut extraer: F,
-    ) -> Paso<Vec<T>>
-    where
-        F: FnMut(&str) -> Paso<Vec<T>>,
-    {
-        let token = self.token(op, rol)?;
-        let (_, mut ruta) = ep.metodo_y_ruta(&self.repo);
-        let mut todo = Vec::new();
-        for _ in 0..MAX_PAGINAS {
-            let r = self
-                .cliente
-                .llamar(Metodo::Get, &ruta, Some(&token), None)
-                .map_err(|f| errores::fallo(op, &f))?;
-            if !r.exito() {
-                return Err(errores::traducir(op, rol, permiso, &r));
-            }
-            todo.extend(extraer(&r.cuerpo)?);
-            match r.siguiente() {
-                Some(url) => {
-                    ruta = self
-                        .cliente
-                        .ruta_de(url)
-                        .map_err(|f| errores::fallo(op, &f))?;
-                }
-                None => return Ok(todo),
-            }
-        }
-        Err(Informe::nuevo(op, Resultado::Error)
-            .con("motivo", "demasiadas-paginas")
-            .diagnostico(format!(
-                "la lista tiene mas de {} elementos: no se evalua a medias",
-                MAX_PAGINAS * 100
-            )))
+    fn ep(&self, ep: &Endpoint<'_>) -> (Metodo, String) {
+        ep.metodo_y_ruta(&self.repo)
     }
 
     /// La identidad de persona tiene que estar en la lista de personas.
     fn exigir_persona(&self, op: &str) -> Paso<()> {
-        let u = self.identidad(Rol::Persona).map_err(|mut i| {
-            op.clone_into(&mut i.comando);
-            i
-        })?;
-        if u.es_persona(&self.personas) {
-            Ok(())
-        } else {
-            Err(Informe::nuevo(op, Resultado::Rechazada)
-                .con("motivo", "identidad")
-                .con("rol", "persona")
-                .diagnostico(format!(
-                    "el token de persona es de «{}», que no esta en la lista de personas del proyecto",
-                    errores::sanear(&u.login)
-                )))
-        }
+        self.con.exigir_persona(op, self.identidad(Rol::Persona))
     }
 
     fn pr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
         let cabeza = format!("{}:{rama}", self.dueno());
-        let prs: Vec<PrApi> = self.leer_paginas(
-            ABRIR_PR,
-            Rol::Agente,
-            "pull_requests:read",
-            &Endpoint::PrsAbiertos {
-                cabeza: &cabeza,
-                base,
-            },
-            |c| parsear(ABRIR_PR, c),
-        )?;
+        let (_, ruta) = self.ep(&Endpoint::PrsAbiertos {
+            cabeza: &cabeza,
+            base,
+        });
+        let prs: Vec<PrApi> =
+            self.con
+                .leer_paginas(ABRIR_PR, Rol::Agente, "pull_requests:read", ruta, |c| {
+                    self.con.parsear(ABRIR_PR, c)
+                })?;
         Ok(prs
             .into_iter()
             .find(|p| p.head.nombre == rama && p.base.nombre == base)
@@ -375,35 +264,6 @@ impl GitHub {
     }
 }
 
-fn parsear<T: DeserializeOwned>(op: &str, cuerpo: &str) -> Paso<T> {
-    serde_json::from_str(cuerpo)
-        .map_err(|e| errores::respuesta_invalida(op, &errores::sanear(&e.to_string())))
-}
-
-/// Rama aceptable en una ruta de la API (la misma forma que exige Git, mas
-/// estricta).
-fn rama_valida(r: &str) -> bool {
-    !r.is_empty()
-        && r.len() <= 200
-        && !r.starts_with(['-', '/', '.'])
-        && !r.ends_with(['/', '.'])
-        && !r.contains("..")
-        && !r.contains("//")
-        && !r.ends_with(".lock")
-        && r.bytes()
-            .all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
-}
-
-fn sha_valido(s: &str) -> bool {
-    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
-}
-
-fn uso(op: &str, motivo: &str, diag: &str) -> Informe {
-    Informe::nuevo(op, Resultado::Uso)
-        .con("motivo", motivo)
-        .diagnostico(diag.to_owned())
-}
-
 impl Plataforma for GitHub {
     fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
         if !rama_valida(rama) || !rama_valida(base) {
@@ -427,7 +287,12 @@ impl Plataforma for GitHub {
             "title": titulo, "head": rama, "base": base, "body": cuerpo,
         })
         .to_string();
-        let r = match self.pedir(ABRIR_PR, Rol::Agente, &Endpoint::CrearPr, Some(&datos)) {
+        let r = match self.con.pedir(
+            ABRIR_PR,
+            Rol::Agente,
+            self.ep(&Endpoint::CrearPr),
+            Some(&datos),
+        ) {
             Ok(r) => r,
             Err(i) => return i,
         };
@@ -437,14 +302,18 @@ impl Plataforma for GitHub {
                 Ok(Some(n)) => {
                     Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
                 }
-                Ok(None) => errores::traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r),
+                Ok(None) => self
+                    .con
+                    .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r),
                 Err(i) => i,
             };
         }
         if !r.exito() {
-            return errores::traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r);
+            return self
+                .con
+                .traducir(ABRIR_PR, Rol::Agente, "pull_requests:write", &r);
         }
-        match parsear::<PrApi>(ABRIR_PR, &r.cuerpo) {
+        match self.con.parsear::<PrApi>(ABRIR_PR, &r.cuerpo) {
             Ok(p) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", p.number.to_string()),
             Err(i) => i,
         }
@@ -453,10 +322,7 @@ impl Plataforma for GitHub {
     fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
         let op = puertas::ESTADO_PUERTAS;
         if rol == Rol::Revisor {
-            return Informe::nuevo(op, Resultado::Rechazada)
-                .con("motivo", "identidad")
-                .con("rol", rol.nombre())
-                .diagnostico("estado-puertas corre con la identidad de agente o de persona");
+            return rol_no_valido(op, rol);
         }
         if rol == Rol::Persona
             && let Err(i) = self.exigir_persona(op)
@@ -464,7 +330,7 @@ impl Plataforma for GitHub {
             return i;
         }
         let permiso = "pull_requests:read y checks:read";
-        let p: PrApi = match self.leer(op, rol, permiso, &Endpoint::Pr(pr)) {
+        let p: PrApi = match self.con.leer(op, rol, permiso, self.ep(&Endpoint::Pr(pr))) {
             Ok(p) => p,
             Err(i) => return i,
         };
@@ -472,21 +338,23 @@ impl Plataforma for GitHub {
             return puertas::mergeado(pr);
         }
         if p.state != "open" {
-            return Informe::nuevo(op, Resultado::Rechazada)
-                .con("pr", pr.to_string())
-                .con("motivo", "pr-cerrado")
-                .diagnostico("el PR esta cerrado sin mergear: reabrelo o abre otro");
+            return puertas::pr_cerrado(pr);
         }
         if !rama_valida(&p.base.nombre) || !sha_valido(&p.head.sha) {
-            return errores::respuesta_invalida(op, "rama base o sha del head");
+            return self.con.invalida(op, "rama base o sha del head");
         }
-        let r = match self.pedir(op, rol, &Endpoint::ReglasRama(&p.base.nombre), None) {
+        let r = match self.con.pedir(
+            op,
+            rol,
+            self.ep(&Endpoint::ReglasRama(&p.base.nombre)),
+            None,
+        ) {
             Ok(r) => r,
             Err(i) => return i,
         };
         let plan_da_reglas = !es_aviso_de_plan(&r);
         if plan_da_reglas && !r.exito() {
-            return errores::traducir(op, rol, permiso, &r);
+            return self.con.traducir(op, rol, permiso, &r);
         }
         if !plan_da_reglas
             && let Some(g) = modelo
@@ -496,13 +364,15 @@ impl Plataforma for GitHub {
         {
             return puertas::rechazada_por_plan(pr, g);
         }
-        let runs: Vec<CheckRunApi> =
-            match self.leer_paginas(op, rol, permiso, &Endpoint::CheckRuns(&p.head.sha), |c| {
-                parsear::<CheckRunsApi>(op, c).map(|x| x.check_runs)
-            }) {
-                Ok(v) => v,
-                Err(i) => return i,
-            };
+        let (_, ruta) = self.ep(&Endpoint::CheckRuns(&p.head.sha));
+        let runs: Vec<CheckRunApi> = match self.con.leer_paginas(op, rol, permiso, ruta, |c| {
+            self.con
+                .parsear::<CheckRunsApi>(op, c)
+                .map(|x| x.check_runs)
+        }) {
+            Ok(v) => v,
+            Err(i) => return i,
+        };
         // La API da los check-runs del mas nuevo al mas viejo: cuenta el
         // primero de cada nombre que publico su origen (integration_id).
         let checks = modelo
@@ -529,11 +399,14 @@ impl Plataforma for GitHub {
                 (nombre.clone(), estado)
             })
             .collect();
-        let reviews: Vec<ReviewApi> =
-            match self.leer_paginas(op, rol, permiso, &Endpoint::Reviews(pr), |c| parsear(op, c)) {
-                Ok(v) => v,
-                Err(i) => return i,
-            };
+        let (_, ruta) = self.ep(&Endpoint::Reviews(pr));
+        let reviews: Vec<ReviewApi> = match self
+            .con
+            .leer_paginas(op, rol, permiso, ruta, |c| self.con.parsear(op, c))
+        {
+            Ok(v) => v,
+            Err(i) => return i,
+        };
         // Las reviews vienen de la mas vieja a la mas nueva: cuenta la ultima
         // de cada cuenta, por id (F1-6, regla 6). Solo cambian el estado las
         // que deciden: un comentario o una review pendiente no retiran una
@@ -555,7 +428,7 @@ impl Plataforma for GitHub {
             .collect();
         let aprobaciones = aprobadas
             .iter()
-            .filter(|u| u.es_persona(&self.personas))
+            .filter(|u| u.es_persona(self.con.personas()))
             .count();
         puertas::evaluar(&Lecturas {
             pr,
@@ -581,10 +454,14 @@ impl Plataforma for GitHub {
             return i;
         }
         let permiso = "contents:write";
-        let p: PrApi = match self.leer(MERGE_PR, Rol::Persona, permiso, &Endpoint::Pr(pr)) {
-            Ok(p) => p,
-            Err(i) => return i,
-        };
+        let p: PrApi =
+            match self
+                .con
+                .leer(MERGE_PR, Rol::Persona, permiso, self.ep(&Endpoint::Pr(pr)))
+            {
+                Ok(p) => p,
+                Err(i) => return i,
+            };
         if p.merged {
             return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
                 .con("pr", pr.to_string())
@@ -597,23 +474,25 @@ impl Plataforma for GitHub {
                 .diagnostico("el PR esta cerrado sin mergear");
         }
         if !p.head.sha.eq_ignore_ascii_case(sha) {
-            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
-                .con("pr", pr.to_string())
-                .con("motivo", "head-cambiado")
-                .diagnostico("la rama ha cambiado desde que se evaluaron las puertas: vuelve a lanzar estado-puertas");
+            return head_cambiado(pr);
         }
         let datos = serde_json::json!({ "merge_method": "merge", "sha": sha }).to_string();
-        let r = match self.pedir(MERGE_PR, Rol::Persona, &Endpoint::Merge(pr), Some(&datos)) {
+        let r = match self.con.pedir(
+            MERGE_PR,
+            Rol::Persona,
+            self.ep(&Endpoint::Merge(pr)),
+            Some(&datos),
+        ) {
             Ok(r) => r,
             Err(i) => return i,
         };
         if !r.exito() {
-            let mut i = errores::traducir(MERGE_PR, Rol::Persona, permiso, &r);
+            let mut i = self.con.traducir(MERGE_PR, Rol::Persona, permiso, &r);
             i.claves
                 .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
             return i;
         }
-        match parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
+        match self.con.parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
             Ok(MergeApi {
                 merged: true,
                 sha: Some(s),
@@ -623,7 +502,7 @@ impl Plataforma for GitHub {
             // Mergeado aunque la respuesta no traiga un sha legible: se dice
             // que esta hecho, sin inventar el commit.
             Ok(m) if m.merged => Informe::nuevo(MERGE_PR, Resultado::Ok).con("pr", pr.to_string()),
-            Ok(_) => errores::respuesta_invalida(MERGE_PR, "merged"),
+            Ok(_) => self.con.invalida(MERGE_PR, "merged"),
             Err(i) => i,
         }
     }
@@ -637,13 +516,12 @@ impl Plataforma for GitHub {
             ));
         }
         if rol == Rol::Revisor {
-            return Err(Informe::nuevo(LEER_PUERTAS, Resultado::Rechazada)
-                .con("motivo", "identidad")
-                .con("rol", rol.nombre())
-                .diagnostico("leer-puertas corre con la identidad de agente o de persona"));
+            return Err(rol_no_valido(LEER_PUERTAS, rol));
         }
         let permiso = "metadata:read";
-        let r = self.pedir(LEER_PUERTAS, rol, &Endpoint::Rulesets, None)?;
+        let r = self
+            .con
+            .pedir(LEER_PUERTAS, rol, self.ep(&Endpoint::Rulesets), None)?;
         if es_aviso_de_plan(&r) {
             return Ok(ReglasLeidas {
                 plan_da_reglas: false,
@@ -651,11 +529,15 @@ impl Plataforma for GitHub {
             });
         }
         if !r.exito() {
-            return Err(errores::traducir(LEER_PUERTAS, rol, permiso, &r));
-        }
-        let rulesets: Vec<RulesetApi> = parsear(LEER_PUERTAS, &r.cuerpo)?;
-        let reglas: Vec<ReglaApi> =
-            self.leer(LEER_PUERTAS, rol, permiso, &Endpoint::ReglasRama(rama))?;
+            return Err(self.con.traducir(LEER_PUERTAS, rol, permiso, &r));
+        }
+        let rulesets: Vec<RulesetApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
+        let reglas: Vec<ReglaApi> = self.con.leer(
+            LEER_PUERTAS,
+            rol,
+            permiso,
+            self.ep(&Endpoint::ReglasRama(rama)),
+        )?;
         Ok(ReglasLeidas {
             plan_da_reglas: true,
             rulesets: rulesets.into_iter().map(|r| r.name).collect(),
@@ -667,11 +549,11 @@ impl Plataforma for GitHub {
         if let Err(i) = self.exigir_persona(LEER_PERMISOS_CI) {
             return i;
         }
-        let p: PermisosApi = match self.leer(
+        let p: PermisosApi = match self.con.leer(
             LEER_PERMISOS_CI,
             Rol::Persona,
             "administration:read",
-            &Endpoint::PermisosWorkflow,
+            self.ep(&Endpoint::PermisosWorkflow),
         ) {
             Ok(p) => p,
             Err(i) => return i,
@@ -697,27 +579,26 @@ impl Plataforma for GitHub {
                 "el sha tiene que ser completo (40 hex)",
             ));
         }
-        let c: CommitApi = self.leer(
+        let c: CommitApi = self.con.leer(
             FIRMA_COMMIT,
             Rol::Agente,
             "contents:read",
-            &Endpoint::Commit(sha),
+            self.ep(&Endpoint::Commit(sha)),
         )?;
         Ok(Firma {
             verificada: c.commit.verification.verified,
             motivo: errores::sanear(&c.commit.verification.reason),
             committer: c.committer.map(Usuario::from),
+            clave: None,
         })
     }
 
     fn identidad(&self, rol: Rol) -> Paso<Usuario> {
-        if let Some(u) = self.identidades.borrow().get(&rol) {
-            return Ok(u.clone());
-        }
-        let u: UsuarioApi = self.leer(IDENTIDAD, rol, "(ninguno)", &Endpoint::Usuario)?;
-        let u = Usuario::from(u);
-        self.identidades.borrow_mut().insert(rol, u.clone());
-        Ok(u)
+        self.con.identidad(rol, || {
+            self.con
+                .leer::<UsuarioApi>(IDENTIDAD, rol, "(ninguno)", self.ep(&Endpoint::Usuario))
+                .map(Usuario::from)
+        })
     }
 }
 
@@ -725,21 +606,6 @@ impl Plataforma for GitHub {
 mod tests {
     use super::*;
 
-    #[test]
-    fn ramas_y_shas() {
-        for r in ["develop", "feature/task-021-f2-6-adaptador-github", "a.b_c"] {
-            assert!(rama_valida(r), "{r}");
-        }
-        for r in [
-            "", "-x", "/x", "x/", "a..b", "a//b", "a b", "a%2e", "x.lock", ".x",
-        ] {
-            assert!(!rama_valida(r), "{r}");
-        }
-        assert!(sha_valido(&"a".repeat(40)));
-        assert!(!sha_valido(&"g".repeat(40)));
-        assert!(!sha_valido("abc"));
-    }
-
     #[test]
     fn repo_invalido_es_uso() {
         for repo in ["o", "o/r/s", "o/..", "o/r?x", "/r"] {
diff --git a/crates/tc2/src/plataforma/gitlab.rs b/crates/tc2/src/plataforma/gitlab.rs
new file mode 100644
index 0000000..faa49e8
--- /dev/null
+++ b/crates/tc2/src/plataforma/gitlab.rs
@@ -0,0 +1,783 @@
+//! Adaptador de GitLab del puerto `Plataforma` (F2-9, ADR F1-13 decisiones
+//! 1, 4 y 6; F1-6 decision 1 para la identidad de cada operacion).
+//!
+//! Mismo trait y misma linea que el de GitHub (RR-15): lo que cambia es de
+//! donde sale cada lectura. Cada endpoint es una variante de [`Endpoint`]
+//! sobre la API REST v4, sin `glab`. El proyecto va en la ruta como id
+//! numerico o como `grupo%2Fproyecto`, codificado una sola vez. Las
+//! respuestas son dato no confiable (F1-8, Z2): structs con los campos que
+//! se usan, y un campo ausente da `error motivo=respuesta-invalida`.
+
+use std::collections::BTreeMap;
+
+use serde::Deserialize;
+
+use super::conexion::{Conexion, head_cambiado, rama_valida, rol_no_valido, sha_valido, uso};
+use super::credenciales::{Credenciales, Rol};
+use super::errores;
+use super::http::{Base, Metodo, codificar};
+use super::puertas::{self, EstadoCheck, Lecturas};
+use super::{Firma, ModeloPuertas, Paso, Personas, Plataforma, Proveedor, ReglasLeidas, Usuario};
+use crate::contrato::{Informe, Resultado, Valor, corto};
+
+const ABRIR_PR: &str = "abrir-pr";
+const MERGE_PR: &str = "merge-pr";
+const LEER_PUERTAS: &str = "leer-puertas";
+const LEER_PERMISOS_CI: &str = "leer-permisos-ci";
+const FIRMA_COMMIT: &str = "firma-commit";
+const IDENTIDAD: &str = "identidad";
+
+/// Permiso de lectura que necesita un token de GitLab.
+const PERMISO_LECTURA: &str = "el scope api (o read_api) y el rol Developer en el proyecto";
+
+/// Permiso para abrir un MR.
+const PERMISO_ESCRITURA: &str = "el scope api y el rol Developer en el proyecto";
+
+/// Permiso para mergear en una rama protegida.
+const PERMISO_MERGE: &str = "el scope api y el rol Maintainer en el proyecto";
+
+/// Garantias que en GitLab solo da Premium o Ultimate (tabla de F1-13): sin
+/// reglas de aprobacion (403 en Free), la politica no puede exigirlas
+/// impuestas.
+const GARANTIAS_PREMIUM: [&str; 6] = ["G03", "G04", "G05", "G06", "G10", "G12"];
+
+/// Nivel de acceso de Maintainer.
+const MAINTAINER: u64 = 40;
+
+/// Endpoints de la API que usa el adaptador (lista cerrada).
+enum Endpoint<'a> {
+    Usuario,
+    Mr(u64),
+    MrConDivergencia(u64),
+    MrsAbiertos { rama: &'a str, base: &'a str },
+    CommitsEntre { base: &'a str, rama: &'a str },
+    CrearMr,
+    Merge(u64),
+    ReglasAprobacion,
+    RamaProtegida(&'a str),
+    RamasProtegidas,
+    Estados(&'a str),
+    Aprobaciones(u64),
+    Firma(&'a str),
+    Commit(&'a str),
+}
+
+impl Endpoint<'_> {
+    fn metodo_y_ruta(&self, proyecto: &str) -> (Metodo, String) {
+        let p = format!("/projects/{proyecto}");
+        match self {
+            Self::Usuario => (Metodo::Get, "/user".into()),
+            Self::Mr(n) => (Metodo::Get, format!("{p}/merge_requests/{n}")),
+            Self::MrConDivergencia(n) => (
+                Metodo::Get,
+                format!("{p}/merge_requests/{n}?include_diverged_commits_count=true"),
+            ),
+            Self::MrsAbiertos { rama, base } => (
+                Metodo::Get,
+                format!(
+                    "{p}/merge_requests?source_branch={}&target_branch={}&state=opened&per_page=100",
+                    codificar(rama),
+                    codificar(base)
+                ),
+            ),
+            Self::CommitsEntre { base, rama } => (
+                Metodo::Get,
+                format!(
+                    "{p}/repository/commits?ref_name={}&per_page=1",
+                    codificar(&format!("{base}..{rama}"))
+                ),
+            ),
+            Self::CrearMr => (Metodo::Post, format!("{p}/merge_requests")),
+            Self::Merge(n) => (Metodo::Put, format!("{p}/merge_requests/{n}/merge")),
+            Self::ReglasAprobacion => (Metodo::Get, format!("{p}/approval_rules?per_page=100")),
+            Self::RamaProtegida(rama) => (
+                Metodo::Get,
+                format!("{p}/protected_branches/{}", codificar(rama)),
+            ),
+            Self::RamasProtegidas => (Metodo::Get, format!("{p}/protected_branches?per_page=100")),
+            Self::Estados(sha) => (
+                Metodo::Get,
+                format!("{p}/repository/commits/{sha}/statuses?per_page=100"),
+            ),
+            Self::Aprobaciones(n) => (Metodo::Get, format!("{p}/merge_requests/{n}/approvals")),
+            Self::Firma(sha) => (
+                Metodo::Get,
+                format!("{p}/repository/commits/{sha}/signature"),
+            ),
+            Self::Commit(sha) => (Metodo::Get, format!("{p}/repository/commits/{sha}")),
+        }
+    }
+}
+
+#[derive(Deserialize)]
+struct UsuarioApi {
+    #[serde(default)]
+    username: String,
+    id: u64,
+    #[serde(default)]
+    bot: bool,
+}
+
+impl From<UsuarioApi> for Usuario {
+    fn from(u: UsuarioApi) -> Self {
+        Self {
+            login: u.username,
+            id: u.id,
+            tipo: if u.bot { "Bot" } else { "User" }.to_owned(),
+        }
+    }
+}
+
+#[derive(Deserialize)]
+struct MrApi {
+    iid: u64,
+    state: String,
+    #[serde(default)]
+    sha: Option<String>,
+    target_branch: String,
+    source_branch: String,
+}
+
+#[derive(Deserialize)]
+struct MrPuertasApi {
+    state: String,
+    sha: String,
+    author: AutorApi,
+    target_branch: String,
+    has_conflicts: bool,
+    diverged_commits_count: u64,
+}
+
+#[derive(Deserialize)]
+struct NivelApi {
+    access_level: u64,
+}
+
+#[derive(Deserialize)]
+struct RamaProtegidaApi {
+    name: String,
+    merge_access_levels: Vec<NivelApi>,
+    #[serde(default)]
+    push_access_levels: Vec<NivelApi>,
+    allow_force_push: bool,
+    #[serde(default)]
+    code_owner_approval_required: bool,
+}
+
+#[derive(Deserialize)]
+struct AutorApi {
+    id: u64,
+}
+
+#[derive(Deserialize)]
+struct EstadoApi {
+    id: u64,
+    name: String,
+    status: String,
+    author: AutorApi,
+}
+
+#[derive(Deserialize)]
+struct AprobacionesApi {
+    approved_by: Vec<AprobadorApi>,
+}
+
+#[derive(Deserialize)]
+struct AprobadorApi {
+    user: UsuarioApi,
+}
+
+#[derive(Deserialize)]
+struct ReglaAprobacionApi {
+    name: String,
+    #[serde(default)]
+    approvals_required: u64,
+}
+
+#[derive(Deserialize)]
+struct MergeApi {
+    state: String,
+    #[serde(default)]
+    merge_commit_sha: Option<String>,
+}
+
+#[derive(Deserialize)]
+struct ClaveApi {
+    id: u64,
+}
+
+#[derive(Deserialize)]
+struct FirmaApi {
+    verification_status: String,
+    #[serde(default)]
+    gpg_key_id: Option<u64>,
+    #[serde(default)]
+    key: Option<ClaveApi>,
+}
+
+#[derive(Deserialize)]
+struct CommitApi {
+    id: String,
+}
+
+/// Adaptador de GitLab para un proyecto.
+pub struct GitLab {
+    con: Conexion,
+    /// Id del proyecto ya codificado para la ruta.
+    proyecto: String,
+}
+
+impl std::fmt::Debug for GitLab {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("GitLab")
+            .field("con", &self.con)
+            .field("proyecto", &self.proyecto)
+            .finish_non_exhaustive()
+    }
+}
+
+impl GitLab {
+    /// Adaptador para el proyecto `proyecto` (id numerico o
+    /// `grupo[/subgrupo...]/proyecto`) en la API `base` (con su `/api/v4`).
+    pub fn nuevo(
+        base: Base,
+        proyecto: &str,
+        credenciales: Box<dyn Credenciales>,
+        personas: Personas,
+    ) -> Result<Self, Informe> {
+        let Some(codificado) = codificar_proyecto(proyecto) else {
+            return Err(Informe::nuevo("plataforma", Resultado::Uso)
+                .con("motivo", "proyecto-invalido")
+                .diagnostico(
+                    "el proyecto de GitLab tiene que ser su id numerico o grupo/proyecto",
+                ));
+        };
+        Ok(Self {
+            con: Conexion::nueva(base, Proveedor::GitLab, credenciales, personas),
+            proyecto: codificado,
+        })
+    }
+
+    /// Llamadas hechas, `METODO ruta` (para medir el coste por operacion).
+    pub fn registro(&self) -> Vec<String> {
+        self.con.registro()
+    }
+
+    fn ep(&self, ep: &Endpoint<'_>) -> (Metodo, String) {
+        ep.metodo_y_ruta(&self.proyecto)
+    }
+
+    fn exigir_persona(&self, op: &str) -> Paso<()> {
+        self.con.exigir_persona(op, self.identidad(Rol::Persona))
+    }
+
+    fn mr_abierto(&self, rama: &str, base: &str) -> Paso<Option<u64>> {
+        let (_, ruta) = self.ep(&Endpoint::MrsAbiertos { rama, base });
+        let mrs: Vec<MrApi> =
+            self.con
+                .leer_paginas(ABRIR_PR, Rol::Agente, PERMISO_LECTURA, ruta, |c| {
+                    self.con.parsear(ABRIR_PR, c)
+                })?;
+        Ok(mrs
+            .into_iter()
+            .find(|m| m.source_branch == rama && m.target_branch == base && m.state == "opened")
+            .map(|m| m.iid))
+    }
+
+    /// Si `rama` tiene commits que no estan en `base`.
+    fn tiene_commits(&self, rama: &str, base: &str) -> Paso<bool> {
+        let commits: Vec<CommitApi> = self.con.leer(
+            ABRIR_PR,
+            Rol::Agente,
+            PERMISO_LECTURA,
+            self.ep(&Endpoint::CommitsEntre { base, rama }),
+        )?;
+        Ok(commits.iter().any(|c| !c.id.is_empty()))
+    }
+}
+
+/// Id del proyecto listo para la ruta: el numero tal cual, o los segmentos
+/// validados unidos con `%2F`. `None` si no vale.
+fn codificar_proyecto(p: &str) -> Option<String> {
+    if !p.is_empty() && p.len() <= 20 && p.bytes().all(|b| b.is_ascii_digit()) {
+        return (p.parse::<u64>().ok()? > 0).then(|| p.to_owned());
+    }
+    let segmentos: Vec<&str> = p.split('/').collect();
+    let valido = segmentos.len() >= 2
+        && p.len() <= 255
+        && segmentos.iter().all(|s| {
+            !s.is_empty()
+                && *s != "."
+                && *s != ".."
+                && !s.starts_with(['-', '.'])
+                && !s.ends_with('.')
+                && s.bytes()
+                    .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
+        });
+    valido.then(|| segmentos.join("%2F"))
+}
+
+/// Estado de un estado de commit de GitLab. Solo `success` es verde; un
+/// estado saltado o manual no da la puerta por cumplida (falla cerrado,
+/// como la PoC M).
+fn estado_check(s: &str) -> EstadoCheck {
+    match s {
+        "success" => EstadoCheck::Ok,
+        "failed" | "canceled" => EstadoCheck::Fallo,
+        _ => EstadoCheck::Pendiente,
+    }
+}
+
+impl Plataforma for GitLab {
+    fn abrir_pr(&self, rama: &str, base: &str, titulo: &str, cuerpo: &str) -> Informe {
+        if !rama_valida(rama) || !rama_valida(base) {
+            return uso(ABRIR_PR, "rama-invalida", "nombre de rama no valido");
+        }
+        if titulo.trim().is_empty() || titulo.chars().any(char::is_control) {
+            return uso(
+                ABRIR_PR,
+                "titulo-invalido",
+                "el titulo no puede estar vacio ni llevar controles",
+            );
+        }
+        match self.mr_abierto(rama, base) {
+            Ok(Some(n)) => {
+                return Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string());
+            }
+            Ok(None) => {}
+            Err(i) => return i,
+        }
+        // GitLab abre un MR sin commits: se comprueba antes para dar la
+        // misma linea que GitHub.
+        match self.tiene_commits(rama, base) {
+            Ok(true) => {}
+            Ok(false) => {
+                return Informe::nuevo(ABRIR_PR, Resultado::Rechazada)
+                    .con("motivo", "sin-commits")
+                    .diagnostico(
+                        "la rama no tiene commits nuevos respecto a la base: no hay nada que revisar",
+                    );
+            }
+            Err(i) => return i,
+        }
+        let datos = serde_json::json!({
+            "source_branch": rama, "target_branch": base,
+            "title": titulo, "description": cuerpo,
+        })
+        .to_string();
+        let r = match self.con.pedir(
+            ABRIR_PR,
+            Rol::Agente,
+            self.ep(&Endpoint::CrearMr),
+            Some(&datos),
+        ) {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        if r.estado == 409 {
+            // Otro proceso lo abrio entre la lectura y la escritura.
+            return match self.mr_abierto(rama, base) {
+                Ok(Some(n)) => {
+                    Informe::nuevo(ABRIR_PR, Resultado::SinCambios).con("pr", n.to_string())
+                }
+                Ok(None) => Informe::nuevo(ABRIR_PR, Resultado::Conflicto)
+                    .con("motivo", "mr-existente")
+                    .diagnostico(format!(
+                        "GitLab dice que ya hay un MR para la rama pero no se encuentra abierto: reintenta (GitLab: «{}»)",
+                        errores::mensaje_api(&r)
+                    )),
+                Err(i) => i,
+            };
+        }
+        if !r.exito() {
+            return self
+                .con
+                .traducir(ABRIR_PR, Rol::Agente, PERMISO_ESCRITURA, &r);
+        }
+        match self.con.parsear::<MrApi>(ABRIR_PR, &r.cuerpo) {
+            Ok(m) => Informe::nuevo(ABRIR_PR, Resultado::Ok).con("pr", m.iid.to_string()),
+            Err(i) => i,
+        }
+    }
+
+    fn estado_puertas(&self, rol: Rol, pr: u64, modelo: &ModeloPuertas) -> Informe {
+        let op = puertas::ESTADO_PUERTAS;
+        if rol == Rol::Revisor {
+            return rol_no_valido(op, rol);
+        }
+        if rol == Rol::Persona
+            && let Err(i) = self.exigir_persona(op)
+        {
+            return i;
+        }
+        let m: MrPuertasApi = match self.con.leer(
+            op,
+            rol,
+            PERMISO_LECTURA,
+            self.ep(&Endpoint::MrConDivergencia(pr)),
+        ) {
+            Ok(m) => m,
+            Err(i) => return i,
+        };
+        if m.state == "merged" {
+            return puertas::mergeado(pr);
+        }
+        if m.state != "opened" {
+            return puertas::pr_cerrado(pr);
+        }
+        if !rama_valida(&m.target_branch) || !sha_valido(&m.sha) {
+            return self.con.invalida(op, "rama destino o sha del MR");
+        }
+        // Las reglas de aprobacion son de Premium. Observado en el E2E: en
+        // Free el endpoint responde 200 con la lista vacia (no el 403 que
+        // suponia F1-13), asi que lo que se comprueba es que haya una regla
+        // que pida al menos una aprobacion; el 403 sigue contando como plan.
+        let r = match self
+            .con
+            .pedir(op, rol, self.ep(&Endpoint::ReglasAprobacion), None)
+        {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        let impone_aprobacion = if r.exito() {
+            match self.con.parsear::<Vec<ReglaAprobacionApi>>(op, &r.cuerpo) {
+                Ok(v) => v.iter().any(|x| x.approvals_required >= 1),
+                Err(i) => return i,
+            }
+        } else if r.estado == 403 {
+            false
+        } else {
+            return self.con.traducir(op, rol, PERMISO_LECTURA, &r);
+        };
+        if !impone_aprobacion
+            && let Some(g) = modelo
+                .exigir_impuestas
+                .iter()
+                .find(|g| GARANTIAS_PREMIUM.contains(&g.as_str()))
+        {
+            return if r.estado == 403 {
+                puertas::rechazada_por_plan(pr, g)
+            } else {
+                puertas::rechazada_por_config(
+                    pr,
+                    g,
+                    "GitLab no impone aprobaciones en este proyecto: crea una regla de aprobacion con al menos una aprobacion (requiere Premium o Ultimate) o quita la garantia de la politica",
+                )
+            };
+        }
+        // G01: la rama destino solo la mergean Maintainers y no admite
+        // force-push.
+        let r = match self.con.pedir(
+            op,
+            rol,
+            self.ep(&Endpoint::RamaProtegida(&m.target_branch)),
+            None,
+        ) {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        let protegida = match r.estado {
+            404 => None,
+            _ if r.exito() => match self.con.parsear::<RamaProtegidaApi>(op, &r.cuerpo) {
+                Ok(p) => Some(p),
+                Err(i) => return i,
+            },
+            _ => return self.con.traducir(op, rol, PERMISO_LECTURA, &r),
+        };
+        let g01 = protegida.is_some_and(|p| {
+            !p.allow_force_push
+                && p.merge_access_levels.len() == 1
+                && p.merge_access_levels
+                    .iter()
+                    .all(|n| n.access_level == MAINTAINER)
+        });
+        if !g01 {
+            return puertas::rechazada_por_config(
+                pr,
+                "G01",
+                "la rama destino no esta protegida con merge solo para Maintainers y sin force-push: protegela (taskctl setup lo propone)",
+            );
+        }
+        let (_, ruta) = self.ep(&Endpoint::Estados(&m.sha));
+        let mut estados: Vec<EstadoApi> =
+            match self
+                .con
+                .leer_paginas(op, rol, PERMISO_LECTURA, ruta, |c| self.con.parsear(op, c))
+            {
+                Ok(v) => v,
+                Err(i) => return i,
+            };
+        // De cada nombre cuenta el ultimo estado (por id) que publico su
+        // origen: `author.id` es el sustituto de G09 (F1-13, decision 6).
+        estados.sort_by_key(|e| e.id);
+        let checks = modelo
+            .checks
+            .iter()
+            .map(|(nombre, origen)| {
+                let estado = estados
+                    .iter()
+                    .rev()
+                    .find(|e| &e.name == nombre && e.author.id == *origen)
+                    .map_or(EstadoCheck::Pendiente, |e| estado_check(&e.status));
+                (nombre.clone(), estado)
+            })
+            .collect();
+        let a: AprobacionesApi = match self.con.leer(
+            op,
+            rol,
+            PERMISO_LECTURA,
+            self.ep(&Endpoint::Aprobaciones(pr)),
+        ) {
+            Ok(a) => a,
+            Err(i) => return i,
+        };
+        // Una aprobacion por cuenta, por id (F1-6, regla 6). La del autor del
+        // MR no cuenta: GitLab Free la admite (observado en el E2E) y GitHub
+        // no, y la linea tiene que ser la misma (RR-15).
+        let aprobadores: BTreeMap<u64, Usuario> = a
+            .approved_by
+            .into_iter()
+            .map(|x| (x.user.id, Usuario::from(x.user)))
+            .collect();
+        let aprobaciones = aprobadores
+            .values()
+            .filter(|u| u.id != m.author.id && u.es_persona(self.con.personas()))
+            .count();
+        puertas::evaluar(&Lecturas {
+            pr,
+            checks,
+            aprobaciones,
+            ignoradas: aprobadores.len() - aprobaciones,
+            al_dia: m.diverged_commits_count == 0,
+            sin_conflictos: !m.has_conflicts,
+        })
+    }
+
+    fn merge_pr(&self, pr: u64, sha: &str) -> Informe {
+        if !sha_valido(sha) {
+            return uso(
+                MERGE_PR,
+                "sha-invalido",
+                "el sha esperado tiene que ser completo (40 hex)",
+            );
+        }
+        if let Err(i) = self.exigir_persona(MERGE_PR) {
+            return i;
+        }
+        let m: MrApi = match self.con.leer(
+            MERGE_PR,
+            Rol::Persona,
+            PERMISO_MERGE,
+            self.ep(&Endpoint::Mr(pr)),
+        ) {
+            Ok(m) => m,
+            Err(i) => return i,
+        };
+        if m.state == "merged" {
+            return Informe::nuevo(MERGE_PR, Resultado::SinCambios)
+                .con("pr", pr.to_string())
+                .con("estado", "mergeado");
+        }
+        if m.state != "opened" {
+            return Informe::nuevo(MERGE_PR, Resultado::Rechazada)
+                .con("pr", pr.to_string())
+                .con("motivo", "pr-cerrado")
+                .diagnostico("el MR esta cerrado sin mergear");
+        }
+        if !m
+            .sha
+            .as_deref()
+            .is_some_and(|s| s.eq_ignore_ascii_case(sha))
+        {
+            return head_cambiado(pr);
+        }
+        let datos = serde_json::json!({
+            "sha": sha, "squash": false, "should_remove_source_branch": false,
+        })
+        .to_string();
+        let r = match self.con.pedir(
+            MERGE_PR,
+            Rol::Persona,
+            self.ep(&Endpoint::Merge(pr)),
+            Some(&datos),
+        ) {
+            Ok(r) => r,
+            Err(i) => return i,
+        };
+        if !r.exito() {
+            let mut i = self.con.traducir(MERGE_PR, Rol::Persona, PERMISO_MERGE, &r);
+            i.claves
+                .insert(0, ("pr".to_owned(), Valor::Texto(pr.to_string())));
+            return i;
+        }
+        match self.con.parsear::<MergeApi>(MERGE_PR, &r.cuerpo) {
+            Ok(MergeApi {
+                state,
+                merge_commit_sha: Some(s),
+            }) if state == "merged" && sha_valido(&s) => Informe::nuevo(MERGE_PR, Resultado::Ok)
+                .con("pr", pr.to_string())
+                .con("commit", corto(&s)),
+            // Mergeado aunque la respuesta no traiga un sha legible.
+            Ok(m) if m.state == "merged" => {
+                Informe::nuevo(MERGE_PR, Resultado::Ok).con("pr", pr.to_string())
+            }
+            Ok(_) => self.con.invalida(MERGE_PR, "state"),
+            Err(i) => i,
+        }
+    }
+
+    fn leer_puertas(&self, rol: Rol, rama: &str) -> Paso<ReglasLeidas> {
+        if !rama_valida(rama) {
+            return Err(uso(
+                LEER_PUERTAS,
+                "rama-invalida",
+                "nombre de rama no valido",
+            ));
+        }
+        if rol == Rol::Revisor {
+            return Err(rol_no_valido(LEER_PUERTAS, rol));
+        }
+        let (_, ruta) = self.ep(&Endpoint::RamasProtegidas);
+        let ramas: Vec<RamaProtegidaApi> =
+            self.con
+                .leer_paginas(LEER_PUERTAS, rol, PERMISO_LECTURA, ruta, |c| {
+                    self.con.parsear(LEER_PUERTAS, c)
+                })?;
+        let mut reglas_rama = Vec::new();
+        if let Some(p) = ramas.iter().find(|p| p.name == rama) {
+            reglas_rama.extend(
+                p.push_access_levels
+                    .iter()
+                    .map(|n| format!("push={}", n.access_level)),
+            );
+            reglas_rama.extend(
+                p.merge_access_levels
+                    .iter()
+                    .map(|n| format!("merge={}", n.access_level)),
+            );
+            if p.allow_force_push {
+                reglas_rama.push("force-push".to_owned());
+            }
+            if p.code_owner_approval_required {
+                reglas_rama.push("code-owner".to_owned());
+            }
+        }
+        let r = self.con.pedir(
+            LEER_PUERTAS,
+            rol,
+            self.ep(&Endpoint::ReglasAprobacion),
+            None,
+        )?;
+        if r.estado == 403 {
+            return Ok(ReglasLeidas {
+                plan_da_reglas: false,
+                rulesets: Vec::new(),
+                reglas_rama,
+            });
+        }
+        if !r.exito() {
+            return Err(self.con.traducir(LEER_PUERTAS, rol, PERMISO_LECTURA, &r));
+        }
+        let reglas: Vec<ReglaAprobacionApi> = self.con.parsear(LEER_PUERTAS, &r.cuerpo)?;
+        Ok(ReglasLeidas {
+            plan_da_reglas: true,
+            rulesets: reglas.into_iter().map(|r| r.name).collect(),
+            reglas_rama,
+        })
+    }
+
+    fn leer_permisos_ci(&self) -> Informe {
+        if let Err(i) = self.exigir_persona(LEER_PERMISOS_CI) {
+            return i;
+        }
+        // El job token de GitLab no puede aprobar MR: es fijo de la
+        // plataforma, no un ajuste del proyecto (G16, tabla de F1-13).
+        Informe::nuevo(LEER_PERMISOS_CI, Resultado::Ok)
+            .con("ci-aprueba", "false")
+            .con("fuente", "plataforma")
+    }
+
+    fn firma_commit(&self, sha: &str) -> Paso<Firma> {
+        if !sha_valido(sha) {
+            return Err(uso(
+                FIRMA_COMMIT,
+                "sha-invalido",
+                "el sha tiene que ser completo (40 hex)",
+            ));
+        }
+        let r = self.con.pedir(
+            FIRMA_COMMIT,
+            Rol::Agente,
+            self.ep(&Endpoint::Firma(sha)),
+            None,
+        )?;
+        if r.estado == 404 {
+            // Sin firma o sin commit: se distingue leyendo el commit.
+            let _: CommitApi = self.con.leer(
+                FIRMA_COMMIT,
+                Rol::Agente,
+                PERMISO_LECTURA,
+                self.ep(&Endpoint::Commit(sha)),
+            )?;
+            return Ok(Firma {
+                verificada: false,
+                motivo: "unsigned".to_owned(),
+                committer: None,
+                clave: None,
+            });
+        }
+        if !r.exito() {
+            return Err(self
+                .con
+                .traducir(FIRMA_COMMIT, Rol::Agente, PERMISO_LECTURA, &r));
+        }
+        let f: FirmaApi = self.con.parsear(FIRMA_COMMIT, &r.cuerpo)?;
+        // `verified_system` es la firma de la propia GitLab (commits web):
+        // no es de una persona.
+        Ok(Firma {
+            verificada: f.verification_status == "verified",
+            motivo: errores::sanear(&f.verification_status),
+            committer: None,
+            clave: f.key.map(|k| k.id).or(f.gpg_key_id),
+        })
+    }
+
+    fn identidad(&self, rol: Rol) -> Paso<Usuario> {
+        self.con.identidad(rol, || {
+            self.con
+                .leer::<UsuarioApi>(IDENTIDAD, rol, PERMISO_LECTURA, self.ep(&Endpoint::Usuario))
+                .map(Usuario::from)
+        })
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn proyecto_codificado_una_vez() {
+        assert_eq!(codificar_proyecto("7").as_deref(), Some("7"));
+        assert_eq!(codificar_proyecto("g/p").as_deref(), Some("g%2Fp"));
+        assert_eq!(
+            codificar_proyecto("g/s.b/p-1_x").as_deref(),
+            Some("g%2Fs.b%2Fp-1_x")
+        );
+        for p in [
+            "", "0", "p", "g/", "/p", "g//p", "g/..", "g/./p", "g/p?x", "g/p%2F", "g/-p", "g/.p",
+            "g/p.", "g p/x",
+        ] {
+            assert_eq!(codificar_proyecto(p), None, "{p}");
+        }
+    }
+
+    #[test]
+    fn solo_success_es_verde() {
+        assert_eq!(estado_check("success"), EstadoCheck::Ok);
+        assert_eq!(estado_check("failed"), EstadoCheck::Fallo);
+        assert_eq!(estado_check("canceled"), EstadoCheck::Fallo);
+        for s in ["pending", "running", "skipped", "manual", "created", "otro"] {
+            assert_eq!(estado_check(s), EstadoCheck::Pendiente, "{s}");
+        }
+    }
+}
diff --git a/crates/tc2/src/plataforma/http.rs b/crates/tc2/src/plataforma/http.rs
index db63d5c..3f9f1d9 100644
--- a/crates/tc2/src/plataforma/http.rs
+++ b/crates/tc2/src/plataforma/http.rs
@@ -4,11 +4,14 @@
 //! Toda llamada a la API sale por [`Cliente::llamar`], que comprueba antes de
 //! abrir la red:
 //! - el metodo: solo los de [`Metodo`] (lista cerrada por tipo);
-//! - el destino: la ruta es relativa a una [`Base`] fija (la API de GitHub,
-//!   la que diga la configuracion o, solo para tests, `http://127.0.0.1`);
+//! - el destino: la ruta es relativa a una [`Base`] fija (la API de GitHub o
+//!   GitLab, la que diga la configuracion o, solo para tests,
+//!   `http://127.0.0.1`);
 //! - la ruta: empieza por `/`, sin segmentos `.` ni `..`, sin `\`, sin
 //!   `%2e` ni `%25` (tampoco codificados dos veces) y sin caracteres fuera
-//!   de los no reservados.
+//!   de los no reservados. Con GitLab admite ademas `%2F` (el id de un
+//!   proyecto con nombre), y los segmentos se comprueban con la barra ya
+//!   decodificada.
 //!
 //! No sigue redirecciones, limita el tamano de la respuesta y los tiempos de
 //! espera, y lleva un registro `METODO ruta` sin cabeceras. El token nunca
@@ -17,6 +20,7 @@
 use std::cell::RefCell;
 use std::time::Duration;
 
+use super::Proveedor;
 use super::credenciales::Secreto;
 
 /// Tamano maximo del cuerpo de una respuesta.
@@ -71,8 +75,15 @@ impl Base {
         let resto = if let Some(r) = url.strip_prefix("https://") {
             r
         } else if let Some(r) = url.strip_prefix("http://127.0.0.1:") {
-            if !puerto_valido(r) {
-                return Err("http solo vale para 127.0.0.1 con un puerto y sin ruta".into());
+            let (puerto, ruta) = r.split_once('/').unwrap_or((r, ""));
+            let valida = puerto_valido(puerto)
+                && !ruta.contains(['?', '#'])
+                && (ruta.is_empty() || validar_ruta(&format!("/{ruta}")).is_ok());
+            if !valida {
+                return Err(
+                    "http solo vale para 127.0.0.1 con un puerto y, si acaso, un prefijo valido"
+                        .into(),
+                );
             }
             return Ok(Self(url.to_owned()));
         } else {
@@ -124,11 +135,12 @@ pub struct Respuesta {
     pub cuerpo: String,
     /// `Link` (paginacion).
     pub link: Option<String>,
-    /// `x-ratelimit-remaining`.
+    /// `x-ratelimit-remaining` (GitHub) o `ratelimit-remaining` (GitLab).
     pub restantes: Option<String>,
     /// `retry-after`, en segundos.
     pub reintentar: Option<String>,
-    /// `x-ratelimit-reset`, en segundos desde 1970.
+    /// `x-ratelimit-reset` (GitHub) o `ratelimit-reset` (GitLab), en
+    /// segundos desde 1970.
     pub reinicio: Option<String>,
 }
 
@@ -167,22 +179,32 @@ pub enum FalloHttp {
     Otro(String),
 }
 
-/// Cliente HTTP con su base fija.
+/// Cliente HTTP con su base fija y la plataforma a la que habla.
 pub struct Cliente {
     base: Base,
+    proveedor: Proveedor,
     agente: ureq::Agent,
     registro: RefCell<Vec<String>>,
 }
 
 impl std::fmt::Debug for Cliente {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
-        f.debug_struct("Cliente").field("base", &self.base).finish()
+        f.debug_struct("Cliente")
+            .field("base", &self.base)
+            .field("proveedor", &self.proveedor)
+            .finish()
     }
 }
 
 impl Cliente {
-    /// Cliente para una base ya validada.
+    /// Cliente de GitHub para una base ya validada.
     pub fn nuevo(base: Base) -> Self {
+        Self::para(base, Proveedor::GitHub)
+    }
+
+    /// Cliente para una base ya validada y su plataforma (que decide las
+    /// cabeceras y si la ruta admite `%2F`).
+    pub fn para(base: Base, proveedor: Proveedor) -> Self {
         let tls = ureq::tls::TlsConfig::builder()
             .root_certs(ureq::tls::RootCerts::PlatformVerifier)
             .build();
@@ -199,6 +221,7 @@ impl Cliente {
             .into();
         Self {
             base,
+            proveedor,
             agente,
             registro: RefCell::new(Vec::new()),
         }
@@ -231,7 +254,8 @@ impl Cliente {
         token: Option<&Secreto>,
         cuerpo: Option<&str>,
     ) -> Result<Respuesta, FalloHttp> {
-        validar_ruta(ruta).map_err(FalloHttp::Rechazada)?;
+        validar_ruta_con(ruta, self.proveedor == Proveedor::GitLab)
+            .map_err(FalloHttp::Rechazada)?;
         self.registro
             .borrow_mut()
             .push(format!("{} {}", metodo.nombre(), ruta));
@@ -259,9 +283,10 @@ impl Cliente {
                 .map(str::to_owned)
         };
         let link = cabecera("link");
-        let restantes = cabecera("x-ratelimit-remaining");
+        let restantes =
+            cabecera("x-ratelimit-remaining").or_else(|| cabecera("ratelimit-remaining"));
         let reintentar = cabecera("retry-after");
-        let reinicio = cabecera("x-ratelimit-reset");
+        let reinicio = cabecera("x-ratelimit-reset").or_else(|| cabecera("ratelimit-reset"));
         // Si la respuesta declara su tamano y pasa del limite, no se lee:
         // cortar la lectura a medias deja la conexion en un estado que el
         // sistema puede notificar como desconexion (y se veria `sinred`).
@@ -293,9 +318,12 @@ impl Cliente {
         p: ureq::RequestBuilder<B>,
         token: Option<&Secreto>,
     ) -> ureq::RequestBuilder<B> {
-        let p = p
-            .header("Accept", "application/vnd.github+json")
-            .header("X-GitHub-Api-Version", "2022-11-28");
+        let p = match self.proveedor {
+            Proveedor::GitHub => p
+                .header("Accept", "application/vnd.github+json")
+                .header("X-GitHub-Api-Version", "2022-11-28"),
+            Proveedor::GitLab => p.header("Accept", "application/json"),
+        };
         match token {
             Some(t) => p.header("Authorization", format!("Bearer {}", t.exponer())),
             None => p,
@@ -330,8 +358,15 @@ fn tipo_error(e: &ureq::Error) -> String {
     t.chars().filter(|c| !c.is_control()).take(200).collect()
 }
 
-/// Comprueba la ruta (con su consulta) antes de la red.
+/// Comprueba la ruta (con su consulta) antes de la red, sin admitir `%2F`.
 pub fn validar_ruta(ruta: &str) -> Result<(), String> {
+    validar_ruta_con(ruta, false)
+}
+
+/// Comprueba la ruta. Con `admite_barra_codificada` (GitLab) acepta la
+/// secuencia exacta `%2F` en el camino y comprueba los segmentos con ella ya
+/// decodificada: `a%2F..` es un `..` y `%2f` o `%252F` siguen rechazados.
+pub fn validar_ruta_con(ruta: &str, admite_barra_codificada: bool) -> Result<(), String> {
     let (camino, consulta) = ruta.split_once('?').unwrap_or((ruta, ""));
     if !camino.starts_with('/') || camino.starts_with("//") {
         return Err("la ruta tiene que ser relativa a la API".into());
@@ -340,6 +375,13 @@ pub fn validar_ruta(ruta: &str) -> Result<(), String> {
     if ruta.contains('\\') || minus.contains("%2e") || minus.contains("%25") {
         return Err("ruta con barra invertida o puntos codificados".into());
     }
+    let decodificado;
+    let camino = if admite_barra_codificada {
+        decodificado = camino.replace("%2F", "/");
+        decodificado.as_str()
+    } else {
+        camino
+    };
     if camino[1..]
         .split('/')
         .any(|s| s.is_empty() || s == "." || s == "..")
@@ -393,6 +435,24 @@ mod tests {
         ] {
             assert!(validar_ruta(r).is_err(), "{r}");
         }
+        for r in [
+            "/projects/g%2Fp",
+            "/projects/g%2Fs%2Fp/merge_requests?source_branch=a%2Fb",
+        ] {
+            assert!(validar_ruta_con(r, true).is_ok(), "{r}");
+        }
+        for r in [
+            "/projects/g%2fp",
+            "/projects/g%252Fp",
+            "/projects/g%2F..",
+            "/projects/g%2F.%2Fp",
+            "/projects/%2Fp",
+            "/projects/g%2F",
+            "/projects/g%2F%2Fp",
+            "/projects/g%3Fp",
+        ] {
+            assert!(validar_ruta_con(r, true).is_err(), "{r}");
+        }
         for r in [
             "/user",
             "/repos/o/r/pulls?head=o%3Afeature%2Fx&state=open",
@@ -407,9 +467,12 @@ mod tests {
         assert!(Base::nueva("https://api.github.com").is_ok());
         assert!(Base::nueva("https://git.empresa.com/api/v3/").is_ok());
         assert!(Base::nueva("http://127.0.0.1:8080").is_ok());
+        assert!(Base::nueva("http://127.0.0.1:8080/api/v4").is_ok());
         for b in [
             "http://api.github.com",
-            "http://127.0.0.1:80/x",
+            "http://127.0.0.1:80/../x",
+            "http://127.0.0.1:80/x?y",
+            "http://127.0.0.1:80/a%2Fb",
             "http://localhost:80",
             "https://user:pw@host",
             "https://",
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index ea8cbf0..5a136ed 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -1,27 +1,31 @@
-//! Puerto `Plataforma` (ADR F1-4 decision 1, F1-13 D1a) y su adaptador de
-//! GitHub (F2-6).
+//! Puerto `Plataforma` (ADR F1-4 decision 1, F1-13 D1a) y sus adaptadores
+//! de GitHub (F2-6) y GitLab (F2-9).
 //!
 //! El puerto es la unica parte del CLI que habla con la plataforma, por su
 //! API HTTP y sin `gh` (F1-14). Sus operaciones forman una lista cerrada y
 //! devuelven el contrato de salida de [`crate::contrato`]; cada una corre con
-//! la identidad que le da F1-6 y falla cerrada sin ella. GitLab (F2-9)
-//! implementa el mismo trait y pasa la misma bateria.
+//! la identidad que le da F1-6 y falla cerrada sin ella. Los dos adaptadores
+//! comparten el transporte ([`conexion`]) y la evaluacion ([`puertas`]), y
+//! dan la misma linea byte a byte (RR-15).
 //!
 //! Se usa desde la biblioteca: el binario no gana subcomandos hasta que los
 //! comandos de fase (F6-2 `review`, F7 `finish`, F3-8 `doctor`) lo llamen.
 
+pub mod conexion;
 pub mod credenciales;
 pub mod deteccion;
 pub mod errores;
 pub mod github;
+pub mod gitlab;
 pub mod http;
 pub mod puertas;
 
 use std::collections::BTreeSet;
 
 pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
-pub use deteccion::{Destino, Proveedor, detectar};
+pub use deteccion::{Destino, detectar};
 pub use github::GitHub;
+pub use gitlab::GitLab;
 pub use puertas::{EstadoCheck, Lecturas, evaluar};
 
 use crate::contrato::Informe;
@@ -30,20 +34,68 @@ use crate::contrato::Informe;
 /// devuelve.
 pub type Paso<T> = Result<T, Informe>;
 
-/// Lista de personas por id numerico (F1-6, decision 3). La lee F2-13 del
-/// fichero de `.taskcode/`; aqui llega ya leida.
+/// Plataforma soportada.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Proveedor {
+    /// GitHub.
+    GitHub,
+    /// GitLab.
+    GitLab,
+}
+
+impl Proveedor {
+    /// Nombre en la configuracion y en la linea.
+    pub fn nombre(self) -> &'static str {
+        match self {
+            Self::GitHub => "github",
+            Self::GitLab => "gitlab",
+        }
+    }
+
+    /// Nombre para la persona, en los diagnosticos.
+    pub fn titulo(self) -> &'static str {
+        match self {
+            Self::GitHub => "GitHub",
+            Self::GitLab => "GitLab",
+        }
+    }
+}
+
+/// Lista de personas por id numerico (F1-6, decision 3) y, para GitLab, los
+/// ids de sus claves de firma (F1-13, decision 6: la firma de GitLab no trae
+/// el usuario). La lee F2-13 del fichero de `.taskcode/`; aqui llega ya
+/// leida.
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
-pub struct Personas(BTreeSet<u64>);
+pub struct Personas {
+    ids: BTreeSet<u64>,
+    claves: BTreeSet<u64>,
+}
 
 impl Personas {
     /// Lista con estos ids.
     pub fn nuevas(ids: impl IntoIterator<Item = u64>) -> Self {
-        Self(ids.into_iter().collect())
+        Self {
+            ids: ids.into_iter().collect(),
+            claves: BTreeSet::new(),
+        }
+    }
+
+    /// La misma lista con estas claves de firma (ids de clave GPG o SSH de
+    /// la plataforma).
+    #[must_use]
+    pub fn con_claves(mut self, claves: impl IntoIterator<Item = u64>) -> Self {
+        self.claves = claves.into_iter().collect();
+        self
     }
 
     /// Si el id es de una persona de la lista.
     pub fn contiene(&self, id: u64) -> bool {
-        self.0.contains(&id)
+        self.ids.contains(&id)
+    }
+
+    /// Si la clave de firma es de una persona de la lista.
+    pub fn contiene_clave(&self, clave: u64) -> bool {
+        self.claves.contains(&clave)
     }
 }
 
@@ -54,7 +106,7 @@ pub struct Usuario {
     pub login: String,
     /// Id numerico (el que se compara, no el login).
     pub id: u64,
-    /// `User` o `Bot` (GitHub).
+    /// `User` o `Bot` (en GitLab, `Bot` si la cuenta es `bot: true`).
     pub tipo: String,
 }
 
@@ -70,8 +122,9 @@ impl Usuario {
 /// impuestas por la plataforma (F1-13, decision 5).
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct ModeloPuertas {
-    /// `(nombre, integration_id)`: un check solo cuenta si lo publico esa
-    /// App (G09).
+    /// `(nombre, origen)`: un check solo cuenta si lo publico su origen
+    /// (G09). En GitHub el origen es el `integration_id` de la App; en
+    /// GitLab, el `author.id` del estado del commit (F1-13, decision 6).
     pub checks: Vec<(String, u64)>,
     /// Ids de garantia (`G03`...) que tienen que estar impuestas.
     pub exigir_impuestas: Vec<String>,
@@ -80,24 +133,27 @@ pub struct ModeloPuertas {
 /// Firma de un commit (contrato del firmante, F1-6 decision 3).
 #[derive(Debug, Clone, PartialEq, Eq)]
 pub struct Firma {
-    /// `commit.verification.verified`.
+    /// Firma verificada por la plataforma.
     pub verificada: bool,
-    /// `commit.verification.reason`.
+    /// Motivo o estado que da la plataforma.
     pub motivo: String,
-    /// La cuenta `committer`, si GitHub la asocia.
+    /// La cuenta `committer`, si la plataforma la asocia (GitHub).
     pub committer: Option<Usuario>,
+    /// Id de la clave que firmo, si la plataforma lo da (GitLab).
+    pub clave: Option<u64>,
 }
 
 impl Firma {
     /// Si el commit cuenta como aprobacion de una persona: firma verificada
-    /// y committer persona de la lista (reglas 1 a 4; la 5, `Plan-Blob`, es
-    /// de quien llama).
+    /// y committer persona de la lista o clave de una persona de la lista
+    /// (reglas 1 a 4; la 5, `Plan-Blob`, es de quien llama).
     pub fn cuenta_como_aprobacion(&self, personas: &Personas) -> bool {
         self.verificada
-            && self
+            && (self
                 .committer
                 .as_ref()
                 .is_some_and(|c| c.es_persona(personas))
+                || self.clave.is_some_and(|k| personas.contiene_clave(k)))
     }
 }
 
@@ -105,11 +161,18 @@ impl Firma {
 /// el modelo es de F3-8.
 #[derive(Debug, Clone, Default, PartialEq, Eq)]
 pub struct ReglasLeidas {
-    /// `false` si el plan no da rulesets (403 con aviso de plan).
+    /// `false` si el plan no da las reglas (GitHub: 403 con aviso de plan
+    /// en los rulesets; GitLab: 403 en las reglas de aprobacion). En GitLab
+    /// Free el endpoint da 200 con la lista vacia (observado en el E2E de
+    /// F2-9), asi que `true` no prueba que el plan sea Premium: lo afina
+    /// F3-8.
     pub plan_da_reglas: bool,
-    /// Nombres de los rulesets del repo.
+    /// Nombres de los rulesets (GitHub) o de las reglas de aprobacion
+    /// (GitLab).
     pub rulesets: Vec<String>,
-    /// Tipos de regla que aplican a la rama.
+    /// Reglas que aplican a la rama: tipos de regla (GitHub) o la
+    /// proteccion de la rama (GitLab: `push=N`, `merge=N`, `force-push`,
+    /// `code-owner`).
     pub reglas_rama: Vec<String>,
 }
 
diff --git a/crates/tc2/src/plataforma/puertas.rs b/crates/tc2/src/plataforma/puertas.rs
index 2ac31f3..dc759c2 100644
--- a/crates/tc2/src/plataforma/puertas.rs
+++ b/crates/tc2/src/plataforma/puertas.rs
@@ -96,6 +96,14 @@ pub fn mergeado(pr: u64) -> Informe {
         .con("estado", "mergeado")
 }
 
+/// PR cerrado sin mergear.
+pub fn pr_cerrado(pr: u64) -> Informe {
+    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
+        .con("pr", pr.to_string())
+        .con("motivo", "pr-cerrado")
+        .diagnostico("el PR esta cerrado sin mergear: reabrelo o abre otro")
+}
+
 /// La politica exige impuesta una garantia que el plan no da.
 pub fn rechazada_por_plan(pr: u64, garantia: &str) -> Informe {
     Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
@@ -107,6 +115,17 @@ pub fn rechazada_por_plan(pr: u64, garantia: &str) -> Informe {
         ))
 }
 
+/// La plataforma no tiene configurada una garantia que se lee de su
+/// configuracion (GitLab: G01, rama destino protegida; G03 y demas de
+/// Premium, una regla de aprobacion). `arreglo` dice que hacer.
+pub fn rechazada_por_config(pr: u64, garantia: &str, arreglo: &str) -> Informe {
+    Informe::nuevo(ESTADO_PUERTAS, Resultado::Rechazada)
+        .con("pr", pr.to_string())
+        .con("motivo", "config")
+        .con("garantia", garantia)
+        .diagnostico(format!("{garantia} no se cumple: {arreglo}"))
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
tareas/02-en-curso/TASK-022/evidencia/e2e-gitlab.log                                      | 40 ++++++++++++++++++++++++++++++++++++++++
 tareas/{01-en-diseno => 02-en-curso}/TASK-022/planificacion/brainstorm/peticion-plan-1.md |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-022/planificacion/plan-final.md                 |  0
 tareas/{01-en-diseno => 02-en-curso}/TASK-022/tarea.md                                    |  8 +++++++-
 4 files changed, 47 insertions(+), 1 deletion(-)
````

## Tests del diff (3 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/comun/http_mock.rs" ":(literal)crates/tc2/tests/plataforma_gitlab.rs" ":(literal)crates/tc2/tests/plataforma_gitlab_e2e.rs"
````

````
crates/tc2/tests/comun/http_mock.rs       |   6 ++
 crates/tc2/tests/plataforma_gitlab.rs     | 805 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/plataforma_gitlab_e2e.rs | 494 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 1305 insertions(+)
````
