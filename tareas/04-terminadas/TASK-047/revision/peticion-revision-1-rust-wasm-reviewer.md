# Peticion de revision — TASK-047 (ronda 1)

- Tarea: TASK-047 — F6-1 Identidades: agente implementador
- Rama revisada: feature/task-047-f6-1-identidades-agente-implementador
- Rama base: develop
- Commit revisado (HEAD): 8123fa68e74571def0591f22da57cfa2536e687c
- Fecha: 2026-11-19
- Agente a lanzar: general-purpose (modelo sugerido: sonnet)
- Skill revisora a cargar: rust-wasm-reviewer
- Carpeta de la tarea: tareas/03-en-revision/TASK-047 (criterios de aceptacion y plan)
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

- tareas/03-en-revision/TASK-047/tarea.md
- tareas/03-en-revision/TASK-047/planificacion/
- crates/tc2/Cargo.toml
- crates/tc2/src/git/puerto.rs
- crates/tc2/src/i18n/catalogo/plataforma.rs
- crates/tc2/src/identidad.rs
- crates/tc2/src/init/inventario.rs
- crates/tc2/src/init/mod.rs
- crates/tc2/src/lib.rs
- crates/tc2/src/plataforma/app_github.rs
- crates/tc2/src/plataforma/conexion.rs
- crates/tc2/src/plataforma/credenciales.rs
- crates/tc2/src/plataforma/entorno_git.rs
- crates/tc2/src/plataforma/github.rs
- crates/tc2/src/plataforma/mod.rs
- crates/tc2/tests/agente_git/mod.rs
- crates/tc2/tests/identidades.rs
- crates/tc2/tests/plataforma_e2e.rs
- crates/tc2/tests/plataforma_gitlab_e2e.rs

## Commits a revisar (git log develop..HEAD)

````
8123fa6 docs(TASK-047): adenda F6-1 en el ADR F1-6 y fila F6-1 del plan
a04e030 feat(TASK-047): identidad propia del agente como GitHub App
6e42d6e chore(TASK-047): tarea en curso
````

## Diff de tu dominio (17 fichero(s) de 19; git diff develop..HEAD -- <tus ficheros>)

````diff
diff --git a/crates/tc2/Cargo.toml b/crates/tc2/Cargo.toml
index 1069e1d..3de43e0 100644
--- a/crates/tc2/Cargo.toml
+++ b/crates/tc2/Cargo.toml
@@ -30,6 +30,12 @@ ureq = { version = "3.5", default-features = false, features = ["rustls", "platf
 # F2-12: tabla de targets, sha256 y SHA256SUMS compartidos con xtask
 # (`tc2 version --json` y `tc2 actualizar --cli`).
 distribucion = { path = "../distribucion" }
+# F6-1: JWT RS256 de la GitHub App del agente (y del revisor) y la cabecera
+# basica de git. Ya estaban en el arbol (ring por rustls, base64 por ureq):
+# se declaran directas en la misma version de Cargo.lock, sin crates nuevos
+# (alternativa descartada: `jsonwebtoken`, que trae su propia cripto).
+ring = "0.17.14"
+base64 = "0.23.1"
 
 # Parser YAML de otra base de codigo (port de libyaml, como el PyYAML del PoC
 # de F1-9) para la ida y vuelta de los tests: con el de produccion seria
diff --git a/crates/tc2/src/git/puerto.rs b/crates/tc2/src/git/puerto.rs
index 4907de8..3972606 100644
--- a/crates/tc2/src/git/puerto.rs
+++ b/crates/tc2/src/git/puerto.rs
@@ -253,6 +253,16 @@ impl Git {
         self
     }
 
+    /// Anade variables a las que ya tenga el puerto, sin quitarlas: el
+    /// entorno del agente de F6-1 (`plataforma::entorno_git`: cabecera de
+    /// autenticacion por `GIT_CONFIG_*` y autor), que asi nunca pasa por la
+    /// linea de ordenes ni por el registro.
+    #[must_use]
+    pub fn con_entorno_extra(mut self, vars: Vec<(OsString, OsString)>) -> Self {
+        self.entorno_extra.extend(vars);
+        self
+    }
+
     /// Copia del registro: una linea `git <subcomando> <args>` por llamada
     /// ejecutada.
     pub fn registro(&self) -> Vec<String> {
diff --git a/crates/tc2/src/i18n/catalogo/plataforma.rs b/crates/tc2/src/i18n/catalogo/plataforma.rs
index 142aa83..6c32039 100644
--- a/crates/tc2/src/i18n/catalogo/plataforma.rs
+++ b/crates/tc2/src/i18n/catalogo/plataforma.rs
@@ -102,6 +102,10 @@ crate::i18n::textos! {
         es: "esta operacion corre con la identidad de {rol}: define {variable}{extra}",
         en: "this operation runs with the {rol} identity: set {variable}{extra}",
     }
+    APP_SIN_TOKEN {
+        es: "la GitHub App de {rol} no consiguio su token de instalacion ({causa}: {detalle}): revisa {variables}, que la App este instalada en el repo y que tenga los permisos del rol",
+        en: "the {rol} GitHub App did not get its installation token ({causa}: {detalle}): check {variables}, that the App is installed on the repo and that it has the role permissions",
+    }
     SOLO_TERMINAL {
         es: " (el de persona solo se usa desde un terminal interactivo)",
         en: " (the persona one is only used from an interactive terminal)",
diff --git a/crates/tc2/src/identidad.rs b/crates/tc2/src/identidad.rs
index debebab..ce77c31 100644
--- a/crates/tc2/src/identidad.rs
+++ b/crates/tc2/src/identidad.rs
@@ -30,8 +30,8 @@ use crate::i18n::catalogo::identidad as t;
 use crate::i18n::{Mensaje, Texto};
 use crate::plataforma::personas::RUTA as RUTA_PERSONAS;
 use crate::plataforma::{
-    Credenciales, CredencialesEntorno, Destino, GitHub, GitLab, Personas, Plataforma, PrAbierto,
-    Proveedor, Rol, Usuario, detectar,
+    Credenciales, Destino, GitHub, GitLab, Personas, Plataforma, PrAbierto, Proveedor, Rol,
+    Usuario, de_entorno, detectar,
 };
 use crate::tarea::Asignado;
 
@@ -571,15 +571,17 @@ pub fn conectar_produccion<'a>(
     move |personas: &Personas| {
         // La misma deteccion que `identidad` y `start` (D6 de TASK-033).
         let destino = destino_de(g, dir).map_err(|_| Consulta::NoConfigurada)?;
-        let credenciales = CredencialesEntorno;
-        let rol = if credenciales.token(Rol::Persona).is_some() {
+        // `tiene` no habla con la red: una App del agente cuenta como
+        // identidad aunque su token se acune despues (F6-1).
+        let credenciales = de_entorno();
+        let rol = if credenciales.tiene(Rol::Persona) {
             Rol::Persona
-        } else if credenciales.token(Rol::Agente).is_some() {
+        } else if credenciales.tiene(Rol::Agente) {
             Rol::Agente
         } else {
             return Err(Consulta::SinCredenciales);
         };
-        let plataforma = adaptador(destino, Box::new(credenciales), personas.clone())
+        let plataforma = adaptador(destino, credenciales, personas.clone())
             .map_err(|_| Consulta::NoConfigurada)?;
         Ok(Conectada { plataforma, rol })
     }
diff --git a/crates/tc2/src/init/inventario.rs b/crates/tc2/src/init/inventario.rs
index 944c6e7..00b04be 100644
--- a/crates/tc2/src/init/inventario.rs
+++ b/crates/tc2/src/init/inventario.rs
@@ -761,7 +761,7 @@ fn plataforma(
         Err(i) => return Ok(Err(motivo_de_deteccion(&i))),
     };
     let credenciales = (c.credenciales)();
-    if credenciales.token(Rol::Persona).is_none() {
+    if !credenciales.tiene(Rol::Persona) {
         return Ok(Err("sin-token".to_owned()));
     }
     // La lista de personas, si ya existe y vale: `aplicar-puertas` exige
diff --git a/crates/tc2/src/init/mod.rs b/crates/tc2/src/init/mod.rs
index 86d759f..f97faa8 100644
--- a/crates/tc2/src/init/mod.rs
+++ b/crates/tc2/src/init/mod.rs
@@ -562,8 +562,7 @@ pub fn orden(args: &[String], json: bool, idioma: Idioma) -> Salida {
         }
     };
     let git = Git::nuevo();
-    let credenciales =
-        || -> Box<dyn Credenciales> { Box::new(crate::plataforma::CredencialesEntorno) };
+    let credenciales = || -> Box<dyn Credenciales> { crate::plataforma::de_entorno() };
     let c = Contexto {
         dir: &dir,
         git: &git,
diff --git a/crates/tc2/src/lib.rs b/crates/tc2/src/lib.rs
index a507f24..4675f64 100644
--- a/crates/tc2/src/lib.rs
+++ b/crates/tc2/src/lib.rs
@@ -353,7 +353,7 @@ fn orden_identidad(args: &[String]) -> Informe {
             dir: &dir,
             git: &g,
             tty: std::io::stdin().is_terminal(),
-            credenciales: Box::new(plataforma::CredencialesEntorno),
+            credenciales: plataforma::de_entorno(),
             ahora: identidad::ahora(),
         },
     )
@@ -479,7 +479,7 @@ fn transicion(
             dir,
             git: g,
             tty: std::io::stdin().is_terminal(),
-            credenciales: Box::new(plataforma::CredencialesEntorno),
+            credenciales: plataforma::de_entorno(),
             ahora,
         })
     };
diff --git a/crates/tc2/src/plataforma/app_github.rs b/crates/tc2/src/plataforma/app_github.rs
new file mode 100644
index 0000000..7c803e7
--- /dev/null
+++ b/crates/tc2/src/plataforma/app_github.rs
@@ -0,0 +1,455 @@
+//! Credenciales de una GitHub App (F6-1; ADR F1-6, decisiones 2 y 5).
+//!
+//! Una App no tiene token fijo: el CLI firma un JWT RS256 con la clave
+//! privada de la App (`iss` = id de la App, `iat` 60 s en el pasado, `exp` a
+//! 9 minutos), busca la instalacion del repo (`GET
+//! /repos/{o}/{r}/installation`) y pide un token de instalacion (`POST
+//! /app/installations/{id}/access_tokens`) **reducido a ese repo y a los
+//! permisos de la tabla 2 del ADR para el rol** ([`permisos`]). El token se
+//! guarda hasta `expires_at` menos [`MARGEN`] y luego se renueva.
+//!
+//! Habla con la API por el mismo [`Cliente`] (y la misma [`Base`]) que el
+//! adaptador: ruta validada, sin redirecciones y sin el token en errores ni
+//! registro. La clave se lee con `ring` (ya en el arbol por rustls): PEM
+//! PKCS#1 (`BEGIN RSA PRIVATE KEY`, el que descarga GitHub) o PKCS#8
+//! (`BEGIN PRIVATE KEY`).
+
+use std::cell::{Cell, RefCell};
+use std::path::Path;
+use std::time::{SystemTime, UNIX_EPOCH};
+
+use base64::Engine as _;
+use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
+use ring::rand::SystemRandom;
+use ring::signature::{RSA_PKCS1_SHA256, RsaKeyPair};
+use serde::Deserialize;
+
+use super::credenciales::{Credenciales, FalloCredencial, Rol, Secreto, causas, id_valido};
+use super::errores::sanear;
+use super::http::{Base, Cliente, FalloHttp, Metodo, Respuesta};
+
+/// Antes de `expires_at` se deja de usar un token y se pide otro (los
+/// tokens de instalacion duran 1 h).
+pub const MARGEN: u64 = 5 * 60;
+
+/// El JWT se firma con `iat` este numero de segundos en el pasado (deriva de
+/// reloj, como pide GitHub).
+const ATRAS: u64 = 60;
+
+/// Vida del JWT (GitHub admite hasta 10 minutos).
+const VIDA_JWT: u64 = 9 * 60;
+
+/// Permisos del token de instalacion de cada rol: la tabla 2 del ADR F1-6.
+/// La persona no tiene App.
+pub fn permisos(rol: Rol) -> Option<&'static [(&'static str, &'static str)]> {
+    match rol {
+        Rol::Agente => Some(&[
+            ("contents", "write"),
+            ("pull_requests", "write"),
+            ("checks", "read"),
+            ("statuses", "read"),
+            ("metadata", "read"),
+        ]),
+        Rol::Revisor => Some(&[
+            ("checks", "write"),
+            ("contents", "read"),
+            ("pull_requests", "read"),
+            ("metadata", "read"),
+        ]),
+        Rol::Persona => None,
+    }
+}
+
+#[derive(Deserialize)]
+struct InstalacionApi {
+    id: u64,
+}
+
+#[derive(Deserialize)]
+struct TokenApi {
+    token: String,
+    expires_at: String,
+}
+
+#[derive(Deserialize)]
+struct AppApi {
+    slug: String,
+}
+
+/// Credenciales de la App de un rol para un repo.
+pub struct CredencialesApp {
+    id: String,
+    rol: Rol,
+    clave: RsaKeyPair,
+    cliente: Cliente,
+    repo: String,
+    instalacion: Cell<Option<u64>>,
+    cache: RefCell<Option<(Secreto, u64)>>,
+}
+
+impl std::fmt::Debug for CredencialesApp {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("CredencialesApp")
+            .field("id", &self.id)
+            .field("rol", &self.rol)
+            .field("repo", &self.repo)
+            .finish_non_exhaustive()
+    }
+}
+
+impl CredencialesApp {
+    /// App `id` del `rol` con su clave en PEM, para `repo` (`dueno/repo`) en
+    /// la API `base`.
+    pub fn nueva(
+        id: &str,
+        pem: &str,
+        base: Base,
+        repo: &str,
+        rol: Rol,
+    ) -> Result<Self, FalloCredencial> {
+        let incompleta = |detalle: &str| FalloCredencial::Acunar {
+            causa: causas::APP_INCOMPLETA,
+            detalle: detalle.to_owned(),
+        };
+        if !id_valido(id) {
+            return Err(incompleta("id"));
+        }
+        if permisos(rol).is_none() {
+            return Err(incompleta(rol.nombre()));
+        }
+        if !repo_valido(repo) {
+            return Err(incompleta("repo"));
+        }
+        Ok(Self {
+            id: id.to_owned(),
+            rol,
+            clave: leer_pem(pem)?,
+            cliente: Cliente::nuevo(base),
+            repo: repo.to_owned(),
+            instalacion: Cell::new(None),
+            cache: RefCell::new(None),
+        })
+    }
+
+    /// Como [`CredencialesApp::nueva`], con la clave en el fichero `ruta`.
+    pub fn desde_fichero(
+        id: &str,
+        ruta: &Path,
+        base: &Base,
+        repo: &str,
+        rol: Rol,
+    ) -> Result<Self, FalloCredencial> {
+        let pem = std::fs::read_to_string(ruta).map_err(|e| FalloCredencial::Acunar {
+            causa: causas::CLAVE_ILEGIBLE,
+            detalle: sanear(&e.kind().to_string()),
+        })?;
+        Self::nueva(id, &pem, base.clone(), repo, rol)
+    }
+
+    /// Llamadas hechas, `METODO ruta`.
+    pub fn registro(&self) -> Vec<String> {
+        self.cliente.registro()
+    }
+
+    /// El `slug` de la App (`GET /app` con el JWT): su cuenta es
+    /// `<slug>[bot]`.
+    pub fn slug(&self) -> Result<String, FalloCredencial> {
+        let jwt = self.firmar(ahora())?;
+        let r = self.llamar(Metodo::Get, "/app", &jwt, None)?;
+        if r.estado != 200 {
+            return Err(fallo_estado(&r, causas::RESPUESTA_INESPERADA));
+        }
+        let a: AppApi = leer(&r)?;
+        Ok(a.slug)
+    }
+
+    /// JWT RS256 de la App para el instante `ahora`.
+    fn firmar(&self, ahora: u64) -> Result<Secreto, FalloCredencial> {
+        let cabecera = URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#);
+        // El id numerico va como numero; un client id (`Iv1.…`), como texto
+        // (`id_valido` ya excluye comillas y barras).
+        let iss = if self.id.bytes().all(|b| b.is_ascii_digit()) {
+            self.id.clone()
+        } else {
+            format!("\"{}\"", self.id)
+        };
+        let carga = format!(
+            r#"{{"iat":{},"exp":{},"iss":{iss}}}"#,
+            ahora.saturating_sub(ATRAS),
+            ahora + VIDA_JWT
+        );
+        let entrada = format!("{cabecera}.{}", URL_SAFE_NO_PAD.encode(carga));
+        let mut firma = vec![0u8; self.clave.public().modulus_len()];
+        self.clave
+            .sign(
+                &RSA_PKCS1_SHA256,
+                &SystemRandom::new(),
+                entrada.as_bytes(),
+                &mut firma,
+            )
+            .map_err(|_| FalloCredencial::Acunar {
+                causa: causas::CLAVE_ILEGIBLE,
+                detalle: "firma".to_owned(),
+            })?;
+        Ok(Secreto::nuevo(format!(
+            "{entrada}.{}",
+            URL_SAFE_NO_PAD.encode(firma)
+        )))
+    }
+
+    fn llamar(
+        &self,
+        metodo: Metodo,
+        ruta: &str,
+        jwt: &Secreto,
+        cuerpo: Option<&str>,
+    ) -> Result<Respuesta, FalloCredencial> {
+        self.cliente
+            .llamar(metodo, ruta, Some(jwt), cuerpo)
+            .map_err(|f| match f {
+                FalloHttp::Sinred(m) => FalloCredencial::Acunar {
+                    causa: causas::SINRED,
+                    detalle: sanear(&m),
+                },
+                otro => FalloCredencial::Acunar {
+                    causa: causas::RESPUESTA_INESPERADA,
+                    detalle: sanear(&format!("{otro:?}")),
+                },
+            })
+    }
+
+    /// Id de la instalacion de la App en el repo (una vez por proceso).
+    fn instalacion(&self, jwt: &Secreto) -> Result<u64, FalloCredencial> {
+        if let Some(id) = self.instalacion.get() {
+            return Ok(id);
+        }
+        let ruta = format!("/repos/{}/installation", self.repo);
+        let r = self.llamar(Metodo::Get, &ruta, jwt, None)?;
+        if r.estado != 200 {
+            return Err(fallo_estado(&r, causas::RESPUESTA_INESPERADA));
+        }
+        let i: InstalacionApi = leer(&r)?;
+        self.instalacion.set(Some(i.id));
+        Ok(i.id)
+    }
+
+    /// Pide un token nuevo, reducido al repo y a los permisos del rol.
+    fn acunar(&self, ahora: u64) -> Result<(Secreto, u64), FalloCredencial> {
+        let jwt = self.firmar(ahora)?;
+        let instalacion = self.instalacion(&jwt)?;
+        let nombre = self.repo.split('/').nth(1).unwrap_or_default();
+        let permisos: serde_json::Map<String, serde_json::Value> = permisos(self.rol)
+            .unwrap_or_default()
+            .iter()
+            .map(|(k, v)| ((*k).to_owned(), (*v).into()))
+            .collect();
+        let cuerpo = serde_json::json!({
+            "repositories": [nombre],
+            "permissions": permisos,
+        })
+        .to_string();
+        let ruta = format!("/app/installations/{instalacion}/access_tokens");
+        let r = self.llamar(Metodo::Post, &ruta, &jwt, Some(&cuerpo))?;
+        if r.estado != 201 {
+            if r.estado == 404 {
+                // La App se desinstalo o se reinstalo con otro id.
+                self.instalacion.set(None);
+            }
+            return Err(fallo_estado(&r, causas::PERMISOS_NO_CONCEDIDOS));
+        }
+        let t: TokenApi = leer(&r)?;
+        let expira = crate::inventario::ms_de_iso(&t.expires_at)
+            .and_then(|ms| u64::try_from(ms / 1000).ok())
+            .ok_or_else(|| FalloCredencial::Acunar {
+                causa: causas::RESPUESTA_INESPERADA,
+                detalle: "expires_at".to_owned(),
+            })?;
+        if t.token.trim().is_empty() || t.token.chars().any(char::is_control) {
+            return Err(FalloCredencial::Acunar {
+                causa: causas::RESPUESTA_INESPERADA,
+                detalle: "token".to_owned(),
+            });
+        }
+        Ok((Secreto::nuevo(t.token), expira))
+    }
+}
+
+impl Credenciales for CredencialesApp {
+    fn token(&self, rol: Rol) -> Option<Secreto> {
+        self.obtener(rol).ok()
+    }
+
+    fn obtener(&self, rol: Rol) -> Result<Secreto, FalloCredencial> {
+        if rol != self.rol {
+            return Err(FalloCredencial::Falta);
+        }
+        let ahora = ahora();
+        if let Some((s, expira)) = self.cache.borrow().as_ref()
+            && expira.saturating_sub(MARGEN) > ahora
+        {
+            return Ok(s.clone());
+        }
+        let (s, expira) = self.acunar(ahora)?;
+        *self.cache.borrow_mut() = Some((s.clone(), expira));
+        Ok(s)
+    }
+
+    fn tiene(&self, rol: Rol) -> bool {
+        rol == self.rol
+    }
+}
+
+/// Segundos desde 1970.
+fn ahora() -> u64 {
+    SystemTime::now()
+        .duration_since(UNIX_EPOCH)
+        .map_or(0, |d| d.as_secs())
+}
+
+/// `dueno/repo` con la misma forma que exige el adaptador.
+fn repo_valido(repo: &str) -> bool {
+    let partes: Vec<&str> = repo.split('/').collect();
+    partes.len() == 2
+        && partes.iter().all(|p| {
+            !p.is_empty()
+                && *p != "."
+                && *p != ".."
+                && p.bytes()
+                    .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
+        })
+}
+
+/// Fallo por el estado HTTP: 401 es el JWT, 404 la instalacion, 422 los
+/// permisos (`en_422`) y el resto, inesperado.
+fn fallo_estado(r: &Respuesta, en_422: &'static str) -> FalloCredencial {
+    let causa = match r.estado {
+        401 => causas::JWT_RECHAZADO,
+        404 => causas::SIN_INSTALACION,
+        422 => en_422,
+        _ => causas::RESPUESTA_INESPERADA,
+    };
+    let mensaje = serde_json::from_str::<serde_json::Value>(&r.cuerpo)
+        .ok()
+        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(sanear))
+        .unwrap_or_default();
+    FalloCredencial::Acunar {
+        causa,
+        detalle: sanear(&format!("HTTP {} {mensaje}", r.estado)),
+    }
+}
+
+fn leer<T: serde::de::DeserializeOwned>(r: &Respuesta) -> Result<T, FalloCredencial> {
+    serde_json::from_str(&r.cuerpo).map_err(|e| FalloCredencial::Acunar {
+        causa: causas::RESPUESTA_INESPERADA,
+        detalle: sanear(&e.to_string()),
+    })
+}
+
+/// Clave RSA de un PEM PKCS#1 o PKCS#8 sin cifrar.
+fn leer_pem(pem: &str) -> Result<RsaKeyPair, FalloCredencial> {
+    let ilegible = |detalle: &str| FalloCredencial::Acunar {
+        causa: causas::CLAVE_ILEGIBLE,
+        detalle: detalle.to_owned(),
+    };
+    let mut lineas = pem.lines().map(str::trim).skip_while(|l| l.is_empty());
+    let etiqueta = lineas
+        .next()
+        .and_then(|l| l.strip_prefix("-----BEGIN "))
+        .and_then(|l| l.strip_suffix("-----"))
+        .ok_or_else(|| ilegible("pem"))?
+        .to_owned();
+    let fin = format!("-----END {etiqueta}-----");
+    let mut b64 = String::new();
+    let mut cerrado = false;
+    for l in lineas {
+        if l == fin {
+            cerrado = true;
+            break;
+        }
+        // Cabeceras (`Proc-Type: 4,ENCRYPTED`): clave cifrada, no se admite.
+        if l.contains(':') {
+            return Err(ilegible("pem cifrado"));
+        }
+        b64.push_str(l);
+    }
+    if !cerrado {
+        return Err(ilegible("pem"));
+    }
+    let der = STANDARD.decode(b64).map_err(|_| ilegible("pem base64"))?;
+    let clave = match etiqueta.as_str() {
+        "RSA PRIVATE KEY" => RsaKeyPair::from_der(&der),
+        "PRIVATE KEY" => RsaKeyPair::from_pkcs8(&der),
+        _ => return Err(ilegible("pem no rsa")),
+    };
+    clave.map_err(|e| ilegible(&e.to_string()))
+}
+
+#[cfg(test)]
+#[allow(clippy::panic)]
+mod tests {
+    use super::*;
+
+    /// C2.2 del plan de TASK-047: los permisos por rol son los de la tabla
+    /// 2 del ADR F1-6; el revisor no pide escritura de PR ni de contents.
+    #[test]
+    fn los_permisos_por_rol_son_los_de_la_tabla_del_adr() {
+        let agente: Vec<_> = permisos(Rol::Agente).unwrap_or_default().to_vec();
+        assert_eq!(
+            agente,
+            vec![
+                ("contents", "write"),
+                ("pull_requests", "write"),
+                ("checks", "read"),
+                ("statuses", "read"),
+                ("metadata", "read"),
+            ]
+        );
+        let revisor: Vec<_> = permisos(Rol::Revisor).unwrap_or_default().to_vec();
+        assert_eq!(
+            revisor,
+            vec![
+                ("checks", "write"),
+                ("contents", "read"),
+                ("pull_requests", "read"),
+                ("metadata", "read"),
+            ]
+        );
+        for (k, v) in &revisor {
+            assert!(
+                !(*v == "write" && (*k == "pull_requests" || *k == "contents")),
+                "el revisor no puede aprobar ni empujar: {k}:{v}"
+            );
+        }
+        // Ninguna App administra, toca workflows ni escribe statuses.
+        for p in [agente, revisor] {
+            for (k, v) in p {
+                assert!(
+                    !["administration", "workflows", "secrets", "issues"].contains(&k),
+                    "{k}"
+                );
+                assert!(!(k == "statuses" && v == "write"));
+            }
+        }
+        assert!(permisos(Rol::Persona).is_none());
+    }
+
+    #[test]
+    fn pem_que_no_vale() {
+        for pem in [
+            "",
+            "hola",
+            "-----BEGIN RSA PRIVATE KEY-----\nAAAA\n",
+            "-----BEGIN RSA PRIVATE KEY-----\n%%%%\n-----END RSA PRIVATE KEY-----\n",
+            "-----BEGIN RSA PRIVATE KEY-----\nProc-Type: 4,ENCRYPTED\nAAAA\n-----END RSA PRIVATE KEY-----\n",
+            "-----BEGIN EC PRIVATE KEY-----\nAAAA\n-----END EC PRIVATE KEY-----\n",
+            "-----BEGIN RSA PRIVATE KEY-----\nAAAA\n-----END RSA PRIVATE KEY-----\n",
+        ] {
+            match leer_pem(pem) {
+                Err(FalloCredencial::Acunar { causa, .. }) => {
+                    assert_eq!(causa, causas::CLAVE_ILEGIBLE, "{pem}");
+                }
+                otro => panic!("{pem}: {:?}", otro.map(|_| ())),
+            }
+        }
+    }
+}
diff --git a/crates/tc2/src/plataforma/conexion.rs b/crates/tc2/src/plataforma/conexion.rs
index 4a8306b..20cbc17 100644
--- a/crates/tc2/src/plataforma/conexion.rs
+++ b/crates/tc2/src/plataforma/conexion.rs
@@ -11,7 +11,7 @@ use std::collections::BTreeMap;
 
 use serde::de::DeserializeOwned;
 
-use super::credenciales::{Credenciales, Rol, Secreto};
+use super::credenciales::{Credenciales, FalloCredencial, Rol, Secreto, causas};
 use super::errores::{self, Permiso};
 use super::http::{Base, Cliente, Metodo, Respuesta};
 use super::{Paso, Personas, Proveedor, Usuario};
@@ -72,21 +72,43 @@ impl Conexion {
     }
 
     /// Token del rol, o el rechazo dirigido si falta (nunca se prueba con el
-    /// de otro rol).
+    /// de otro rol). Una App que no consigue su token (F6-1) no es «falta»:
+    /// `denegada motivo=credenciales causa=<causa>`, o `sinred`.
     pub fn token(&self, op: &str, rol: Rol) -> Paso<Secreto> {
-        self.credenciales.token(rol).ok_or_else(|| {
-            let m = Mensaje::nuevo(&t::FALTA_IDENTIDAD)
-                .con("rol", rol.nombre())
-                .con("variable", rol.variable());
-            let m = if rol == Rol::Persona {
-                m.con_mensaje("extra", &t::SOLO_TERMINAL)
-            } else {
-                m.con("extra", "")
-            };
-            Informe::nuevo(op, Resultado::Rechazada)
-                .con("motivo", "identidad")
-                .con("falta", rol.nombre())
-                .diagnostico(m)
+        self.credenciales.obtener(rol).map_err(|f| match f {
+            FalloCredencial::Falta => {
+                let m = Mensaje::nuevo(&t::FALTA_IDENTIDAD)
+                    .con("rol", rol.nombre())
+                    .con("variable", rol.variable());
+                let m = if rol == Rol::Persona {
+                    m.con_mensaje("extra", &t::SOLO_TERMINAL)
+                } else {
+                    m.con("extra", "")
+                };
+                Informe::nuevo(op, Resultado::Rechazada)
+                    .con("motivo", "identidad")
+                    .con("falta", rol.nombre())
+                    .diagnostico(m)
+            }
+            FalloCredencial::Acunar { causa, detalle } => {
+                let variables = rol
+                    .variables_app()
+                    .map(|(a, b)| format!("{a}, {b}"))
+                    .unwrap_or_default();
+                let m = Mensaje::nuevo(&t::APP_SIN_TOKEN)
+                    .con("rol", rol.nombre())
+                    .con("causa", causa)
+                    .con("detalle", errores::sanear(&detalle))
+                    .con("variables", variables);
+                let i = if causa == causas::SINRED {
+                    Informe::nuevo(op, Resultado::Sinred)
+                } else {
+                    Informe::nuevo(op, Resultado::Denegada).con("motivo", "credenciales")
+                };
+                i.con("rol", rol.nombre())
+                    .con("causa", causa)
+                    .diagnostico(m)
+            }
         })
     }
 
diff --git a/crates/tc2/src/plataforma/credenciales.rs b/crates/tc2/src/plataforma/credenciales.rs
index c18cd99..3c27dc0 100644
--- a/crates/tc2/src/plataforma/credenciales.rs
+++ b/crates/tc2/src/plataforma/credenciales.rs
@@ -1,14 +1,20 @@
-//! Credenciales por rol (ADR F1-6, decisiones 1 y 5).
+//! Credenciales por rol (ADR F1-6, decisiones 1 y 5; F6-1).
 //!
 //! Cada operacion del puerto declara con que [`Rol`] corre y pide su token a
 //! un proveedor de [`Credenciales`]. Si falta, la operacion sale `rechazada
 //! motivo=identidad` y nunca prueba con otro rol.
 //!
-//! Hasta que F6-1 cree las Apps, el proveedor de produccion lee variables de
-//! entorno; el token de persona solo se usa con un terminal interactivo.
+//! El proveedor de produccion sale de [`de_entorno`] (F6-1): por rol, una
+//! GitHub App (`TC2_APP_<ROL>_ID` y `TC2_APP_<ROL>_CLAVE`, ruta al PEM) si
+//! esta configurada y, si no, el token de `TC2_TOKEN_<ROL>`. El token de
+//! persona solo se usa con un terminal interactivo y la persona no tiene App.
 
 use std::collections::BTreeMap;
 use std::io::IsTerminal;
+use std::path::PathBuf;
+
+use super::app_github::CredencialesApp;
+use super::http::Base;
 
 /// Identidad con la que corre una operacion (F1-6).
 #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
@@ -39,6 +45,16 @@ impl Rol {
             Self::Persona => "TC2_TOKEN_PERSONA",
         }
     }
+
+    /// Variables de su GitHub App (id y ruta al PEM). La persona no tiene
+    /// App: usa su cuenta (F1-6, tabla 2).
+    pub fn variables_app(self) -> Option<(&'static str, &'static str)> {
+        match self {
+            Self::Agente => Some(("TC2_APP_AGENTE_ID", "TC2_APP_AGENTE_CLAVE")),
+            Self::Revisor => Some(("TC2_APP_REVISOR_ID", "TC2_APP_REVISOR_CLAVE")),
+            Self::Persona => None,
+        }
+    }
 }
 
 /// Un token. No tiene `Display` y su `Debug` no lo ensena.
@@ -51,7 +67,7 @@ impl Secreto {
         Self(token.into())
     }
 
-    /// El valor, solo para la cabecera `Authorization`.
+    /// El valor, solo para la cabecera `Authorization` (de la API o de git).
     pub(crate) fn exponer(&self) -> &str {
         &self.0
     }
@@ -63,27 +79,187 @@ impl std::fmt::Debug for Secreto {
     }
 }
 
+/// Por que no hay token de un rol.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum FalloCredencial {
+    /// El rol no tiene identidad configurada (ni App ni token).
+    Falta,
+    /// Hay App configurada, pero no se pudo acunar su token de instalacion.
+    /// `causa` va a la linea (`causa=`) y es una de [`causas`]; `detalle`,
+    /// saneado y sin secretos, al diagnostico.
+    Acunar {
+        /// Causa corta.
+        causa: &'static str,
+        /// Detalle para la persona.
+        detalle: String,
+    },
+}
+
+/// Valores de `causa` de [`FalloCredencial::Acunar`].
+pub mod causas {
+    /// La App esta a medias: id sin clave o al reves, o id no valido.
+    pub const APP_INCOMPLETA: &str = "app-incompleta";
+    /// La App solo existe en GitHub (en GitLab va un token de proyecto o de
+    /// cuenta de servicio en `TC2_TOKEN_<ROL>`).
+    pub const APP_SOLO_GITHUB: &str = "app-solo-github";
+    /// No se pudo leer el PEM o no es una clave RSA que se entienda.
+    pub const CLAVE_ILEGIBLE: &str = "clave-ilegible";
+    /// GitHub rechaza el JWT de la App (401).
+    pub const JWT_RECHAZADO: &str = "jwt-rechazado";
+    /// La App no esta instalada en el repo (404).
+    pub const SIN_INSTALACION: &str = "sin-instalacion";
+    /// La instalacion no concede los permisos pedidos para el rol (422).
+    pub const PERMISOS_NO_CONCEDIDOS: &str = "permisos-no-concedidos";
+    /// Sin red al pedir el token.
+    pub const SINRED: &str = "sinred";
+    /// Respuesta con otro estado o con una forma inesperada.
+    pub const RESPUESTA_INESPERADA: &str = "respuesta-inesperada";
+}
+
 /// Proveedor de tokens por rol.
 pub trait Credenciales {
     /// Token de ese rol, si lo hay.
     fn token(&self, rol: Rol) -> Option<Secreto>;
+
+    /// Token de ese rol, o por que no lo hay: una App que no consigue su
+    /// token no se confunde con «falta la identidad».
+    fn obtener(&self, rol: Rol) -> Result<Secreto, FalloCredencial> {
+        self.token(rol).ok_or(FalloCredencial::Falta)
+    }
+
+    /// Si el rol tiene identidad configurada, sin hablar con la red (para
+    /// elegir con que rol se pregunta; no es un reintento con otro rol).
+    fn tiene(&self, rol: Rol) -> bool {
+        self.token(rol).is_some()
+    }
+
+    /// Ata el proveedor al repo de GitHub `repo` en la API `base`: la App
+    /// necesita saber que instalacion pedir. Lo llama el adaptador de
+    /// GitHub al crearse; por defecto no hace nada.
+    fn atar_github(&mut self, _base: &Base, _repo: &str) {}
+}
+
+/// Configuracion de una App leida del entorno.
+#[derive(Debug, Clone)]
+struct ConfigApp {
+    id: String,
+    clave: PathBuf,
+}
+
+/// Lo que da el entorno para un rol con App: la App ya atada a un repo, la
+/// configuracion aun sin atar, o la App a medias.
+enum EstadoApp {
+    SinAtar(ConfigApp),
+    Atada(Box<CredencialesApp>),
+    Fallo(FalloCredencial),
+}
+
+/// Proveedor de produccion (F6-1): App por rol si esta configurada, si no
+/// el token de `TC2_TOKEN_<ROL>`; nunca pasa de un rol a otro. El token de
+/// persona solo con terminal (F1-6, decision 5).
+pub struct CredencialesDeEntorno {
+    tokens: BTreeMap<Rol, Secreto>,
+    apps: BTreeMap<Rol, EstadoApp>,
+}
+
+impl std::fmt::Debug for CredencialesDeEntorno {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("CredencialesDeEntorno")
+            .field("tokens", &self.tokens.keys().collect::<Vec<_>>())
+            .field("apps", &self.apps.keys().collect::<Vec<_>>())
+            .finish()
+    }
 }
 
-/// Tokens de las variables `TC2_TOKEN_<ROL>`. El de persona solo si la
-/// entrada estandar es un terminal (F1-6 decision 5: token de persona solo
-/// con TTY).
-#[derive(Debug, Clone, Copy, Default)]
-pub struct CredencialesEntorno;
+/// El proveedor de produccion con las variables y el terminal del proceso.
+pub fn de_entorno() -> Box<dyn Credenciales> {
+    Box::new(de_entorno_con(
+        |v| std::env::var(v).ok(),
+        std::io::stdin().is_terminal(),
+    ))
+}
+
+/// El proveedor con un lector de variables y si hay terminal (los tests no
+/// tocan el entorno del proceso: `set_var` exige `unsafe`).
+pub fn de_entorno_con(leer: impl Fn(&str) -> Option<String>, tty: bool) -> CredencialesDeEntorno {
+    let valor = |v: &str| leer(v).filter(|t| !t.trim().is_empty());
+    let mut tokens = BTreeMap::new();
+    let mut apps = BTreeMap::new();
+    for rol in [Rol::Agente, Rol::Revisor, Rol::Persona] {
+        if rol == Rol::Persona && !tty {
+            continue;
+        }
+        if let Some((vid, vclave)) = rol.variables_app() {
+            match (valor(vid), valor(vclave)) {
+                (None, None) => {}
+                (Some(id), Some(clave)) if id_valido(id.trim()) => {
+                    apps.insert(
+                        rol,
+                        EstadoApp::SinAtar(ConfigApp {
+                            id: id.trim().to_owned(),
+                            clave: PathBuf::from(clave.trim()),
+                        }),
+                    );
+                    continue;
+                }
+                _ => {
+                    apps.insert(
+                        rol,
+                        EstadoApp::Fallo(FalloCredencial::Acunar {
+                            causa: causas::APP_INCOMPLETA,
+                            detalle: format!("{vid}, {vclave}"),
+                        }),
+                    );
+                    continue;
+                }
+            }
+        }
+        if let Some(t) = valor(rol.variable()) {
+            tokens.insert(rol, Secreto::nuevo(t));
+        }
+    }
+    CredencialesDeEntorno { tokens, apps }
+}
+
+/// Id de App (numerico) o client id (`Iv1.` y alfanumericos).
+pub(crate) fn id_valido(id: &str) -> bool {
+    !id.is_empty()
+        && id.len() <= 64
+        && id
+            .bytes()
+            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
+}
 
-impl Credenciales for CredencialesEntorno {
+impl Credenciales for CredencialesDeEntorno {
     fn token(&self, rol: Rol) -> Option<Secreto> {
-        if rol == Rol::Persona && !std::io::stdin().is_terminal() {
-            return None;
+        self.obtener(rol).ok()
+    }
+
+    fn obtener(&self, rol: Rol) -> Result<Secreto, FalloCredencial> {
+        match self.apps.get(&rol) {
+            Some(EstadoApp::Atada(app)) => app.obtener(rol),
+            Some(EstadoApp::Fallo(f)) => Err(f.clone()),
+            Some(EstadoApp::SinAtar(c)) => Err(FalloCredencial::Acunar {
+                causa: causas::APP_SOLO_GITHUB,
+                detalle: format!("app {}", c.id),
+            }),
+            None => self.tokens.get(&rol).cloned().ok_or(FalloCredencial::Falta),
+        }
+    }
+
+    fn tiene(&self, rol: Rol) -> bool {
+        self.apps.contains_key(&rol) || self.tokens.contains_key(&rol)
+    }
+
+    fn atar_github(&mut self, base: &Base, repo: &str) {
+        for (rol, estado) in &mut self.apps {
+            if let EstadoApp::SinAtar(c) = estado {
+                *estado = match CredencialesApp::desde_fichero(&c.id, &c.clave, base, repo, *rol) {
+                    Ok(app) => EstadoApp::Atada(Box::new(app)),
+                    Err(f) => EstadoApp::Fallo(f),
+                };
+            }
         }
-        std::env::var(rol.variable())
-            .ok()
-            .filter(|t| !t.trim().is_empty())
-            .map(Secreto::nuevo)
     }
 }
 
@@ -112,6 +288,7 @@ impl Credenciales for CredencialesFijas {
 }
 
 #[cfg(test)]
+#[allow(clippy::panic)]
 mod tests {
     use super::*;
 
@@ -123,5 +300,89 @@ mod tests {
         assert!(!format!("{c:?}").contains("supersecreto"));
         assert!(c.token(Rol::Persona).is_none());
         assert!(c.token(Rol::Agente).is_some());
+        let e = de_entorno_con(
+            |v| (v == "TC2_TOKEN_AGENTE").then(|| "ghp_supersecreto".to_owned()),
+            false,
+        );
+        assert!(!format!("{e:?}").contains("supersecreto"));
+    }
+
+    fn lector(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
+        let m: BTreeMap<String, String> = vars
+            .iter()
+            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
+            .collect();
+        move |k| m.get(k).cloned()
+    }
+
+    /// C2.1 del plan de TASK-047: la App gana al token, sin paso entre
+    /// roles, la persona solo con terminal y una App que no acuna no es
+    /// «falta».
+    #[test]
+    fn la_fabrica_da_precedencia_a_la_app_sin_paso_entre_roles() {
+        let todo = [
+            ("TC2_TOKEN_AGENTE", "tok-agente"),
+            ("TC2_TOKEN_PERSONA", "tok-persona"),
+            ("TC2_APP_AGENTE_ID", "123"),
+            ("TC2_APP_AGENTE_CLAVE", "/no/existe/clave.pem"),
+        ];
+        // Sin terminal: la persona no tiene token aunque la variable exista.
+        let c = de_entorno_con(lector(&todo), false);
+        assert!(!c.tiene(Rol::Persona));
+        assert_eq!(c.obtener(Rol::Persona), Err(FalloCredencial::Falta));
+        // Con terminal, si.
+        let c = de_entorno_con(lector(&todo), true);
+        assert_eq!(c.token(Rol::Persona), Some(Secreto::nuevo("tok-persona")));
+        // El agente tiene App: su token fijo no se usa nunca, y sin atar a
+        // un repo de GitHub la App no da token (ni cae al de la variable).
+        assert!(c.tiene(Rol::Agente));
+        match c.obtener(Rol::Agente) {
+            Err(FalloCredencial::Acunar { causa, .. }) => {
+                assert_eq!(causa, causas::APP_SOLO_GITHUB);
+            }
+            otro => panic!("{otro:?}"),
+        }
+        // Atada, con una clave que no existe: fallo al acunar, no «falta».
+        let mut c = de_entorno_con(lector(&todo), true);
+        c.atar_github(&Base::github(), "o/r");
+        match c.obtener(Rol::Agente) {
+            Err(FalloCredencial::Acunar { causa, detalle }) => {
+                assert_eq!(causa, causas::CLAVE_ILEGIBLE);
+                assert!(!detalle.contains("tok-agente"), "{detalle}");
+            }
+            otro => panic!("{otro:?}"),
+        }
+        // El revisor no tiene nada: falta, aunque el agente si tenga.
+        assert!(!c.tiene(Rol::Revisor));
+        assert_eq!(c.obtener(Rol::Revisor), Err(FalloCredencial::Falta));
+        // Sin App, el token del rol; nunca el de otro.
+        let c = de_entorno_con(lector(&[("TC2_TOKEN_REVISOR", "tok-rev")]), true);
+        assert_eq!(c.token(Rol::Revisor), Some(Secreto::nuevo("tok-rev")));
+        assert_eq!(c.obtener(Rol::Agente), Err(FalloCredencial::Falta));
+        // App a medias (id sin clave, o id no valido): fallo, no token.
+        for vars in [
+            vec![("TC2_APP_AGENTE_ID", "123"), ("TC2_TOKEN_AGENTE", "t")],
+            vec![
+                ("TC2_APP_AGENTE_CLAVE", "/x.pem"),
+                ("TC2_TOKEN_AGENTE", "t"),
+            ],
+            vec![
+                ("TC2_APP_AGENTE_ID", "12 3"),
+                ("TC2_APP_AGENTE_CLAVE", "/x.pem"),
+            ],
+        ] {
+            let c = de_entorno_con(lector(&vars), true);
+            match c.obtener(Rol::Agente) {
+                Err(FalloCredencial::Acunar { causa, .. }) => {
+                    assert_eq!(causa, causas::APP_INCOMPLETA, "{vars:?}");
+                }
+                otro => panic!("{vars:?}: {otro:?}"),
+            }
+        }
+        // La persona no tiene App: sus variables de App no existen.
+        assert_eq!(Rol::Persona.variables_app(), None);
+        // Variables vacias cuentan como ausentes.
+        let c = de_entorno_con(lector(&[("TC2_TOKEN_AGENTE", "  ")]), true);
+        assert!(!c.tiene(Rol::Agente));
     }
 }
diff --git a/crates/tc2/src/plataforma/entorno_git.rs b/crates/tc2/src/plataforma/entorno_git.rs
new file mode 100644
index 0000000..1b7253e
--- /dev/null
+++ b/crates/tc2/src/plataforma/entorno_git.rs
@@ -0,0 +1,85 @@
+//! Entorno de `git` para que el agente empuje y firme como autor con su
+//! propia identidad (F6-1; ADR F1-6, decision 1: `push` y `commit-estado`
+//! corren con el agente).
+//!
+//! El token va a `git` **por el entorno, nunca en la linea de ordenes**: con
+//! `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_<n>` y `GIT_CONFIG_VALUE_<n>` (config
+//! de ambito «command», Git >= 2.31) se define
+//! `http.<url>.extraheader = Authorization: Basic <base64(usuario:token)>`,
+//! limitado a la URL del remoto, y el autor y committer con
+//! `GIT_AUTHOR_*`/`GIT_COMMITTER_*`. Ni el registro del puerto `Git` ni los
+//! argumentos de los hijos de git (`git-remote-https`) lo ven.
+//!
+//! Quien llama compone el puerto con [`crate::git::Git::con_entorno_extra`]:
+//! la capa `git` no depende de `plataforma`. Conectarlo a `review` es de
+//! F6-2.
+
+use std::ffi::OsString;
+
+use base64::Engine as _;
+use base64::engine::general_purpose::STANDARD;
+
+use super::Proveedor;
+use super::credenciales::Secreto;
+
+/// Autor (y committer) de los commits del agente.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct AutorGit {
+    /// Nombre.
+    pub nombre: String,
+    /// Correo.
+    pub correo: String,
+}
+
+impl AutorGit {
+    /// La cuenta `<slug>[bot]` de una GitHub App. Con el id de esa cuenta,
+    /// el correo `<id>+<slug>[bot]@users.noreply.github.com`, que GitHub
+    /// asocia a la cuenta; sin el, `<slug>[bot]@users.noreply.github.com`.
+    pub fn bot_github(slug: &str, id_cuenta: Option<u64>) -> Self {
+        let login = format!("{slug}[bot]");
+        let correo = match id_cuenta {
+            Some(id) => format!("{id}+{login}@users.noreply.github.com"),
+            None => format!("{login}@users.noreply.github.com"),
+        };
+        Self {
+            nombre: login,
+            correo,
+        }
+    }
+}
+
+/// Usuario de la autenticacion basica de git con un token: el de los tokens
+/// de instalacion de GitHub y el de OAuth de GitLab (GitLab admite cualquier
+/// usuario no vacio con un token de proyecto o de cuenta de servicio).
+fn usuario(p: Proveedor) -> &'static str {
+    match p {
+        Proveedor::GitHub => "x-access-token",
+        Proveedor::GitLab => "oauth2",
+    }
+}
+
+/// Variables para que `git` hable con `url_remoto` (p. ej.
+/// `https://github.com/`; la cabecera solo va a las URL que empiezan asi)
+/// con `token` y firme como `autor`. Ocupan las entradas 0 de
+/// `GIT_CONFIG_*`: no se combinan con otras definidas igual.
+pub fn entorno_git(
+    proveedor: Proveedor,
+    url_remoto: &str,
+    token: &Secreto,
+    autor: &AutorGit,
+) -> Vec<(OsString, OsString)> {
+    let basica = STANDARD.encode(format!("{}:{}", usuario(proveedor), token.exponer()));
+    let par = |k: &str, v: String| (OsString::from(k), OsString::from(v));
+    vec![
+        par("GIT_CONFIG_COUNT", "1".to_owned()),
+        par("GIT_CONFIG_KEY_0", format!("http.{url_remoto}.extraheader")),
+        par(
+            "GIT_CONFIG_VALUE_0",
+            format!("Authorization: Basic {basica}"),
+        ),
+        par("GIT_AUTHOR_NAME", autor.nombre.clone()),
+        par("GIT_AUTHOR_EMAIL", autor.correo.clone()),
+        par("GIT_COMMITTER_NAME", autor.nombre.clone()),
+        par("GIT_COMMITTER_EMAIL", autor.correo.clone()),
+    ]
+}
diff --git a/crates/tc2/src/plataforma/github.rs b/crates/tc2/src/plataforma/github.rs
index 3710747..38b574b 100644
--- a/crates/tc2/src/plataforma/github.rs
+++ b/crates/tc2/src/plataforma/github.rs
@@ -325,6 +325,9 @@ impl GitHub {
                 .con("motivo", "repo-invalido")
                 .diagnostico(&t::REPO_GITHUB));
         }
+        // Una GitHub App necesita el repo para pedir su instalacion (F6-1).
+        let mut credenciales = credenciales;
+        credenciales.atar_github(&base, repo);
         Ok(Self {
             con: Conexion::nueva(base, Proveedor::GitHub, credenciales, personas),
             repo: repo.to_owned(),
diff --git a/crates/tc2/src/plataforma/mod.rs b/crates/tc2/src/plataforma/mod.rs
index d9029e2..060123b 100644
--- a/crates/tc2/src/plataforma/mod.rs
+++ b/crates/tc2/src/plataforma/mod.rs
@@ -11,9 +11,11 @@
 //! Se usa desde la biblioteca: el binario no gana subcomandos hasta que los
 //! comandos de fase (F6-2 `review`, F7 `finish`, F3-8 `doctor`) lo llamen.
 
+pub mod app_github;
 pub mod conexion;
 pub mod credenciales;
 pub mod deteccion;
+pub mod entorno_git;
 pub mod errores;
 pub mod github;
 pub mod gitlab;
@@ -21,8 +23,13 @@ pub mod http;
 pub mod personas;
 pub mod puertas;
 
-pub use credenciales::{Credenciales, CredencialesEntorno, CredencialesFijas, Rol, Secreto};
+pub use app_github::CredencialesApp;
+pub use credenciales::{
+    Credenciales, CredencialesDeEntorno, CredencialesFijas, FalloCredencial, Rol, Secreto,
+    de_entorno,
+};
 pub use deteccion::{Destino, detectar};
+pub use entorno_git::{AutorGit, entorno_git};
 pub use github::GitHub;
 pub use gitlab::GitLab;
 pub use personas::{Clave, Persona, Personas};
````

## Excluido del diff (4 fichero(s))

Su diff no se embebe: es codigo generado, lockfiles o la propia carpeta de
tareas (clave `excluir_de_revision` de `.taskcode/config.yml`). Si lo
necesitas, pidelo con:

````
git diff develop..HEAD -- ":(glob)**/dist/**" ":(glob)**/*.lock" ":(glob)**/*-lock.json" ":(glob)**/*-lock.yaml" ":(glob)**/*-lock.yml" ":(glob)**/*-lock.toml" ":(glob)tareas/**" ":(glob)tareas/*/TASK-*/aceptacion/**"
````

````
Cargo.lock                                                                                | 2 ++
 tareas/{01-en-diseno => 02-en-curso}/TASK-047/planificacion/brainstorm/peticion-plan-1.md | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-047/planificacion/plan-final.md                 | 0
 tareas/{01-en-diseno => 02-en-curso}/TASK-047/tarea.md                                    | 3 ++-
 4 files changed, 4 insertions(+), 1 deletion(-)
````

## Tests del diff (4 fichero(s))

Su diff no se embebe, para aligerar la peticion. Leelos en su ruta o pide su
diff con:

````
git diff develop..HEAD -- ":(literal)crates/tc2/tests/agente_git/mod.rs" ":(literal)crates/tc2/tests/identidades.rs" ":(literal)crates/tc2/tests/plataforma_e2e.rs" ":(literal)crates/tc2/tests/plataforma_gitlab_e2e.rs"
````

````
crates/tc2/tests/agente_git/mod.rs        | 124 ++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/identidades.rs           | 358 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 crates/tc2/tests/plataforma_e2e.rs        | 165 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++---
 crates/tc2/tests/plataforma_gitlab_e2e.rs | 165 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 4 files changed, 804 insertions(+), 8 deletions(-)
````
